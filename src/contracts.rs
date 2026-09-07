use crate::{messages, paths, store::*};
use anyhow::{Context, Result, ensure};
use hacp::v2::{Contract, ContractLimits, ContractState, Relationship, Task};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{fs, path::Path};

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Terms {
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
    pub acceptance: Vec<String>,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct Entry {
    pub contract: Contract,
    pub pending_terms: Terms,
    pub pending_digest: String,
    pub proposer: String,
    #[serde(default)]
    pub artifacts: Vec<crate::artifacts::StoredArtifact>,
    #[serde(default)]
    pub submissions: Vec<hacp::v2::Submission>,
    #[serde(default)]
    pub verifications: Vec<hacp::v2::Verification>,
    #[serde(default)]
    pub attempts: Vec<crate::verify::Attempt>,
}
pub fn entry(s: &Snapshot, id: &str) -> Result<Entry> {
    serde_json::from_value(
        s.contracts
            .get(id)
            .context("unknown contract ID; poll for current contracts")?
            .clone(),
    )
    .map_err(Into::into)
}
pub fn save(s: &mut Snapshot, e: &Entry) -> Result<()> {
    s.contracts
        .insert(e.contract.contract_id.clone(), serde_json::to_value(e)?);
    Ok(())
}
pub fn terms(root: &Path, file: &Path) -> Result<Terms> {
    let file = if file.is_absolute() {
        file.to_path_buf()
    } else {
        root.join(file)
    };
    let mut t: Terms = serde_json::from_slice(&fs::read(file)?)?;
    ensure!(!t.outputs.is_empty(), "terms require at least one output");
    ensure!(
        !t.acceptance.is_empty() && t.acceptance.iter().all(|x| !x.trim().is_empty()),
        "terms require nonempty acceptance commands"
    );
    t.inputs = paths::list(root, &t.inputs)?;
    t.outputs = paths::list(root, &t.outputs)?;
    Ok(t)
}
pub fn claims(root: &Path, s: &Snapshot, candidate: &Entry) -> Result<()> {
    for (id, v) in &s.contracts {
        if id == &candidate.contract.contract_id {
            continue;
        }
        let e: Entry = serde_json::from_value(v.clone())?;
        if matches!(
            e.contract.state,
            ContractState::Executing | ContractState::Amending | ContractState::Verifying
        ) {
            if let Some(r) = e.contract.revisions.last() {
                let t: Terms = serde_json::from_value(r.content.clone())?;
                paths::disjoint(root, &candidate.pending_terms.outputs, &t.outputs)?;
            }
        }
    }
    Ok(())
}
pub fn propose(st: &Store, s: &mut Snapshot, peer: &str, t: Terms) -> Result<Value> {
    messages::active(s)?;
    let cid = id("c");
    let task = Task {
        task_id: id("t"),
        summary: s.peers[peer].task.clone(),
        owner: urn(peer),
    };
    let mut c = Contract::propose(
        &s.session,
        &cid,
        task,
        Relationship::Collaboration,
        vec![],
        ContractLimits {
            max_rounds: 3,
            max_amendments: 3,
        },
    )?;
    let value = serde_json::to_value(&t)?;
    c.agree(&urn(peer), &value)?;
    let e = Entry {
        contract: c,
        pending_terms: t,
        pending_digest: hacp::v2::canon::digest_of(&value)?,
        proposer: peer.into(),
        artifacts: vec![],
        submissions: vec![],
        verifications: vec![],
        attempts: vec![],
    };
    save(s, &e)?;
    notify(s, peer, "contract.proposed", &e)?;
    st.commit(s)?;
    Ok(json!(e))
}
pub fn accept(st: &Store, s: &mut Snapshot, peer: &str, cid: &str, digest: &str) -> Result<Value> {
    messages::active(s)?;
    let mut e = entry(s, cid)?;
    ensure!(
        digest == e.pending_digest,
        "stale terms digest; poll and review the current proposal before accepting"
    );
    ensure!(
        peer != e.proposer,
        "proposer's acceptance is already recorded; the counterparty must accept"
    );
    claims(&st.root, s, &e)?;
    let value = serde_json::to_value(&e.pending_terms)?;
    let outcome = if e.contract.state == ContractState::Amending {
        e.contract
            .decide_amendment(&urn(peer), true, Some(value))
            .map(|_| ())
    } else {
        e.contract
            .agree(&urn(peer), &value)
            .and_then(|_| e.contract.freeze(value).map(|_| ()))
    };
    bounded(&e.contract, outcome)?;
    save(s, &e)?;
    notify(
        s,
        peer,
        if e.contract.state == ContractState::NoAgreement {
            "contract.no_agreement"
        } else {
            "contract.frozen"
        },
        &e,
    )?;
    st.commit(s)?;
    Ok(json!(e))
}
pub fn notify(s: &mut Snapshot, peer: &str, kind: &str, e: &Entry) -> Result<()> {
    messages::send(
        s,
        peer,
        kind,
        json!({"contract_id":e.contract.contract_id,"task":e.contract.task,"terms":e.pending_terms,"terms_digest":e.pending_digest,"revision":e.contract.frozen_digest(),"state":e.contract.state,"next":match e.contract.state {ContractState::Executing=>"owner implements frozen outputs then submits",ContractState::Verifying=>"counterparty verifies",ContractState::NoAgreement=>"stop work; discuss a new contract",ContractState::Settled=>"remain available to verify peer",_=>"review terms and accept digest or counter"}}),
        None,
    )?;
    Ok(())
}

pub fn bounded(
    c: &Contract,
    result: std::result::Result<(), hacp::v2::contract::ContractError>,
) -> Result<()> {
    match result {
        Ok(()) => Ok(()),
        Err(_) if c.state == ContractState::NoAgreement => Ok(()),
        Err(e) => Err(e.into()),
    }
}
pub fn counter(st: &Store, s: &mut Snapshot, peer: &str, cid: &str, t: Terms) -> Result<Value> {
    messages::active(s)?;
    let mut e = entry(s, cid)?;
    let amendment = e.contract.state == ContractState::Executing;
    if amendment {
        e.contract.propose_amendment(&urn(peer))?;
    } else {
        let result = e.contract.counter(&urn(peer));
        bounded(&e.contract, result)?;
    }
    e.pending_terms = t;
    let value = serde_json::to_value(&e.pending_terms)?;
    e.pending_digest = hacp::v2::canon::digest_of(&value)?;
    e.proposer = peer.into();
    if e.contract.state == ContractState::Amending {
        e.contract.decide_amendment(&urn(peer), true, Some(value))?;
    } else if e.contract.state != ContractState::NoAgreement {
        e.contract.agree(&urn(peer), &value)?;
    }
    save(s, &e)?;
    notify(
        s,
        peer,
        if e.contract.state == ContractState::NoAgreement {
            "contract.no_agreement"
        } else if amendment {
            "contract.amendment.proposed"
        } else {
            "contract.countered"
        },
        &e,
    )?;
    st.commit(s)?;
    Ok(json!(e))
}
pub fn decline(st: &Store, s: &mut Snapshot, peer: &str, cid: &str, digest: &str) -> Result<Value> {
    messages::active(s)?;
    let mut e = entry(s, cid)?;
    ensure!(digest == e.pending_digest, "stale terms digest; poll again");
    ensure!(
        peer != e.proposer,
        "only counterparty can decline the pending proposal"
    );
    if e.contract.state == ContractState::Amending {
        e.contract.decide_amendment(&urn(peer), false, None)?;
    } else {
        e.contract.withdraw(&urn(peer))?;
    }
    save(s, &e)?;
    notify(s, peer, "contract.declined", &e)?;
    st.commit(s)?;
    Ok(json!(e))
}
