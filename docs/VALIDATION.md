# v0.1 validation — 2026-09-07

The package was built independently from HIVE. No HIVE run is used as evidence.

## Deterministic checks

Formatting, Clippy with warnings denied, all 19 integration tests, and separate skill structural validation passed on macOS and Linux (Rust 1.98.0). Linux was tested in the official Rust container; GitHub Actions also runs macOS and Ubuntu; [latest release checks](https://github.com/manvendersingh21/hacp-skill/actions/workflows/ci.yml) include the final protected-hard-link regression. See [macOS output](evidence/macos-checks.txt) and [Linux output](evidence/linux-checks.txt).

Tests cover session recovery, explicit closure, questions/answers, crossed questions, concurrent writers, duplicate and forged envelopes, fetched message recovery, wait timeouts, metadata lock timeout/release, crashes before commit/after commit/after delivery, initial ownership and filesystem aliases (including hard links to protected state), simultaneous conflicting freezes, counters, stale digests, amendment collisions, negotiation/amendment bounds, immutable submissions, self-verification refusal, failed checks followed by repair, missing/changed artifacts, command-group timeouts, interrupted verification, and conflict-safe idempotent installation.

```sh
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
python3 scripts/validate_skill.py
```

The supplied skill-creator validator also passed with PyYAML in an isolated `uv run --with pyyaml` environment. This checks structure separately from actual behavior.

## Discovery

| CLI | Version | Evidence |
| --- | --- | --- |
| Claude Code | 2.1.263 | [Clean native `/hacp` probe](evidence/discovery/claude-clean.stdout) |
| Codex | 0.153.4 | [Native `$hacp` probe](evidence/discovery/codex.stdout) |
| AGY | 1.1.27 | [Native `/hacp` retry](evidence/discovery/agy-retry.stdout) |
| OpenCode | 1.18.29 | [Actual `debug skill` discovery](evidence/discovery/opencode.stdout); native `/hacp` command exercised in live runs |

AGY's first headless discovery attempt could not prompt for command permission. A retry with per-process approval enabled succeeded. The original Claude probe inherited extra stdin from the driver; the clean probe uses `/dev/null` and confirms discovery independently. The installer did not change any permissions or configuration.

Paths follow [Codex's documented local discovery](https://learn.chatgpt.com/docs/build-skills), [Claude's personal skills](https://code.claude.com/docs/en/skills), [OpenCode's skill paths](https://opencode.ai/docs/skills) and [command paths](https://opencode.ai/docs/commands). AGY uses the installed 1.1.27 bundled customization guide.

## Actual live pairings

Each agent received one initial task with its native skill invocation. Every subsequent question, proposal, acceptance, submission, and verification went through `hacp`; the harness did not supply additional coordination turns. The task was a Python greeting function and a separate unittest file, owned by different peers. Each successful pairing produced two frozen contracts, two submissions, counterparty verification, passing independently rerun tests, and explicit session closure.

| Starter a | Joiner b | Result | Elapsed |
| --- | --- | --- | --- |
| Claude Code | Codex | Both contracts settled | 132 s |
| AGY | Claude Code | Both contracts settled | 152 s |
| Claude Code | OpenCode | Both contracts settled | 241 s |
| Codex | AGY | Both contracts settled on user-requested retry | 155 s |
| OpenCode | Codex | Both contracts settled | 134 s |
| AGY | OpenCode | Both contracts settled | 183 s |

All four CLIs completed collaborations in both starter and joiner roles. **All six distinct pairings now have a successful live run.** The user-requested Codex–AGY retry completed in 155.46 seconds with the same configured models: both agents exited 0, both contracts settled through counterparty verification, the session closed, and all three independently rerun tests passed. This run used the released v0.1.0 binary at `37fdf2d`.

The two earlier Codex–AGY attempts remain recorded as failures. AGY's default `Gemini 3.6 Flash (Medium)` returned HTTP 503/no capacity. In the first attempt it exited after joining; Codex waited, reported the unresolved proposal, and explicitly closed the session. In one fresh retry both contracts froze, but AGY again exited with a high-traffic error. The waiting Codex process was then stopped; the retry remains active with unresolved work and no fabricated consent.

The failed attempts' independent `unittest` command exited 0 while finding no tests. That exit status is not treated as evidence of completed work. Settlement and actual artifacts are required for a pass.

- [Successful Codex–AGY retry](evidence/live-user-retry-20260907-173336/results.json), [its final log](evidence/live-user-retry-20260907-173336/codex-agy/state/log.md), and [model/binary fingerprint](evidence/live-user-retry-20260907-173336/environment.json)
- [Initial six-pair results](evidence/live-2026-09-07/results.json)
- [Codex–AGY retry result](evidence/live-retry-2026-09-07/results.json) and [why its companion was stopped](evidence/live-retry-2026-09-07/codex-agy/cancellation.json)
- [Versions, models, and live binary fingerprint](evidence/models.json)
- [Example final log](evidence/live-2026-09-07/claude-codex/state/log.md)

Models: Claude Sonnet 5, Codex's configured GPT-6 Astra, AGY's configured Gemini 3.6 Flash (Medium), and OpenCode's observed GLM-5.3 via `zai-coding-plan`. No model defaults were changed. The original seven attempts used the M5 binary at milestone `3ac45ff`. The successful user-requested Codex–AGY retry used the released v0.1.0 binary at `37fdf2d`, including the later path-alias, atomic artifact publication, prerequisite-reporting, and opening-status improvements. Its exact binary hash is recorded with that run.

Each pairing directory retains initial briefs, CLI command/output transcripts, the authoritative snapshot, its final readable log, frozen contracts, artifact copies, measurements, independent test output, and live event timestamps. Published transcripts omit reasoning and unrelated host initialization; their original stream hashes are in `transcripts.json`. Original streams are retained locally outside the public repository.

To reproduce (this makes real model calls and permits scratch-project edits and acceptance commands):

```sh
python3 scripts/live.py --run-live --out /tmp/hacp-live-results
# Or a single pairing:
python3 scripts/live.py --run-live --pair claude-codex --out /tmp/hacp-one-pair
```

The harness uses each CLI's installed default model and per-process unattended permissions. It never alters personal host settings. It allows two collaborations at a time and bounds each run. Account access and model service capacity remain external prerequisites.

## Complete small-project test

A separate Codex–AGY run built a usable JSON-backed task CLI in 221 seconds. Both contracts settled, all five questions were answered, all four artifact hashes matched, and the 16 peer tests plus 12 independent CLI tests passed on macOS and Linux. See the [full small-project test report](PROJECT-E2E.md) for the runnable project and evidence.

Three further full-project runs—Codex–Claude Code, Codex–OpenCode, and Claude Code–OpenCode—also passed. Each pair froze both contracts before observed implementation, settled through cross-verification, answered all questions, and passed its peer suite plus 12 independent CLI tests on macOS and Linux. See the [three-pair project report](PROJECT-MATRIX.md) for timings, runnable projects, models, and complete logs.

## Recording

[Open the standalone player](demo.html) locally in a browser, or download [the asciicast](demo.cast). It contains 115.54 seconds of actual live event-monitor capture at clearly labeled **2× playback**, followed by the real final log, for **74.77 seconds total**. [Unaccelerated event timestamps](evidence/live-2026-09-07/claude-codex/events.cast) and CLI transcripts are retained. It is a live lifecycle monitor recording, not a scripted imitation of agent output.

## Publication

The [public repository](https://github.com/manvendersingh21/hacp-skill) was pushed and its remote commit verified. [GitHub CI](https://github.com/manvendersingh21/hacp-skill/actions/runs/34173129182) passed on macOS and Ubuntu at `de8649464d9cae0827aae87e77c78e478e609c79`. A clean clone and isolated `cargo install --git https://github.com/manvendersingh21/hacp-skill --locked` succeeded, followed by idempotent installation for all four hosts and a fresh session/question/answer smoke test. See the [installation report](evidence/github-install.json), [command output](evidence/github-install.txt), and [CI job results](evidence/github-ci.json). The package name was rechecked before the first commit using `cargo search hacp-skill`; no result was returned. The direct crates.io HTTP lookup returned 403, so registry availability is not asserted beyond that search. This release is distributed from GitHub; no crates.io publication was performed.
