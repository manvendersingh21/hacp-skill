# hacp-skill

Two coding agents can work in the same repo. Getting them to agree on interfaces, avoid editing each other's files, and check each other's work usually leaves you copying messages between terminals.

`hacp-skill` gives two agents a shared conversation and explicit contracts. They agree on files and acceptance commands before implementing, ask each other questions, and verify each other's submissions. You can read what happened in `.hacp/log.md`.

One small Rust binary, one shared skill, exactly two peers. No coordinator service or model API integration.

## Install

Requires Rust/Cargo, Git, macOS or Linux, and two authenticated supported agent CLIs on the same machine. Start in a **scratch repository**: agents act on peer instructions, and agreed acceptance commands run locally. **This is not a sandbox.**

```sh
cargo install --git https://github.com/manvendersingh21/hacp-skill --locked
hacp install --cli all
```

Restart the CLIs after installation. `hacp install` without `--cli` selects all detected supported CLIs. Choose one with `--cli claude|codex|agy|opencode`; `all` installs for all four. Identical files are an idempotent success. Conflicting destinations cause refusal before any installation. Your existing configuration and permission settings are preserved.

`hacp doctor --cli all` reports versions, discovery paths, conflicts, and prerequisites without calling a model. It does not establish that your accounts are authenticated or your host permissions allow a collaboration.

| Client | Native invocation | Personal discovery path |
| --- | --- | --- |
| Claude Code | `/hacp` | `~/.claude/skills/hacp` |
| Codex | `$hacp` | `~/.agents/skills/hacp` |
| AGY | `/hacp` | `~/.gemini/config/skills/hacp` |
| OpenCode | `/hacp` | Reuses an identical discoverable skill, otherwise `~/.config/opencode/skills/hacp`; wrapper in `~/.config/opencode/commands/hacp.md` |

**Live validation:** all six distinct CLI pairings have completed successfully. Codex–AGY passed on a later retry with the same models after two earlier AGY capacity failures. Full outcomes, including those failures, are linked below.

OpenCode honors `XDG_CONFIG_HOME`. AGY's path follows the bundled guide in version 1.1.27. See [discovery and validation evidence](docs/VALIDATION.md) for tested versions and actual outcomes. Prebuilt binaries and `install.sh` are deferred to v0.2.

## Try two agents

Create a scratch repo and open both CLIs there:

```sh
mkdir hacp-demo
cd hacp-demo
git init
```

In Claude Code, AGY, or OpenCode, give the first agent:

```text
/hacp You are peer a. Implement greet.py with greet(name) returning Hello, NAME!
after trimming whitespace; raise ValueError for an empty name. Own greet.py.
Agree with peer b before coding. Stay available until both tasks are verified.
```

In the second terminal, give Codex:

```text
$hacp You are peer b. Write test_greet.py using Python unittest to test greet.greet:
normal names, whitespace trimming, and empty-name rejection. Own test_greet.py.
Agree with peer a before coding. Stay available until both tasks are verified.
```

Use `$hacp` in either position when the client is Codex; use `/hacp` for the others. The agents negotiate through `hacp`, implement their agreed outputs, and cross-verify. The users of both CLIs need to allow their local file edits and acceptance commands through each host's normal permission flow.

Follow along with `hacp --peer a status`, or read `.hacp/log.md`. A CLI restart does not reset the collaboration: invoke the skill with the same peer identity and project to resume. The binary never stores a shared “current peer.” Existing sessions, including closed sessions, are preserved; use a new project directory for another session in v0.1.

## Commands

Every session command requires `--peer a|b` (or `HACP_PEER`). Put `--project PATH` anywhere to target a project from another directory, and use `--json` for machine-readable output. Agents should pass their remembered peer explicitly on every command.

