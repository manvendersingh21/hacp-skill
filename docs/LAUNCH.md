# Launch drafts — not posted

## r/ClaudeAI

**Title:** I built a small skill so Claude Code and another coding CLI can agree on work and verify each other

I kept ending up as the messenger between two coding agents: copying interface questions, reminding them who owned which file, and checking whether “done” meant tested.

`hacp-skill` adds `/hacp` to Claude Code, AGY, and OpenCode, and `$hacp` to Codex. Two CLIs share one project directory. They negotiate explicit contracts, ask/answer questions, submit immutable artifact copies, and run each other's agreed acceptance checks. There's a readable log and bounded waiting; silence never counts as agreement.

It is a Rust binary plus a shared skill. No coordinator service, hooks, tmux, or MCP setup. This is an early local-filesystem tool, and it is not a sandbox: acceptance commands run with your user's permissions. Start in a scratch repo.

Repository: https://github.com/manvendersingh21/hacp-skill

The repository includes deterministic tests, actual CLI pairing outcomes, and a short accelerated recording. I'd particularly like feedback on how understandable the agreement/rework log is and where the agents still need human help.

## Hacker News

**Title:** Show HN: Hacp-skill – two coding agents agree on files and verify each other's work

**URL:** https://github.com/manvendersingh21/hacp-skill

**First comment draft:**

I built this to reduce the copy/paste coordination between two coding-agent terminals. It's one Rust binary and one skill, using Claude Code, Codex, AGY, or OpenCode's native skill discovery.

The interesting part is the boundary between “the agent says it is done” and a measured agreement: terms have canonical digests, both peers explicitly accept the same revision, outputs are preserved with hashes, and the other peer runs frozen acceptance commands before the HACP contract can settle. Amendments keep the previous file claims until both accept the replacement.

The local state is a single atomic snapshot with recoverable inbox, contract, and readable-log projections. No daemon. It supports exactly two peers on one local filesystem and relies on cooperative agents. It does not sandbox acceptance commands or prevent someone with filesystem access from editing state.

All six distinct CLI pairings were scheduled as opt-in live tests; the validation document reports actual results, including service failures. Deterministic CI doesn't need model accounts. Feedback on the protocol binding, recovery design, and practical usefulness is welcome.
