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
#[test]
fn install_preflights_all_destinations_and_is_idempotent() {
    let home = tempfile::tempdir().unwrap();
    let r = home.path();
    let conflict = r.join(".gemini/config/skills/hacp/SKILL.md");
    std::fs::create_dir_all(conflict.parent().unwrap()).unwrap();
    std::fs::write(&conflict, "user skill").unwrap();
    let run = |cmd: &str| {
        Command::new(env!("CARGO_BIN_EXE_hacp"))
            .args(["--json", cmd, "--cli", "all", "--home"])
            .arg(r)
            .output()
            .unwrap()
    };
    let doc = run("doctor");
    assert!(doc.status.success());
    let doc: Value = serde_json::from_slice(&doc.stdout).unwrap();
    assert_eq!(doc["conflicts"], true);
    assert_eq!(doc["model_calls"], 0);
    assert!(!run("install").status.success());
    assert!(!r.join(".agents").exists());
    assert_eq!(std::fs::read_to_string(&conflict).unwrap(), "user skill");
    std::fs::remove_file(conflict).unwrap();
    assert!(run("install").status.success());
    assert!(run("install").status.success());
    for p in [
        ".agents/skills/hacp/SKILL.md",
        ".claude/skills/hacp/SKILL.md",
        ".gemini/config/skills/hacp/SKILL.md",
        ".config/opencode/commands/hacp.md",
    ] {
        assert!(r.join(p).is_file(), "{p}");
    }
    assert!(
        !r.join(".config/opencode/skills/hacp").exists(),
        "reuse identical discoverable skill"
    );
}
#[test]
fn crash_before_commit_and_after_delivery_has_one_or_no_effect() {
    for point in ["before_commit", "after_delivery"] {
        let dir = tempfile::tempdir().unwrap();
        let r = dir.path();
        pair(r);
        let out = Command::new(env!("CARGO_BIN_EXE_hacp"))
            .arg("--project")
            .arg(r)
            .args(["--peer", "a", "ask", "unique question"])
            .env("HACP_TEST_CRASH", point)
            .output()
            .unwrap();
        assert_eq!(out.status.code(), Some(86));
        let s = ok(r, "b", &["poll"]);
        let count = if point == "before_commit" { 0 } else { 1 };
        assert_eq!(s["messages"].as_array().unwrap().len(), count);
        let log = std::fs::read_to_string(r.join(".hacp/log.md")).unwrap();
        assert_eq!(log.matches("unique question").count(), count);
        ok(r, "a", &["status"]);
        assert_eq!(
            std::fs::read_to_string(r.join(".hacp/log.md")).unwrap(),
            log
        );
    }
}
#[test]
fn forged_envelopes_are_refused_and_fetched_question_remains_actionable() {
    let dir = tempfile::tempdir().unwrap();
    let r = dir.path();
    pair(r);
    ok(r, "a", &["ask", "question"]);
    let p = ok(r, "b", &["poll"]);
    assert_eq!(
        ok(r, "b", &["wait", "--timeout", "0"])["outstanding_questions"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    for field in ["from", "to", "session_id", "protocol"] {
        let mut m = p["messages"][0].clone();
        m["message_id"] = serde_json::json!("m-000000000000000000");
        m[field] = serde_json::json!(if field == "from" || field == "to" {
            "urn:hacp:agent:outsider"
        } else {
            "wrong"
        });
        let path = r.join(".hacp/inbox/b/forged.json");
        std::fs::write(&path, serde_json::to_vec(&m).unwrap()).unwrap();
        assert!(!call(r, "b", &["poll"]).status.success(), "{field}");
        std::fs::remove_file(path).unwrap();
    }
}
#[test]
fn case_aliases_and_symlink_escapes_are_checked_before_freeze() {
    let dir = tempfile::tempdir().unwrap();
    let r = dir.path();
    pair(r);
    std::fs::write(r.join("CaseProbe"), "probe").unwrap();
    if r.join("caseprobe").exists() {
        let p = proposal(r, "a", &["Future.txt"], &["true"]);
        accept(r, "b", &p);
        let q = proposal(r, "b", &["future.txt"], &["true"]);
        assert!(
            !call(
                r,
                "a",
                &[
                    "accept",
                    q["contract"]["contract_id"].as_str().unwrap(),
                    q["pending_digest"].as_str().unwrap()
                ]
            )
            .status
            .success()
        );
    }
    let q = proposal(r, "a", &["later.txt"], &["true"]);
    let external = tempfile::NamedTempFile::new().unwrap();
    std::os::unix::fs::symlink(external.path(), r.join("later.txt")).unwrap();
    assert!(
        !call(
            r,
            "b",
            &[
                "accept",
                q["contract"]["contract_id"].as_str().unwrap(),
                q["pending_digest"].as_str().unwrap()
            ]
        )
        .status
        .success()
    );
}
#[test]
fn concurrent_questions_preserve_every_message() {
    let dir = tempfile::tempdir().unwrap();
    let r = dir.path();
    pair(r);
    let children: Vec<_> = (0..6)
        .map(|n| {
            Command::new(env!("CARGO_BIN_EXE_hacp"))
                .arg("--project")
                .arg(r)
                .args([
                    "--peer",
                    if n % 2 == 0 { "a" } else { "b" },
                    "--json",
                    "ask",
                    &format!("question-{n}"),
                ])
                .stdout(std::process::Stdio::piped())
                .spawn()
                .unwrap()
        })
        .collect();
    for c in children {
        let out = c.wait_with_output().unwrap();
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stdout)
        );
    }
    assert_eq!(
        ok(r, "a", &["poll"])["messages"].as_array().unwrap().len(),
        3
    );
    assert_eq!(
        ok(r, "b", &["poll"])["messages"].as_array().unwrap().len(),
        3
    );
    assert_eq!(
        ok(r, "a", &["status"])["processed"]
            .as_array()
            .unwrap()
            .len(),
        6
    );
}
#[test]
fn opening_can_be_inspected_by_joiner_and_explicitly_ended() {
    let dir = tempfile::tempdir().unwrap();
    let r = dir.path();
    ok(r, "a", &["start", "task"]);
    assert_eq!(ok(r, "b", &["status"])["session"]["state"], "opening");
    ok(r, "a", &["close", "--reason", "peer unavailable"]);
    assert_eq!(ok(r, "a", &["status"])["session"]["state"], "abandoned");
}
#[test]
fn hard_links_to_protected_state_are_refused() {
    let dir = tempfile::tempdir().unwrap();
    let r = dir.path();
    pair(r);
    // The lock file is never replaced, so this alias survives projection recovery.
    std::fs::hard_link(r.join(".hacp/lock"), r.join("looks-like-output.txt")).unwrap();
    std::fs::write(
        r.join("terms.json"),
        r#"{"inputs":[],"outputs":["looks-like-output.txt"],"acceptance":["true"]}"#,
    )
    .unwrap();
    let out = call(r, "a", &["propose", "--terms", "terms.json"]);
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stdout).contains("protected .hacp"));
}

