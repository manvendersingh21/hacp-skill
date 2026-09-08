# Task Pocket: Codex, Claude Code, and OpenCode

On 2026-09-07, each of the three requested pairs built the same dependency-free Python task CLI in its own fresh scratch repository. Runs were sequential to limit account load. Each CLI received one initial brief with its native HACP skill invocation; all subsequent peer coordination used `hacp`. The harness observed the sessions and did not act as a peer or edit their implementation files.

**All three pairings passed the complete project and protocol checks.** Each project supports adding, listing, and completing tasks with JSON output, persistent storage, input validation, and atomic database replacement. This is separate from the earlier greeting-function demos and the [Codex–AGY project run](PROJECT-E2E.md).

| Pair (starter–joiner) | Outcome | Elapsed | Peer + independent tests, per platform | Answered questions |
| --- | --- | --- | --- | --- |
| Codex–Claude Code | Passed | 279.21 s | 20 + 12 | 4 |
| Codex–OpenCode | Passed | 460.22 s | 15 + 12 | 6 |
| Claude Code–OpenCode | Passed | 315.73 s | 16 + 12 | 1 |

Both test suites passed on **macOS and Linux** for every project. The Linux rerun mounted the completed project read-only in the official Rust image. Each run also passed a real six-command CLI walkthrough.

## What was verified

All 12 checks in each run's protocol audit passed: both processes exited successfully; both contracts settled; the session closed; all questions were answered; a question/answer crossed peer identities; verification came from the counterparty; all four output files were first observed after both contracts froze; the expected four files were observed; immutable artifact hashes, sizes, and revisions matched (and latest submissions matched working files); the initial spec was unchanged; the peer suite passed with nonzero tests; and all 12 independent black-box tests passed.

The independent suite exercises fresh CLI processes, trimming, Unicode, missing databases and parent directories, sorted open/all listings, persistence, increasing IDs, idempotent completion, invalid titles and IDs, and malformed JSON/database shapes without mutation. The observer checks files every 100 ms; its timeline records when files were first seen.

Codex and Claude used their installed default models. At the user's request, both successful OpenCode runs used the per-process override `zai-coding-plan/glm-5.3-flash`; personal settings were unchanged. Exact CLI versions, observed/configured model identifiers, overrides, and the released HACP binary SHA-256 are in each run's environment record. Published transcripts retain commands and output while omitting reasoning and unrelated host initialization. Original stream hashes are preserved in `transcripts.json`; originals remain in the local archive.

## Codex–Claude Code

Peer a (Codex) owned `task_store.py`; peer b (Claude Code) owned `task_cli.py`, `tests/test_tasks.py`, and `README.md`. Both CLI processes exited 0. Both contracts settled and the session closed. 4 questions received 4 answers. The record contains one counterproposal and no measured rework verdicts.

- [Run result](evidence/project-codex-claude-20260907-182214/result.json), [versions and models](evidence/project-codex-claude-20260907-182214/environment.json), and [protocol audit](evidence/project-codex-claude-20260907-182214/audit.json)
- [Readable HACP log](evidence/project-codex-claude-20260907-182214/state/log.md), [authoritative state](evidence/project-codex-claude-20260907-182214/state/session.json), and [observed timeline](evidence/project-codex-claude-20260907-182214/observations.json)
- [Peer tests](evidence/project-codex-claude-20260907-182214/peer-tests.txt), [independent tests](evidence/project-codex-claude-20260907-182214/independent-tests.txt), [Linux tests](evidence/project-codex-claude-20260907-182214/linux-tests.txt), and [CLI walkthrough](evidence/project-codex-claude-20260907-182214/cli-walkthrough.txt)
- [Runnable project and README](evidence/project-codex-claude-20260907-182214/project/README.md), [storage](evidence/project-codex-claude-20260907-182214/project/task_store.py), [CLI](evidence/project-codex-claude-20260907-182214/project/task_cli.py), and [tests](evidence/project-codex-claude-20260907-182214/project/tests/test_tasks.py)
- [Peer a transcript](evidence/project-codex-claude-20260907-182214/a.jsonl) and [peer b transcript](evidence/project-codex-claude-20260907-182214/b.jsonl)

The original scratch project, including `.hacp/log.md`, remains at `/Users/manubaba/Documents/hacp-task-codex-claude-20260907-182214`.

## Codex–OpenCode

Peer a (Codex) owned `task_store.py`; peer b (OpenCode) owned `task_cli.py`, `tests/test_tasks.py`, and `README.md`. Both CLI processes exited 0. Both contracts settled and the session closed. 6 questions received 6 answers. The record contains one counterproposal and one measured rework verdict.

