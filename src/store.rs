use anyhow::{Context, Result, bail};
use fs2::FileExt;
use hacp::v2::{Envelope, Session};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    thread,
    time::{Duration, Instant},
};

#[derive(Clone, Serialize, Deserialize)]
pub struct Registration {
    pub task: String,
    pub owns: Vec<String>,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub format: u32,
    pub session: Session,
    pub peers: BTreeMap<String, Registration>,
    pub processed: BTreeSet<String>,
    pub messages: Vec<Envelope>,
    pub fetched: BTreeMap<String, BTreeSet<String>>,
    pub contracts: BTreeMap<String, Value>,
    pub events: Vec<Value>,
}
pub fn urn(peer: &str) -> String {
    format!("urn:hacp:agent:{peer}")
}
pub fn other(peer: &str) -> &'static str {
    if peer == "a" { "b" } else { "a" }
}
pub fn id(prefix: &str) -> String {
    format!("{prefix}-{}", uuid::Uuid::new_v4().simple())
}
pub struct Store {
    pub root: PathBuf,
    _lock: File,
}
impl Store {
    pub fn lock(root: &Path) -> Result<Self> {
        let root = root
            .canonicalize()
            .context("project directory does not exist")?;
        let state = root.join(".hacp");
        if fs::symlink_metadata(&state).is_ok_and(|m| m.file_type().is_symlink()) {
            bail!(".hacp must not be a symlink");
        }
        fs::create_dir_all(&state)?;
        let lock = OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(state.join("lock"))?;
        let start = Instant::now();
        loop {
            match lock.try_lock_exclusive() {
                Ok(()) => break,
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    if start.elapsed() >= Duration::from_secs(3) {
                        bail!(
                            "busy project {}: metadata lock timed out after three seconds",
                            root.display()
                        );
                    }
                    thread::sleep(Duration::from_millis(25));
                }
                Err(e) => return Err(e.into()),
            }
        }
        Ok(Self { root, _lock: lock })
    }
    pub fn load(&self) -> Result<Snapshot> {
        let mut s: Snapshot = serde_json::from_slice(
            &fs::read(self.root.join(".hacp/session.json"))
                .context("no session; peer a must run start")?,
        )?;
        anyhow::ensure!(s.format == 1, "unsupported snapshot format");
        if crate::verify::recover(self, &mut s)? {
            atomic(
                &self.root.join(".hacp/session.json"),
                &serde_json::to_vec_pretty(&s)?,
            )?;
        }
        self.project(&s, false)?;
        Ok(s)
    }
    pub fn commit(&self, s: &Snapshot) -> Result<()> {
        failpoint("before_commit");
        atomic(
            &self.root.join(".hacp/session.json"),
            &serde_json::to_vec_pretty(s)?,
        )?;
        failpoint("after_commit");
        self.project(s, true)
    }
    fn project(&self, s: &Snapshot, crash: bool) -> Result<()> {
        let state = self.root.join(".hacp");
        for (key, c) in &s.contracts {
            atomic(
                &state.join("contracts").join(format!("{key}.json")),
                &serde_json::to_vec_pretty(c)?,
            )?;
        }
        for m in &s.messages {
            let peer = hacp::v2::envelope::agent_urn::parse(&m.to).map_err(anyhow::Error::msg)?;
            atomic(
                &state
                    .join("inbox")
                    .join(peer)
                    .join(format!("{}.json", m.message_id)),
                &serde_json::to_vec_pretty(m)?,
            )?;
        }
        if crash {
            failpoint("after_delivery");
        }
        let mut log = String::from("HACP collaboration log\n\n");
        for e in &s.events {
            if let Some(rendered) = e["rendered"].as_str() {
                log.push_str(rendered);
                log.push_str("\n\n");
            } else {
                log.push_str(&format!("{}\n", serde_json::to_string(e)?));
            }
        }
        atomic(&state.join("log.md"), log.as_bytes())
    }
}
pub fn atomic(path: &Path, bytes: &[u8]) -> Result<()> {
    let parent = path.parent().context("missing parent")?;
    fs::create_dir_all(parent)?;
    let tmp = parent.join(format!(".tmp-{}", uuid::Uuid::new_v4()));
    let result = (|| -> Result<()> {
        let mut file = OpenOptions::new().create_new(true).write(true).open(&tmp)?;
        file.write_all(bytes)?;
        file.sync_all()?;
        fs::rename(&tmp, path)?;
        File::open(parent)?.sync_all()?;
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(tmp);
    }
    result
}
pub fn event(s: &mut Snapshot, peer: &str, action: &str, detail: Value) {
    let eid = id("e");
    let time = hacp::v2::canon::canonical_now();
    let mut rendered = format!("{time} | Peer {peer} | {action}\nEvent {eid}\n");
    let readable = serde_json::to_string_pretty(&detail).expect("JSON value serializes");
    for line in readable.lines() {
        rendered.push_str("    ");
        rendered.push_str(line);
        rendered.push('\n');
    }
    s.events.push(json!({"id":eid,"time":time,"peer":peer,"action":action,"detail":detail,"rendered":rendered}));
}
pub fn authorize(s: &Snapshot, peer: &str) -> Result<()> {
    s.session.authorize_author(&urn(peer))?;
    anyhow::ensure!(s.peers.contains_key(peer), "peer {peer} has not joined");
    Ok(())
}
pub fn failpoint(name: &str) {
    if cfg!(debug_assertions) && std::env::var("HACP_TEST_CRASH").as_deref() == Ok(name) {
        std::process::exit(86);
    }
}