fn write_terms(r: &std::path::Path, value: &Value) {
    std::fs::write(
        r.join("requirements.json"),
        serde_json::to_vec(value).unwrap(),
    )
    .unwrap();
}
fn error(r: &std::path::Path, peer: &str, args: &[&str]) -> String {
    let out = call(r, peer, args);
    assert!(!out.status.success(), "unexpected success: {:?}", args);
    serde_json::from_slice::<Value>(&out.stdout).unwrap()["error"]
        .as_str()
        .unwrap()
        .to_string()
}

#[test]
fn requirements_bind_negotiation_freeze_amendment_submission_and_verification() {
    use hacp::v2::canon::digest_of;
    use serde_json::json;
    let dir = tempfile::tempdir().unwrap();
    let r = dir.path();
    pair(r);
    let mut terms = json!({
        "inputs":[], "outputs":["out.txt"], "acceptance":["test -s out.txt"],
        "requirements": {
            "retry":{"failed_tx_id_reserved":false},
            "normalization":{"id":"none"},
            "validation_order":["format","exists","capacity"]
        }
    });
    write_terms(r, &terms);
    let p = ok(r, "a", &["propose", "--terms", "requirements.json"]);
    let cid = p["contract"]["contract_id"].as_str().unwrap();
    assert_eq!(p["pending_digest"], digest_of(&terms).unwrap());
    assert_eq!(p["contract"]["state"], "proposed");
    assert_eq!(p["contract"]["revisions"], json!([]));
    assert!(
        error(
            r,
            "a",
            &["accept", cid, p["pending_digest"].as_str().unwrap()]
        )
        .contains("counterparty")
    );

    let original = terms.clone();
    terms["requirements"]["retry"]["failed_tx_id_reserved"] = json!(true);
    let changed_digest = digest_of(&terms).unwrap();
    assert_ne!(p["pending_digest"], changed_digest);
    assert!(error(r, "b", &["accept", cid, &changed_digest]).contains("stale terms digest"));
    // A counter changes only nested behavior, invalidating the original vote.
    write_terms(r, &terms);
    let counter = ok(r, "b", &["propose", cid, "--terms", "requirements.json"]);
    assert_eq!(
        counter["contract"]["agreed_by"],
        json!(["urn:hacp:agent:b"])
    );
    assert!(
        error(
            r,
            "a",
            &["accept", cid, p["pending_digest"].as_str().unwrap()]
        )
        .contains("stale terms digest")
    );
    let frozen = accept(r, "a", &counter);
    let first = frozen["contract"]["revisions"][0].clone();
    let rev1 = first["digest"].as_str().unwrap();
    assert_eq!(first["content"], terms);
    assert_eq!(
        rev1,
        digest_of(&json!({"contract_id":cid,"revision":1,"content":terms})).unwrap()
    );
    // Same contract and revision number isolate the effect of the nested change.
    assert_ne!(
        rev1,
        digest_of(&json!({"contract_id":cid,"revision":1,"content":original})).unwrap()
    );
    assert_eq!(frozen["contract"]["agreed_by"], json!([]));
    assert!(frozen["contract"]["agreed_terms_digest"].is_null());

    let question = ok(r, "a", &["ask", "Change the retry requirement?"]);
    ok(
        r,
        "b",
        &[
            "answer",
            question["message_id"].as_str().unwrap(),
            "Yes, change it.",
        ],
    );
    assert_eq!(
        ok(r, "a", &["status"])["contracts"][cid]["contract"]["revisions"],
        json!([first])
    );
    write_terms(r, &original);
    let amendment = ok(r, "a", &["propose", cid, "--terms", "requirements.json"]);
    assert_eq!(amendment["contract"]["state"], "amending");
    assert_eq!(amendment["contract"]["revisions"], json!([first]));
    assert!(error(r, "a", &["submit", cid, rev1]).contains("not executing"));
    assert!(
        error(
            r,
            "a",
            &["accept", cid, amendment["pending_digest"].as_str().unwrap()]
        )
        .contains("counterparty")
    );
    assert!(error(r, "b", &["accept", cid, &changed_digest]).contains("stale terms digest"));
    let amended = accept(r, "b", &amendment);
    assert_eq!(amended["contract"]["revisions"][0], first);
    assert_eq!(amended["contract"]["revisions"][1]["content"], original);
    let rev2 = amended["contract"]["revisions"][1]["digest"]
        .as_str()
        .unwrap();
    assert_ne!(rev1, rev2);
    assert_eq!(
        rev2,
        digest_of(&json!({"contract_id":cid,"revision":2,"content":original})).unwrap()
    );
    assert!(error(r, "a", &["submit", cid, rev1]).contains("stale revision"));
    std::fs::write(r.join("out.txt"), "result").unwrap();
    ok(r, "a", &["submit", cid, rev2]);
    let verified = ok(r, "b", &["verify", cid]);
    assert_eq!(verified["outcome"], "accept");
    assert_eq!(verified["verification"]["against_revision"], rev2);
    assert_eq!(ok(r, "a", &["complete"])["outcome"], "completed");
}

