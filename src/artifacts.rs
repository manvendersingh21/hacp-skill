use crate::{
    contracts::{self, Terms},
    messages, paths,
    store::*,
};
use anyhow::{Result, ensure};
use hacp::v2::{Artifact, ContractState, Submission};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    fs::{self, OpenOptions},
    io::Write,
    os::unix::fs::PermissionsExt,
    path::Path,
};
#[derive(Clone, Serialize, Deserialize)]
pub struct StoredArtifact {
    pub path: String,
    pub record: Artifact,
}
pub fn submit(root: &Path, peer: &str, cid: &str, revision: &str, claim: &str) -> Result<Value> {
    let st = Store::lock(root)?;
    let s = st.load()?;
    authorize(&s, peer)?;
    messages::active(&s)?;
    let mut e = contracts::entry(&s, cid)?;
    ensure!(
        e.contract.task.owner == urn(peer),
        "only task owner can submit"
    );
    ensure!(
        e.contract.state == ContractState::Executing,
        "contract is not executing"
    );
    ensure!(
        e.contract.frozen_digest() == Some(revision),
        "stale revision; poll and submit the current frozen revision"
    );
    let before = serde_json::to_value(&e)?;
    let t: Terms = serde_json::from_value(e.contract.revisions.last().unwrap().content.clone())?;
    let root = st.root.clone();
    drop(st);
    let mut records = vec![];
    for output in &t.outputs {
        let path = paths::normalize(&root, output)?;
        ensure!(
            &path == output,
            "output alias changed since freeze: {output}; amend before submitting"
        );
        let bytes = fs::read(root.join(&path))?;
        let digest = hash(&bytes);
        let uuid = uuid::Uuid::new_v4();
        let location = format!(".hacp/artifacts/{uuid}.bin");
        fs::create_dir_all(root.join(".hacp/artifacts"))?;
        let dest = root.join(&location);
        let tmp = dest.with_extension("tmp");
        let mut file = OpenOptions::new().write(true).create_new(true).open(&tmp)?;
        file.write_all(&bytes)?;
        file.sync_all()?;
        file.set_permissions(fs::Permissions::from_mode(0o444))?;
        fs::rename(tmp, &dest)?;
        std::fs::File::open(dest.parent().unwrap())?.sync_all()?;
        let record = Artifact::new(
            &format!("urn:hacp:artifact:{uuid}"),
            "application/octet-stream",
            &digest,
            bytes.len() as u64,
            &urn(peer),
            &e.contract.task.task_id,
            cid,
            revision,
            &location,
        )?;
        records.push(StoredArtifact { path, record });
    }
    let submission = Submission {
        against_revision: revision.into(),
        artifacts: records
            .iter()
            .map(|a| a.record.artifact_id.clone())
            .collect(),
        evidence: vec![],
        claim: claim.into(),
    };
    let st = Store::lock(&root)?;
    let mut s = st.load()?;
    messages::active(&s)?;
    ensure!(
        serde_json::to_value(contracts::entry(&s, cid)?)? == before,
        "contract changed while copying artifacts; poll and retry submission"
    );
    // Filesystem aliases can change while bytes are copied; recheck all frozen claims.
    e.pending_terms = t;
    contracts::claims(&root, &s, &e)?;
    e.contract.submit(&urn(peer), submission.clone())?;
    e.submissions.push(submission);
    e.artifacts.extend(records);
    contracts::save(&mut s, &e)?;
    contracts::notify(&mut s, peer, "submission.delivered", &e)?;
    event(
        &mut s,
        peer,
        "artifacts preserved",
        json!({"contract_id":cid,"artifacts":e.artifacts,"claim":claim}),
    );
    st.commit(&s)?;
    Ok(json!(e))
}

pub fn hash(bytes: &[u8]) -> String {
    use sha2::Digest;
    format!("{:x}", sha2::Sha256::digest(bytes))
}
