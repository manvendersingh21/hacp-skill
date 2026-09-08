use crate::{contracts, messages, store::*};
use anyhow::{Result, ensure};
use hacp::v2::{ContractState, Verdict};
use serde_json::{Value, json};
use std::collections::BTreeSet;

/// Every recorded contract is required: the binding has no optional-work marker.
/// Questions use the same blocking definition as poll/wait, regardless of fetches.
pub fn blockers(s: &Snapshot) -> Result<Vec<String>> {
    let mut blockers = vec![];
    if s.contracts.is_empty() {
        blockers.push("session has no contracts".into());
    }
    for cid in s.contracts.keys() {
        let e = contracts::entry(s, cid)?;
        if e.contract.state != ContractState::Settled {
            blockers.push(format!(
                "contract {cid} is {} (requires settled)",
                serde_json::to_value(e.contract.state)?.as_str().unwrap()
            ));
        }
        // Settlement alone is insufficient: the core also exposes a low-level
        // verdict transition. Require a valid stored counterparty record bound
        // to the latest submission and current frozen revision.
        let accepted = e.submissions.last().is_some_and(|submission| {
            e.contract.frozen_digest() == Some(submission.against_revision.as_str())
                && e.verifications.last().is_some_and(|v| {
                    v.verdict == Verdict::Accept
                        && v.validate().is_ok()
                        && v.contract_id == *cid
                        && v.against_revision == submission.against_revision
                        && e.contract.participants.contains(&v.verifier)
                        && v.verifier != e.contract.task.owner
                        && v.artifacts.iter().collect::<BTreeSet<_>>()
                            == submission.artifacts.iter().collect::<BTreeSet<_>>()
                })
        });
        if !accepted {
            blockers.push(format!(
                "contract {cid} has no accepted counterparty verification for its latest submission and frozen revision"
            ));
        }
    }
    for question in messages::outstanding_questions(s) {
        blockers.push(format!(
            "question {} remains unanswered",
            question.message_id
        ));
    }
    Ok(blockers)
}

pub fn complete(st: &Store, s: &mut Snapshot, peer: &str) -> Result<Value> {
    messages::active(s)?;
    // The caller holds the metadata lock through validation and commit.
    let blockers = blockers(s)?;
    ensure!(
        blockers.is_empty(),
        "cannot complete:\n- {}",
        blockers.join("\n- ")
    );
    let reason = "all contracts settled and verified; no unanswered questions";
    s.session.close(&urn(peer), reason)?;
    s.outcome = Some(Outcome::Completed);
    messages::send(
        s,
        peer,
        "session.close",
        json!({"reason":reason,"outcome":"completed"}),
        None,
    )?;
    event(s, peer, "complete", json!({"outcome":"completed"}));
    st.commit(s)?;
    Ok(json!({"session":s.session,"outcome":s.outcome}))
}