- [Run result](evidence/project-codex-opencode-flash-20260907-183710/result.json), [versions and models](evidence/project-codex-opencode-flash-20260907-183710/environment.json), and [protocol audit](evidence/project-codex-opencode-flash-20260907-183710/audit.json)
- [Readable HACP log](evidence/project-codex-opencode-flash-20260907-183710/state/log.md), [authoritative state](evidence/project-codex-opencode-flash-20260907-183710/state/session.json), and [observed timeline](evidence/project-codex-opencode-flash-20260907-183710/observations.json)
- [Peer tests](evidence/project-codex-opencode-flash-20260907-183710/peer-tests.txt), [independent tests](evidence/project-codex-opencode-flash-20260907-183710/independent-tests.txt), [Linux tests](evidence/project-codex-opencode-flash-20260907-183710/linux-tests.txt), and [CLI walkthrough](evidence/project-codex-opencode-flash-20260907-183710/cli-walkthrough.txt)
- [Runnable project and README](evidence/project-codex-opencode-flash-20260907-183710/project/README.md), [storage](evidence/project-codex-opencode-flash-20260907-183710/project/task_store.py), [CLI](evidence/project-codex-opencode-flash-20260907-183710/project/task_cli.py), and [tests](evidence/project-codex-opencode-flash-20260907-183710/project/tests/test_tasks.py)
- [Peer a transcript](evidence/project-codex-opencode-flash-20260907-183710/a.jsonl) and [peer b transcript](evidence/project-codex-opencode-flash-20260907-183710/b.jsonl)

The original scratch project, including `.hacp/log.md`, remains at `/Users/manubaba/Documents/hacp-task-codex-opencode-flash-20260907-183710`.

## Claude Code–OpenCode

Peer a (Claude Code) owned `task_store.py`; peer b (OpenCode) owned `task_cli.py`, `tests/test_tasks.py`, and `README.md`. Both CLI processes exited 0. Both contracts settled and the session closed. One question received one answer. The record contains no counterproposals or measured rework verdicts.

- [Run result](evidence/project-claude-opencode-flash-20260907-183710/result.json), [versions and models](evidence/project-claude-opencode-flash-20260907-183710/environment.json), and [protocol audit](evidence/project-claude-opencode-flash-20260907-183710/audit.json)
- [Readable HACP log](evidence/project-claude-opencode-flash-20260907-183710/state/log.md), [authoritative state](evidence/project-claude-opencode-flash-20260907-183710/state/session.json), and [observed timeline](evidence/project-claude-opencode-flash-20260907-183710/observations.json)
- [Peer tests](evidence/project-claude-opencode-flash-20260907-183710/peer-tests.txt), [independent tests](evidence/project-claude-opencode-flash-20260907-183710/independent-tests.txt), [Linux tests](evidence/project-claude-opencode-flash-20260907-183710/linux-tests.txt), and [CLI walkthrough](evidence/project-claude-opencode-flash-20260907-183710/cli-walkthrough.txt)
- [Runnable project and README](evidence/project-claude-opencode-flash-20260907-183710/project/README.md), [storage](evidence/project-claude-opencode-flash-20260907-183710/project/task_store.py), [CLI](evidence/project-claude-opencode-flash-20260907-183710/project/task_cli.py), and [tests](evidence/project-claude-opencode-flash-20260907-183710/project/tests/test_tasks.py)
- [Peer a transcript](evidence/project-claude-opencode-flash-20260907-183710/a.jsonl) and [peer b transcript](evidence/project-claude-opencode-flash-20260907-183710/b.jsonl)

The original scratch project, including `.hacp/log.md`, remains at `/Users/manubaba/Documents/hacp-task-claude-opencode-flash-20260907-183710`.

## Earlier quota failures

The initial OpenCode attempts used the installed default `alibaba/qwen3.8-flash` and received HTTP 403: free quota exhausted. They are preserved as failed attempts, separate from the successful lighter-model retries:

- [Codex–OpenCode failure](evidence/project-codex-opencode-20260907-182214/result.json) and [log](evidence/project-codex-opencode-20260907-182214/state/log.md): the storage contract settled, but OpenCode changed its README/tests after submission. HACP measured the hash mismatch and correctly required rework. The provider failure prevented resubmission. Although 21 peer tests and 12 independent tests passed, the unfinished contract and mismatching latest submission make the run a failure. The waiting Codex process was stopped; the active session remains preserved.
- [Claude–OpenCode failure](evidence/project-claude-opencode-20260907-182214/result.json) and [log](evidence/project-claude-opencode-20260907-182214/state/log.md): OpenCode failed at startup before joining. The waiting Claude process was stopped and the opening session preserved. No implementation was completed.

The first failed attempt also exposed an evidence-copy race when a transient inbox temporary file disappeared after cancellation. Its snapshot was recovered after the processes stopped, without changing HACP state. The harness now excludes transient `.tmp-*` projection files when copying evidence, and exports OpenCode model observations through a regular temporary file to avoid truncated pipe output.

## Reproduce a live pair

From the `hacp-skill` checkout, with the selected CLIs authenticated and the personal HACP skills installed:

```sh
python3 scripts/project_e2e.py --run-live --pair codex-claude \
  --project /tmp/task-pocket-codex-claude \
  --out /tmp/task-pocket-codex-claude-evidence
python3 scripts/audit_project_run.py /tmp/task-pocket-codex-claude-evidence
```

For `codex-opencode` or `claude-opencode`, also pass `--opencode-model zai-coding-plan/glm-5.3-flash` to reproduce the successful model choice. Use fresh nonexistent project/evidence paths. These runs make real model calls and authorize the agents to edit their scratch project and run local acceptance commands. No host configuration is changed.

To exercise a preserved project without model calls, enter its linked `project` directory and run:

```sh
python3 task_cli.py --db /tmp/task-pocket-example.json add "Try HACP"
python3 task_cli.py --db /tmp/task-pocket-example.json list
python3 task_cli.py --db /tmp/task-pocket-example.json done 1
python3 -m unittest discover -s tests -v
```