#[test]
fn completion_lists_all_blockers_and_close_can_terminate_unfinished_work() {
    let dir = tempfile::tempdir().unwrap();
    let r = dir.path();
    pair(r);
    assert!(error(r, "a", &["complete"]).contains("no contracts"));
    let frozen = accept(r, "b", &proposal(r, "a", &["out.txt"], &["true"]));
    let cid = frozen["contract"]["contract_id"].as_str().unwrap();
    let q = ok(r, "b", &["ask", "Are all edge cases specified?"]);
    let qid = q["message_id"].as_str().unwrap();
    ok(r, "a", &["poll"]); // Fetching does not resolve a blocking question.
    let before = ok(r, "a", &["status"]);
    let err = error(r, "a", &["complete"]);
    assert!(err.contains(&format!("contract {cid} is executing")));
    assert!(err.contains(&format!(
        "contract {cid} has no accepted counterparty verification"
    )));
    assert!(err.contains(&format!("question {qid} remains unanswered")));
    assert_eq!(ok(r, "a", &["status"]), before);
    ok(
        r,
        "a",
        &[
            "close",
            "--reason",
            "success claimed in text is not completion",
        ],
    );
    let s = ok(r, "b", &["status"]);
    assert_eq!(s["session"]["state"], "closed");
    assert_eq!(s["outcome"], "terminated");
    assert_eq!(s["contracts"][cid]["contract"]["state"], "executing");
    assert_eq!(ok(r, "b", &["poll"])["outcome"], "terminated");
    assert!(error(r, "a", &["complete"]).contains("session must be active"));
    assert!(
        std::fs::read_to_string(r.join(".hacp/log.md"))
            .unwrap()
            .contains("terminated")
    );
}

