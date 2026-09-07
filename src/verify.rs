use crate::{
    artifacts::{StoredArtifact, hash},
    contracts::{self, Entry, Terms},
    messages, paths,
    store::*,
};
use anyhow::{Context, Result, ensure};
use fs2::FileExt;
use hacp::v2::{Check, ContractState, Verdict, Verification};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    fs::{self, File, OpenOptions},
    os::unix::process::{CommandExt, ExitStatusExt},
    path::Path,
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};
#[derive(Clone, Serialize, Deserialize)]
pub struct Attempt {
    pub id: String,
    pub peer: String,
    pub status: String,
    pub commands: Vec<Measurement>,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct Measurement {
    pub command: String,
    pub stdout: String,
    pub stderr: String,
    pub duration_ms: u64,
    pub exit_code: Option<i32>,
    pub signal: Option<i32>,
    pub timed_out: bool,
}
fn lease(st: &Store, aid: &str) -> Result<File> {
    let dir = st.root.join(".hacp/verification");
    fs::create_dir_all(&dir)?;
    Ok(OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(dir.join(format!("{aid}.lock")))?)
}
pub fn recover(st: &Store, s: &mut Snapshot) -> Result<bool> {
    let mut changed = false;
    for cid in s.contracts.keys().cloned().collect::<Vec<_>>() {
        let mut e = contracts::entry(s, &cid)?;
        for a in &mut e.attempts {
            if a.status != "running" {
                continue;
            }
            let lock = lease(st, &a.id)?;
            match lock.try_lock_exclusive() {
                Ok(()) => {
                    a.status = "interrupted".into();
                    changed = true;
                    event(
                        s,
                        &a.peer,
                        "verification interrupted",
                        json!({"contract_id":cid,"attempt":a.id,"next":"inspect captured files; verify --retry-interrupted explicitly to run again"}),
                    );
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => (),
                Err(e) => return Err(e.into()),
            }
        }
        contracts::save(s, &e)?;
    }
    Ok(changed)
}
fn artifact_checks(root: &Path, records: &[StoredArtifact], phase: &str) -> Vec<Check> {
    records
        .iter()
        .map(|a| {
            let measured = (|| -> Result<()> {
                ensure!(
                    paths::normalize(root, &a.path)? == a.path,
                    "output alias changed"
                );
                for path in [&a.path, &a.record.location] {
                    let bytes = fs::read(root.join(path))
                        .with_context(|| format!("missing artifact: {path}"))?;
                    ensure!(
                        hash(&bytes) == a.record.digest && bytes.len() as u64 == a.record.size,
                        "artifact changed: {path}"
                    );
                }
                Ok(())
            })();
            Check {
                name: format!("{phase}: {}", a.path),
                passed: measured.is_ok(),
                detail: match measured {
                    Ok(()) => format!(
                        "SHA-256 {} and {} bytes match working file and immutable copy",
                        a.record.digest, a.record.size
                    ),
                    Err(e) => format!("{e:#}"),
                },
            }
        })
        .collect()
}
struct Group(i32);
impl Drop for Group {
    fn drop(&mut self) {
        unsafe {
            libc::kill(-self.0, libc::SIGKILL);
        }
    }
}
fn measure(
    root: &Path,
    aid: &str,
    index: usize,
    command: &str,
    timeout: u64,
) -> Result<Measurement> {
    let base = root
        .join(".hacp/verification")
        .join(format!("{aid}-{index}"));
    let stdout = base.with_extension("stdout");
    let stderr = base.with_extension("stderr");
    let started = Instant::now();
    let mut child = Command::new("/bin/sh")
        .args(["-c", command])
        .current_dir(root)
        .stdin(Stdio::null())
        .stdout(File::create(&stdout)?)
        .stderr(File::create(&stderr)?)
        .process_group(0)
        .spawn()?;
    let group = Group(child.id() as i32);
    let mut timed_out = false;
    let status = loop {
        if let Some(status) = child.try_wait()? {
            break status;
        }
        if started.elapsed() >= Duration::from_secs(timeout) {
            timed_out = true;
            unsafe {
                libc::kill(-group.0, libc::SIGKILL);
            }
            break child.wait()?;
        }
        thread::sleep(Duration::from_millis(20));
    };
    drop(group);
    Ok(Measurement {
        command: command.into(),
        stdout: String::from_utf8_lossy(&fs::read(stdout)?).into_owned(),
        stderr: String::from_utf8_lossy(&fs::read(stderr)?).into_owned(),
        duration_ms: started.elapsed().as_millis().try_into().unwrap_or(u64::MAX),
        exit_code: status.code(),
        signal: status.signal(),
        timed_out,
    })
}
pub fn verify(root: &Path, peer: &str, cid: &str, timeout: u64, retry: bool) -> Result<Value> {
    let st = Store::lock(root)?;
    let mut s = st.load()?;
    authorize(&s, peer)?;
    messages::active(&s)?;
    let mut e = contracts::entry(&s, cid)?;
    ensure!(
        e.contract.task.owner != urn(peer),
        "self-verification refused; only the counterparty verifies"
    );
    ensure!(
        e.contract.state == ContractState::Verifying,
        "no pending submission to verify"
    );
    ensure!(
        !e.attempts.iter().any(|a| a.status == "running"),
        "verification already running"
    );
    ensure!(
        retry || !e.attempts.last().is_some_and(|a| a.status == "interrupted"),
        "previous verification interrupted; inspect outputs then use --retry-interrupted to explicitly rerun"
    );
    let pending = e
        .contract
        .pending_submission
        .clone()
        .context("no pending submission")?;
    let terms: Terms =
        serde_json::from_value(e.contract.revisions.last().unwrap().content.clone())?;
    let records: Vec<_> = e
        .artifacts
        .iter()
        .filter(|a| pending.artifacts.contains(&a.record.artifact_id))
        .cloned()
        .collect();
    ensure!(
        records.len() == pending.artifacts.len(),
        "pending artifact metadata missing"
    );
    let aid = id("v");
    let run_lock = lease(&st, &aid)?;
    run_lock.try_lock_exclusive()?;
    e.attempts.push(Attempt {
        id: aid.clone(),
        peer: peer.into(),
        status: "running".into(),
        commands: vec![],
    });
    contracts::save(&mut s, &e)?;
    event(
        &mut s,
        peer,
        "verification started",
        json!({"contract_id":cid,"attempt":aid,"revision":pending.against_revision,"commands":terms.acceptance}),
    );
    st.commit(&s)?;
    let root = st.root.clone();
    drop(st);
    failpoint("verification_started");
    let mut checks = artifact_checks(&root, &records, "before");
    let mut measurements = vec![];
    if checks.iter().all(|c| c.passed) {
        for command in &terms.acceptance {
            let measured = measure(&root, &aid, measurements.len(), command, timeout)?;
            checks.push(Check {
                name: command.clone(),
                passed: measured.exit_code == Some(0) && !measured.timed_out,
                detail: serde_json::to_string(&measured)?,
            });
            measurements.push(measured);
            // Persist each completed command; recovery never executes commands.
            let st = Store::lock(&root)?;
            let mut s = st.load()?;
            let mut current = contracts::entry(&s, cid)?;
            ensure!(
                current.contract.pending_submission.as_ref() == Some(&pending),
                "submission changed during verification"
            );
            current
                .attempts
                .iter_mut()
                .find(|a| a.id == aid)
                .context("missing verification attempt")?
                .commands = measurements.clone();
            contracts::save(&mut s, &current)?;
            st.commit(&s)?;
        }
    }
    checks.extend(artifact_checks(&root, &records, "after"));
    let failed: Vec<_> = checks
        .iter()
        .filter(|c| !c.passed)
        .map(|c| format!("{}: {}", c.name, c.detail))
        .collect();
    let verdict = if failed.is_empty() {
        Verdict::Accept
    } else {
        Verdict::Rework {
            scope: failed.join("; "),
        }
    };
    let record = Verification::decide(
        &aid,
        &urn(peer),
        cid,
        &pending.against_revision,
        pending.artifacts.clone(),
        vec![],
        checks,
        verdict,
        failed,
        vec![],
    )?;
    let st = Store::lock(&root)?;
    let mut s = st.load()?;
    messages::active(&s)?;
    let mut current: Entry = contracts::entry(&s, cid)?;
    ensure!(
        current.contract.pending_submission.as_ref() == Some(&pending),
        "submission changed during verification; result not applied"
    );
    current.contract.apply_verification(&record)?;
    current
        .attempts
        .iter_mut()
        .find(|a| a.id == aid)
        .context("missing verification attempt")?
        .status = "completed".into();
    current.verifications.push(record.clone());
    contracts::save(&mut s, &current)?;
    event(
        &mut s,
        peer,
        "verification measured",
        json!({"record":record,"commands":measurements}),
    );
    contracts::notify(&mut s, peer, "verification.delivered", &current)?;
    st.commit(&s)?;
    drop(run_lock);
    Ok(json!({"outcome":record.verdict,"verification":record,"contract":current.contract}))
}
