use serde_json::Value;
use std::process::{Command, Output};
fn call(root: &std::path::Path, peer: &str, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_hacp"))
        .arg("--project")
        .arg(root)
        .args(["--peer", peer, "--json"])
        .args(args)
        .output()
        .unwrap()
}
fn ok(root: &std::path::Path, peer: &str, args: &[&str]) -> Value {
    let out = call(root, peer, args);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stdout)
    );
    serde_json::from_slice(&out.stdout).unwrap()
}
#[test]
fn session_recovers_and_does_not_restart() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    assert_eq!(
        ok(root, "a", &["start", "write parser"])["session"]["state"],
        "opening"
    );
    assert!(!call(root, "a", &["join", "bad"]).status.success());
    assert_eq!(
        ok(root, "b", &["join", "write tests"])["session"]["state"],
        "active"
    );
    std::fs::remove_file(root.join(".hacp/log.md")).unwrap();
    assert_eq!(ok(root, "a", &["status"])["session"]["state"], "active");
    assert!(root.join(".hacp/log.md").exists());
    assert!(!call(root, "a", &["start", "again"]).status.success());
    ok(root, "b", &["close", "--reason", "done"]);
    assert_eq!(ok(root, "a", &["status"])["session"]["state"], "closed");
}
fn pair(root: &std::path::Path) {
    ok(root, "a", &["start", "write parser"]);
    ok(root, "b", &["join", "write tests"]);
}
#[test]
fn crossed_questions_answers_history_and_timeout() {
    let dir = tempfile::tempdir().unwrap();
    let r = dir.path();
    pair(r);
    let a = ok(r, "a", &["ask", "interface?"])["message_id"]
        .as_str()
        .unwrap()
        .to_string();
    let b = ok(r, "b", &["ask", "format?"])["message_id"]
        .as_str()
        .unwrap()
        .to_string();
    assert_eq!(
        ok(r, "a", &["wait", "--timeout", "0"])["messages"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(ok(r, "b", &["poll"])["messages"][0]["message_id"], a);
    ok(r, "a", &["answer", &b, "json"]);
    ok(r, "b", &["answer", &a, "parse()"]);
    assert_eq!(ok(r, "a", &["poll"])["messages"][0]["in_reply_to"], a);
    assert_eq!(
        ok(r, "a", &["poll", "--all"])["messages"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    assert!(!call(r, "a", &["wait", "--timeout", "0"]).status.success());
    assert_eq!(
        ok(r, "b", &["poll"])["outstanding_questions"]
            .as_array()
            .unwrap()
            .len(),
        0
    );
}
#[test]
fn committed_delivery_recovers_once() {
    let dir = tempfile::tempdir().unwrap();
    let r = dir.path();
    pair(r);
    let out = Command::new(env!("CARGO_BIN_EXE_hacp"))
        .arg("--project")
        .arg(r)
        .args(["--peer", "a", "ask", "hello"])
        .env("HACP_TEST_CRASH", "after_commit")
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(86));
    let first = ok(r, "b", &["poll"]);
    assert_eq!(first["messages"].as_array().unwrap().len(), 1);
    let m = &first["messages"][0];
    std::fs::write(
        r.join(".hacp/inbox/b/duplicate.json"),
        serde_json::to_vec(m).unwrap(),
    )
    .unwrap();
    assert_eq!(
        ok(r, "b", &["poll"])["messages"].as_array().unwrap().len(),
        0
    );
    let s = ok(r, "a", &["status"]);
    assert_eq!(s["messages"].as_array().unwrap().len(), 1);
    assert_eq!(s["events"].as_array().unwrap().len(), 3);
}
fn proposal(r: &std::path::Path, peer: &str, outputs: &[&str], commands: &[&str]) -> Value {
    let name = format!("terms-{peer}.json");
    std::fs::write(
        r.join(&name),
        serde_json::to_vec(
            &serde_json::json!({"inputs":[],"outputs":outputs,"acceptance":commands}),
        )
        .unwrap(),
    )
    .unwrap();
    ok(r, peer, &["propose", "--terms", &name])
}
#[test]
fn initial_aliases_and_invalid_ownership() {
    let dir = tempfile::tempdir().unwrap();
    let r = dir.path();
    std::fs::write(r.join("real"), "x").unwrap();
    std::fs::hard_link(r.join("real"), r.join("alias")).unwrap();
    ok(r, "a", &["start", "a", "--owns", "real"]);
    for p in ["alias", "../outside", ".hacp/session.json", "*.rs", "."] {
        assert!(
            !call(r, "b", &["join", "b", "--owns", p]).status.success(),
            "{p}"
        );
    }
    ok(r, "b", &["join", "b", "--owns", "other"]);
}
#[test]
fn simultaneous_conflicting_freezes_cannot_both_succeed() {
    let dir = tempfile::tempdir().unwrap();
    let r = dir.path();
    pair(r);
    let a = proposal(r, "a", &["shared.txt"], &["test -f shared.txt"]);
    let b = proposal(r, "b", &["shared.txt"], &["test -f shared.txt"]);
    let spawn = |peer: &str, v: &Value| {
        Command::new(env!("CARGO_BIN_EXE_hacp"))
            .arg("--project")
            .arg(r)
            .args([
                "--peer",
                peer,
                "--json",
                "accept",
                v["contract"]["contract_id"].as_str().unwrap(),
                v["pending_digest"].as_str().unwrap(),
            ])
            .stdout(std::process::Stdio::piped())
            .spawn()
            .unwrap()
    };
    let p = spawn("b", &a);
    let q = spawn("a", &b);
    let p = p.wait_with_output().unwrap();
    let q = q.wait_with_output().unwrap();
    assert_ne!(p.status.success(), q.status.success());
    let state = ok(r, "a", &["status"]);
    assert_eq!(
        state["contracts"]
            .as_object()
            .unwrap()
            .values()
            .filter(|e| e["contract"]["state"] == "executing")
            .count(),
        1
    );
}
fn accept(r: &std::path::Path, peer: &str, p: &Value) -> Value {
    ok(
        r,
        peer,
        &[
            "accept",
            p["contract"]["contract_id"].as_str().unwrap(),
            p["pending_digest"].as_str().unwrap(),
        ],
    )
}
fn amend(r: &std::path::Path, peer: &str, cid: &str, output: &str) -> Value {
    std::fs::write(r.join("amend.json"),serde_json::to_vec(&serde_json::json!({"inputs":[],"outputs":[output],"acceptance":[format!("test -f {output}")]})).unwrap()).unwrap();
    ok(r, peer, &["propose", cid, "--terms", "amend.json"])
}
#[test]
fn counter_amendment_stale_submission_and_bounds() {
    let dir = tempfile::tempdir().unwrap();
    let r = dir.path();
    pair(r);
    let p = proposal(r, "a", &["a.txt"], &["test -f a.txt"]);
    let cid = p["contract"]["contract_id"].as_str().unwrap();
    let q = amend(r, "b", cid, "b.txt");
    assert!(
        !call(
            r,
            "b",
            &["accept", cid, p["pending_digest"].as_str().unwrap()]
        )
        .status
        .success()
    );
    let frozen = accept(r, "a", &q);
    let rev = frozen["contract"]["revisions"][0]["digest"]
        .as_str()
        .unwrap();
    let next = amend(r, "a", cid, "c.txt");
    assert_eq!(next["contract"]["revisions"][0]["digest"], rev);
    let next = accept(r, "b", &next);
    assert_eq!(next["contract"]["revisions"].as_array().unwrap().len(), 2);
    assert!(!call(r, "a", &["submit", cid, rev]).status.success());
    std::fs::write(r.join("c.txt"), "data").unwrap();
    let rev = next["contract"]["revisions"][1]["digest"].as_str().unwrap();
    assert!(!call(r, "b", &["submit", cid, rev]).status.success());
    let submitted = ok(r, "a", &["submit", cid, rev]);
    assert_eq!(submitted["contract"]["state"], "verifying");
    let copy = submitted["artifacts"][0]["record"]["location"]
        .as_str()
        .unwrap();
    std::fs::write(r.join("c.txt"), "changed").unwrap();
    assert_eq!(std::fs::read_to_string(r.join(copy)).unwrap(), "data");
    let p = proposal(r, "a", &["z"], &["true"]);
    let cid = p["contract"]["contract_id"].as_str().unwrap();
    amend(r, "b", cid, "z1");
    amend(r, "a", cid, "z2");
    let end = amend(r, "b", cid, "z3");
    assert_eq!(end["contract"]["state"], "noagreement");
}
#[test]
fn amendment_collision_preserves_prior_claims_and_amendment_limit() {
    let dir = tempfile::tempdir().unwrap();
    let r = dir.path();
    pair(r);
    let a = accept(r, "b", &proposal(r, "a", &["a.txt"], &["true"]));
    let cid = a["contract"]["contract_id"].as_str().unwrap();
    let b = accept(r, "a", &proposal(r, "b", &["b.txt"], &["true"]));
    let bid = b["contract"]["contract_id"].as_str().unwrap();
    let a2 = amend(r, "a", cid, "b.txt");
    assert!(
        !call(
            r,
            "b",
            &["accept", cid, a2["pending_digest"].as_str().unwrap()]
        )
        .status
        .success()
    );
    let b2 = amend(r, "b", bid, "a.txt");
    assert!(
        !call(
            r,
            "a",
            &["accept", bid, b2["pending_digest"].as_str().unwrap()]
        )
        .status
        .success()
    );
    ok(
        r,
        "b",
        &["decline", cid, a2["pending_digest"].as_str().unwrap()],
    );
    for n in 0..4 {
        let p = amend(r, "a", cid, &format!("new{n}.txt"));
        let e = accept(r, "b", &p);
        assert_eq!(
            e["contract"]["state"],
            if n == 3 { "noagreement" } else { "executing" }
        );
    }
}
fn submitted(r: &std::path::Path, commands: &[&str]) -> (String, String) {
    let e = accept(r, "b", &proposal(r, "a", &["out.txt"], commands));
    let cid = e["contract"]["contract_id"].as_str().unwrap().to_string();
    let rev = e["contract"]["revisions"][0]["digest"]
        .as_str()
        .unwrap()
        .to_string();
    std::fs::write(r.join("out.txt"), "bad\n").unwrap();
    ok(r, "a", &["submit", &cid, &rev]);
    (cid, rev)
}
#[test]
fn verification_rework_repair_and_self_refusal() {
    let dir = tempfile::tempdir().unwrap();
    let r = dir.path();
    pair(r);
    let (cid, rev) = submitted(r, &["printf 'measured\\n'; test \"$(cat out.txt)\" = good"]);
    assert!(!call(r, "a", &["verify", &cid]).status.success());
    let result = ok(r, "b", &["verify", &cid]);
    assert_eq!(result["contract"]["state"], "executing");
    assert!(result["outcome"]["rework"].is_object());
    std::fs::write(r.join("out.txt"), "good\n").unwrap();
    ok(r, "a", &["submit", &cid, &rev]);
    assert_eq!(ok(r, "b", &["verify", &cid])["outcome"], "accept");
    assert!(
        std::fs::read_to_string(r.join(".hacp/log.md"))
            .unwrap()
            .contains("measured")
    );
}
#[test]
fn changed_missing_and_command_modified_artifacts_request_rework() {
    for mode in ["changed", "missing", "during"] {
        let dir = tempfile::tempdir().unwrap();
        let r = dir.path();
        pair(r);
        let (cid, _) = submitted(
            r,
            &[if mode == "during" {
                "printf changed > out.txt"
            } else {
                "true"
            }],
        );
        if mode == "changed" {
            std::fs::write(r.join("out.txt"), "changed").unwrap();
        }
        if mode == "missing" {
            std::fs::remove_file(r.join("out.txt")).unwrap();
        }
        assert_eq!(
            ok(r, "b", &["verify", &cid])["contract"]["state"],
            "executing"
        );
    }
}
#[test]
fn verification_timeout_kills_descendants_and_releases_metadata_lock() {
    let dir = tempfile::tempdir().unwrap();
    let r = dir.path();
    pair(r);
    let (cid, _) = submitted(r, &["(sleep 2; touch escaped) & wait"]);
    let mut child = Command::new(env!("CARGO_BIN_EXE_hacp"))
        .arg("--project")
        .arg(r)
        .args(["--peer", "b", "--json", "verify", &cid, "--timeout", "1"])
        .stdout(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    std::thread::sleep(std::time::Duration::from_millis(300));
    ok(r, "a", &["ask", "still responsive?"]);
    let result = child.wait().unwrap();
    assert!(result.success());
    std::thread::sleep(std::time::Duration::from_secs(2));
    assert!(!r.join("escaped").exists());
    let s = ok(r, "a", &["status"]);
    let e = &s["contracts"][&cid];
    assert_eq!(e["attempts"][0]["commands"][0]["timed_out"], true);
}
#[test]
fn interrupted_verification_requires_explicit_retry() {
    let dir = tempfile::tempdir().unwrap();
    let r = dir.path();
    pair(r);
    let (cid, _) = submitted(r, &["touch ran"]);
    let out = Command::new(env!("CARGO_BIN_EXE_hacp"))
        .arg("--project")
        .arg(r)
        .args(["--peer", "b", "verify", &cid])
        .env("HACP_TEST_CRASH", "verification_started")
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(86));
    let s = ok(r, "a", &["status"]);
    assert_eq!(s["contracts"][&cid]["attempts"][0]["status"], "interrupted");
    assert!(!r.join("ran").exists());
    assert!(!call(r, "b", &["verify", &cid]).status.success());
    assert_eq!(
        ok(r, "b", &["verify", &cid, "--retry-interrupted"])["outcome"],
        "accept"
    );
}
#[test]
fn advisory_lock_timeout_and_process_release() {
    use fs2::FileExt;
    let dir = tempfile::tempdir().unwrap();
    let r = dir.path();
    pair(r);
    let file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(r.join(".hacp/lock"))
        .unwrap();
    file.lock_exclusive().unwrap();
    let started = std::time::Instant::now();
    let o = call(r, "a", &["status"]);
    assert!(!o.status.success());
    assert!(started.elapsed() >= std::time::Duration::from_secs(3));
    assert!(String::from_utf8_lossy(&o.stdout).contains("busy project"));
    drop(file);
    ok(r, "a", &["status"]);
}