#[test]
fn completion_waits_for_every_contract_and_question_then_persists_success() {
    let dir = tempfile::tempdir().unwrap();
    let r = dir.path();
    pair(r);
    let (cid, _) = submitted(r, &["test -s out.txt"]);
    let second = accept(
        r,
        "a",
        &proposal(r, "b", &["tests.txt"], &["test -s tests.txt"]),
    );
    let bid = second["contract"]["contract_id"].as_str().unwrap();
    let brev = second["contract"]["revisions"][0]["digest"]
        .as_str()
        .unwrap();
    let q = ok(r, "a", &["ask", "Any remaining concerns?"]);
    ok(r, "b", &["verify", &cid]);
    assert!(error(r, "a", &["complete"]).contains(&format!("contract {bid} is executing")));
    std::fs::write(r.join("tests.txt"), "tests").unwrap();
    ok(r, "b", &["submit", bid, brev]);
    assert!(error(r, "b", &["complete"]).contains(&format!("contract {bid} is verifying")));
    ok(r, "a", &["verify", bid]);
    let err = error(r, "b", &["complete"]);
    assert!(err.contains(q["message_id"].as_str().unwrap()));
    assert!(!err.contains("contract "));
    ok(
        r,
        "b",
        &["answer", q["message_id"].as_str().unwrap(), "None."],
    );
    assert_eq!(ok(r, "b", &["complete"])["outcome"], "completed");
    std::fs::remove_file(r.join(".hacp/log.md")).unwrap();
    let s = ok(r, "a", &["status"]);
    assert_eq!(s["session"]["state"], "closed");
    assert_eq!(s["outcome"], "completed");
    assert_eq!(
        s["events"].as_array().unwrap().last().unwrap()["action"],
        "complete"
    );
    assert_eq!(
        s["messages"].as_array().unwrap().last().unwrap()["body"]["outcome"],
        "completed"
    );
    assert!(
        std::fs::read_to_string(r.join(".hacp/log.md"))
            .unwrap()
            .contains("completed")
    );
    assert!(
        !call(r, "a", &["close", "--reason", "overwrite success"])
            .status
            .success()
    );
    assert_eq!(ok(r, "a", &["status"])["outcome"], "completed");
}

