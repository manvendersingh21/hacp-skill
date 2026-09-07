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
