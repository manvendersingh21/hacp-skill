# Codex–AGY: complete small-project test

On 2026-09-07, Codex and AGY built **Task Pocket**, a dependency-free Python CLI backed by a JSON file. A fresh scratch repository was used. This is a new small-project collaboration, separate from the greeting-function pairing tests.

**Result: passed.** Both agents exited 0 in 220.88 seconds. Both HACP contracts settled through counterparty verification, and the session closed. No capacity error occurred.

## Project and ownership

| Peer | CLI / configured model | Frozen outputs |
| --- | --- | --- |
| a | Codex 0.153.4 / GPT-6 Astra | `task_store.py` |
| b | AGY 1.1.27 / Gemini 3.6 Flash (Medium) | `task_cli.py`, `tests/test_tasks.py`, `README.md` |

The CLI adds tasks, lists open/all tasks, and completes tasks. It persists across fresh processes, uses increasing IDs and atomic replacement, validates input, and preserves corrupted databases without overwriting them. Concurrent database writers are outside the brief's scope.

Each agent received one initial brief and its native skill invocation (`$hacp` or `/hacp`). Subsequent coordination used HACP. There were five question/answer round trips and 1 counterproposal(s). The independent monitor first observed all four output files after both contracts froze. Every preserved artifact and current output matched the submitted hash and size. All questions were answered before closure.

## Checks performed

- **16 agent-authored tests passed**, including storage/CLI integration, persistence, ID rules, idempotent completion, validation, malformed database preservation, and atomic-write failure cleanup.
- **12 independent black-box tests passed**, written outside the peer-owned outputs and executed through fresh CLI processes. They cover trimming, Unicode/quotes, missing databases, missing parent directories, filtering, persistence, IDs, completion, blank titles, malformed/unknown IDs, malformed JSON, and invalid database shapes across commands.
- **Both suites passed on macOS and Linux**: 28 tests per platform. Linux used a read-only project mount in the official Rust image's Python environment.
- **Nine protocol/evidence checks passed**: settlement, closure, answered questions, a bilateral question round trip, verifier identity, both agreements preceding observed outputs, observation of all four outputs, artifact hashes/sizes, and the independent suite result.
- A real CLI walkthrough added two tasks, completed one, and checked both open/all listings.

HACP's separate 19-test deterministic suite covers its stale digests, duplicate messages, ownership races, lock timeouts, interrupted verification, crash recovery, and rework behavior. That suite's results remain in [the main validation report](VALIDATION.md).

## Evidence and runnable files

- [Run result](evidence/project-e2e-20260907-180719/result.json) and [versions/model/binary fingerprint](evidence/project-e2e-20260907-180719/environment.json)
- [Protocol audit](evidence/project-e2e-20260907-180719/audit.json) and [observed event/file timeline](evidence/project-e2e-20260907-180719/observations.json)
- [Readable HACP log](evidence/project-e2e-20260907-180719/state/log.md) and [authoritative snapshot](evidence/project-e2e-20260907-180719/state/session.json)
- [Agent-authored test output](evidence/project-e2e-20260907-180719/peer-tests.txt), [independent test output](evidence/project-e2e-20260907-180719/independent-tests.txt), and [Linux results](evidence/project-e2e-20260907-180719/linux-tests.txt)
- [Real CLI walkthrough](evidence/project-e2e-20260907-180719/cli-walkthrough.txt)
- [Project README](evidence/project-e2e-20260907-180719/project/README.md), [storage](evidence/project-e2e-20260907-180719/project/task_store.py), [CLI](evidence/project-e2e-20260907-180719/project/task_cli.py), and [tests](evidence/project-e2e-20260907-180719/project/tests/test_tasks.py)
- [Codex command transcript](evidence/project-e2e-20260907-180719/a.jsonl) and [AGY command transcript](evidence/project-e2e-20260907-180719/b.jsonl)

Published transcripts omit reasoning and unrelated host initialization; original stream hashes are preserved in the run's `transcripts.json`. Original streams remain in the local evidence archive.

## Reproduce

Run this from the `hacp-skill` checkout with authenticated Codex and AGY and the `hacp` skill installed. Use new, nonexistent paths for the project and evidence directory:

```sh
python3 scripts/project_e2e.py --run-live \
  --project /tmp/task-pocket-project \
  --out /tmp/task-pocket-evidence
python3 scripts/audit_task_project.py /tmp/task-pocket-project
```

To try the preserved project from this repository:

```sh
cd docs/evidence/project-e2e-20260907-180719/project
python3 task_cli.py --db /tmp/task-pocket-demo.json add "Try HACP"
python3 task_cli.py --db /tmp/task-pocket-demo.json list
python3 task_cli.py --db /tmp/task-pocket-demo.json done 1
python3 -m unittest discover -s tests -v
```

The original local project remains at `/Users/manubaba/Documents/hacp-task-demo-20260907-180719`, including its `.hacp/log.md`.