#[test]
fn completion_requires_matching_verification_even_when_snapshot_says_settled() {
    use serde_json::json;
    let dir = tempfile::tempdir().unwrap();
    let r = dir.path();
    pair(r);
    let (cid, _) = submitted(r, &["true"]);
    ok(r, "b", &["verify", &cid]);
    let valid = ok(r, "a", &["status"]);
    // Trusted legacy/host state may have used the core's low-level decide API.
    // Completion must independently require the binding's verification record.
    for mode in [
        "missing",
        "self",
        "outsider",
        "revision",
        "contract",
        "artifacts",
        "rework",
        "submission",
    ] {
        let mut s = valid.clone();
        let e = &mut s["contracts"][&cid];
        match mode {
            "missing" => e["verifications"] = json!([]),
            "self" => e["verifications"][0]["verifier"] = json!("urn:hacp:agent:a"),
            "outsider" => e["verifications"][0]["verifier"] = json!("urn:hacp:agent:outsider"),
            "revision" => e["verifications"][0]["against_revision"] = json!("0".repeat(64)),
            "contract" => e["verifications"][0]["contract_id"] = json!("c-other"),
            "artifacts" => e["verifications"][0]["artifacts"] = json!([]),
            "rework" => e["verifications"][0]["verdict"] = json!({"rework":{"scope":"fix"}}),
            "submission" => e["submissions"] = json!([]),
            _ => unreachable!(),
        }
        std::fs::write(
            r.join(".hacp/session.json"),
            serde_json::to_vec(&s).unwrap(),
        )
        .unwrap();
        assert!(
            error(r, "a", &["complete"]).contains("no accepted counterparty verification"),
            "{mode}"
        );
    }
}

#[test]
fn legacy_snapshot_without_outcome_is_readable_and_not_inferred_successful() {
    let dir = tempfile::tempdir().unwrap();
    let r = dir.path();
    pair(r);
    let (cid, _) = submitted(r, &["true"]);
    ok(r, "b", &["verify", &cid]);
    let mut legacy = ok(r, "a", &["status"]);
    legacy.as_object_mut().unwrap().remove("outcome");
    std::fs::write(
        r.join(".hacp/session.json"),
        serde_json::to_vec(&legacy).unwrap(),
    )
    .unwrap();
    assert_eq!(ok(r, "a", &["complete"])["outcome"], "completed");
    let mut legacy = ok(r, "a", &["status"]);
    legacy.as_object_mut().unwrap().remove("outcome");
    std::fs::write(
        r.join(".hacp/session.json"),
        serde_json::to_vec(&legacy).unwrap(),
    )
    .unwrap();
    assert!(ok(r, "a", &["status"])["outcome"].is_null());
    assert!(error(r, "a", &["complete"]).contains("session must be active"));
}

#[test]
fn completion_commit_recovers_outcome_and_notification_atomically() {
    for point in ["before_commit", "after_commit", "after_delivery"] {
        let dir = tempfile::tempdir().unwrap();
        let r = dir.path();
        pair(r);
        let (cid, _) = submitted(r, &["true"]);
        ok(r, "b", &["verify", &cid]);
        let out = Command::new(env!("CARGO_BIN_EXE_hacp"))
            .arg("--project")
            .arg(r)
            .args(["--peer", "a", "complete"])
            .env("HACP_TEST_CRASH", point)
            .output()
            .unwrap();
        assert_eq!(out.status.code(), Some(86));
        let s = ok(r, "b", &["status"]);
        if point == "before_commit" {
            assert_eq!(s["session"]["state"], "active");
            assert!(s["outcome"].is_null());
        } else {
            assert_eq!(s["session"]["state"], "closed");
            assert_eq!(s["outcome"], "completed");
        }
        assert_eq!(
            s["events"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|e| e["action"] == "complete")
                .count(),
            usize::from(point != "before_commit")
        );
    }
}
