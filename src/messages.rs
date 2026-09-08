use crate::store::*;
use anyhow::{Result, ensure};
use hacp::v2::{Envelope, SessionState};
use serde_json::{Value, json};
use std::{
    fs,
    path::Path,
    thread,
    time::{Duration, Instant},
};

pub fn send(
    s: &mut Snapshot,
    peer: &str,
    kind: &str,
    body: Value,
    reply: Option<String>,
) -> Result<String> {
    let mut m = Envelope::new(
        &s.session.session_id,
        urn(peer),
        urn(other(peer)),
        kind,
        body,
    );
    m.in_reply_to = reply;
    receive(s, peer, &m, false)?;
    Ok(m.message_id)
}
fn receive(s: &mut Snapshot, peer: &str, m: &Envelope, incoming: bool) -> Result<()> {
    m.validate()?;
    s.session.authorize_author(&m.from)?;
    s.session.authorize_author(&m.to)?;
    ensure!(
        m.session_id == s.session.session_id,
        "message belongs to another session"
    );
    ensure!(m.from != m.to, "message must address the other peer");
    ensure!(
        if incoming {
            m.to == urn(peer)
        } else {
            m.from == urn(peer)
        },
        "incorrect message recipient or sender"
    );
    ensure!(
        s.peers.contains_key(
            hacp::v2::envelope::agent_urn::parse(&m.from).map_err(anyhow::Error::msg)?
        ),
        "sender has not joined"
    );
    if s.processed.contains(&m.message_id) {
        return Ok(());
    }
    if m.kind == "hacp.skill.answer" {
        let question = s
            .messages
            .iter()
            .find(|q| Some(&q.message_id) == m.in_reply_to.as_ref())
            .ok_or_else(|| anyhow::anyhow!("answer must reference an existing question"))?;
        ensure!(
            question.kind == "hacp.skill.ask" && question.from == m.to && question.to == m.from,
            "answer must address the question's author"
        );
        ensure!(
            !s.messages
                .iter()
                .any(|x| x.kind == "hacp.skill.answer" && x.in_reply_to == m.in_reply_to),
            "question already answered"
        );
    }
    if matches!(m.kind.as_str(), "hacp.skill.ask" | "hacp.skill.answer") {
        ensure!(
            m.body["text"]
                .as_str()
                .is_some_and(|t| !t.trim().is_empty()),
            "message text must not be empty"
        );
    }
    s.processed.insert(m.message_id.clone());
    s.messages.push(m.clone());
    event(
        s,
        hacp::v2::envelope::agent_urn::parse(&m.from).map_err(anyhow::Error::msg)?,
        &m.kind,
        json!({"message_id":m.message_id,"to":m.to,"in_reply_to":m.in_reply_to,"body":m.body}),
    );
    Ok(())
}
pub fn ingest(st: &Store, s: &mut Snapshot, peer: &str) -> Result<bool> {
    let dir = st.root.join(".hacp/inbox").join(peer);
    let mut paths = if dir.exists() {
        fs::read_dir(dir)?
            .map(|x| x.map(|x| x.path()))
            .collect::<std::io::Result<Vec<_>>>()?
    } else {
        vec![]
    };
    paths.sort();
    let before = s.messages.len();
    for p in paths {
        if p.extension().is_some_and(|x| x == "json") {
            let m: Envelope = serde_json::from_slice(&fs::read(&p)?)?;
            receive(s, peer, &m, true)?;
        }
    }
    Ok(s.messages.len() != before)
}
pub fn outstanding_questions(s: &Snapshot) -> Vec<&Envelope> {
    s.messages
        .iter()
        .filter(|m| {
            m.kind == "hacp.skill.ask"
                && !s.messages.iter().any(|a| {
                    a.kind == "hacp.skill.answer" && a.in_reply_to.as_ref() == Some(&m.message_id)
                })
        })
        .collect()
}
pub fn view(s: &Snapshot, peer: &str, all: bool) -> Value {
    let fetched = s.fetched.get(peer);
    let messages: Vec<_> = s
        .messages
        .iter()
        .filter(|m| {
            m.to == urn(peer) && (all || !fetched.is_some_and(|f| f.contains(&m.message_id)))
        })
        .collect();
    let questions = outstanding_questions(s);
    let actions:Vec<_>=s.contracts.values().filter_map(|v| {
        let e:crate::contracts::Entry=serde_json::from_value(v.clone()).ok()?;
        use hacp::v2::ContractState::*;
        let action=match e.contract.state {
            Proposed|Countered|Amending if e.proposer!=peer=>"review and accept pending digest or counter",
            Verifying if e.contract.task.owner!=urn(peer)=>"verify pending submission",
            Executing if e.contract.task.owner==urn(peer) && e.contract.rework_scope.is_some()=>"repair and resubmit",
            _=>return None,
        };
        Some(json!({"contract_id":e.contract.contract_id,"action":action,"pending_digest":e.pending_digest,"revision":e.contract.frozen_digest()}))
    }).collect();
    json!({"contract_actions":actions,"peer":peer,"session_state":s.session.state,"outcome":s.outcome,"messages":messages,"outstanding_questions":questions,"contracts":s.contracts})
}
pub fn poll(root: &Path, peer: &str, all: bool, timeout: Option<u64>) -> Result<Value> {
    let start = Instant::now();
    loop {
        let st = Store::lock(root)?;
        let mut s = st.load()?;
        authorize(&s, peer)?;
        ingest(&st, &mut s, peer)?;
        let result = view(&s, peer, all);
        let ready = !result["messages"].as_array().unwrap().is_empty()
            || result["outstanding_questions"]
                .as_array()
                .unwrap()
                .iter()
                .any(|q| q["to"] == urn(peer))
            || !result["contract_actions"].as_array().unwrap().is_empty();
        if ready || timeout.is_none() {
            for m in &s.messages {
                if m.to == urn(peer) {
                    s.fetched
                        .entry(peer.into())
                        .or_default()
                        .insert(m.message_id.clone());
                }
            }
            st.commit(&s)?;
            return Ok(result);
        }
        st.commit(&s)?;
        if start.elapsed() >= Duration::from_secs(timeout.unwrap()) {
            anyhow::bail!(
                "wait timed out for peer {peer}; no consent inferred. Outstanding work: {}",
                serde_json::to_string(&result)?
            );
        }
        drop(st);
        thread::sleep(Duration::from_millis(250));
    }
}
pub fn active(s: &Snapshot) -> Result<()> {
    ensure!(
        s.session.state == SessionState::Active,
        "session must be active"
    );
    Ok(())
}