| Command | Purpose |
| --- | --- |
| `start "task" --owns file ...` | Peer a opens a session with proposed file ownership. |
| `join "task" --owns file ...` | Peer b joins; the session becomes active. |
| `status` | Inspect persisted state, contracts, history, and both registrations. |
| `ask "text"` | Send a question; returns its message ID. |
| `answer MESSAGE_ID "text"` | Answer the other peer's question using HACP `in_reply_to`. |
| `poll [--all]` | Fetch your new messages and inspect outstanding questions and contract actions. `--all` recovers fetched messages. |
| `wait [--timeout SECONDS]` | Check immediately, then every 250 ms; default 180 seconds. |
| `propose [CONTRACT_ID] --terms FILE` | Propose your task's contract, counter, or propose an amendment. |
| `accept CONTRACT_ID PENDING_DIGEST` | Explicitly accept identical current terms and freeze a revision. |
| `decline CONTRACT_ID PENDING_DIGEST` | Withdraw a proposal or decline an amendment, preserving its previous revision. |
| `submit CONTRACT_ID REVISION_DIGEST [--claim TEXT]` | Owner submits the full frozen output set with immutable artifact copies. |
| `verify CONTRACT_ID [--timeout SECONDS]` | Counterparty runs frozen acceptance commands; default 300 seconds each. |
| `close --reason TEXT` | Explicitly end a session. An unjoined opening is recorded as abandoned. |

A terms file is JSON:

```json
{
  "inputs": ["test_greet.py"],
  "outputs": ["greet.py"],
  "acceptance": ["python3 -m unittest -v"]
}
```

Paths must name concrete project-relative files, even if not created yet. Directories, globs, escapes, `.hacp` paths, and overlapping output aliases are refused. Initial declarations reserve files until that peer first freezes a contract. Afterward, active frozen revisions determine ownership. Terms files are coordination material; each peer should use a distinct filename.

Proposing terms records the proposer's acceptance through HACP. The counterparty reviews them and copies the current pending digest from `poll` into `accept`. Stale acceptance and submission digests are refused. A changed interface or output set requires an amendment. Prior frozen claims remain in force throughout amendment negotiation; both acceptances replace them atomically.

Negotiation permits three counter rounds, and three accepted amendments. Reaching HACP's bounds produces terminal `noagreement`; silence never counts as acceptance. Rework and no agreement return exit status 0 with an explicit outcome. Refusals, invalid usage, operational errors, and wait timeouts return nonzero. Always inspect the outcome.

Acceptance commands run sequentially from the project root using `/bin/sh -c`. Verification captures stdout, stderr, elapsed time, exit code, signal, and timeout. It checks both the working files and preserved artifact copies before and after execution, then applies a measured record exclusively through `Contract::apply_verification`. Failed commands or changed/missing artifacts request rework. A timed-out command's process group is killed.

## Recovery and scope

`.hacp/session.json` is authoritative: it contains HACP state, deduplication IDs, messages, projection intents, artifact metadata, and measured verification records. `.hacp/inbox/a`, `.hacp/inbox/b`, `.hacp/contracts`, and `.hacp/log.md` are recoverable projections. Bytes live in `.hacp/artifacts`; command capture files live in `.hacp/verification`.

Metadata updates use one process-released advisory lock with a three-second acquisition timeout. Snapshots use flushed temporary files next to the destination and atomic renames. Recovery republishes committed records under their original IDs without repeating transitions or log entries. The lock is released before hashing, running commands, or waiting for a peer.

If a verifier dies, the next inspection marks the attempt `interrupted`. Inspect its capture files and explicitly run `verify CONTRACT_ID --retry-interrupted` to rerun; recovery never executes shell commands. A crash can leave an unreferenced immutable copy or a temporary file; neither becomes a submission automatically.

This is cooperative coordination on one local filesystem. File claims do not intercept editor writes, peer flags are not authentication, and a process with project access can alter state. Arbitrary acceptance commands have the host user's permissions. Output is safely escaped in the log; raw command captures remain available. No broker, daemon, socket, hook, steering, tmux, remote machine, MCP, or extra peer is part of this product.

## Develop and validate

```sh
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
python3 scripts/validate_skill.py
```

Deterministic tests require no model accounts. [Validation](docs/VALIDATION.md) separates those results from opt-in live collaborations, provides reproduction commands, and links the terminal recording. See [the protocol binding](docs/PROTOCOL.md) for application message semantics and recovery details.

Apache-2.0. Uses [HACP's Rust v2 API](https://github.com/manvendersingh21/hacp) pinned to `f3d30794b2b502f65b7bd28fd666d7e33c888c59` (HACP/2.0 draft).

Related: [HIVE](https://github.com/manvendersingh21/HIVE), the larger agent orchestration project that motivated this smaller tool.
