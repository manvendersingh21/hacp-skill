# Changelog

## 0.1.1

Material negotiated behavior can now be included in optional structured `requirements`. The existing HACP canonical revision digest binds the complete terms, and changes to frozen requirements use bilateral amendments.

The new `complete` command records successful completion only for an active session with at least one contract, every recorded contract settled, accepted counterparty verification of each latest submission/revision/artifact set, and no unanswered questions. Generic `close` remains available for unfinished or abandoned work and records a distinct termination outcome.

- Preserve old three-field terms and format-1 snapshot readability.
- Update agent instructions to promote correctness-relevant chat decisions into frozen terms.
- Pin the protocol dependency to HACP 1.1.1 without changing HACP/2.0 protocol semantics.
- Publish the fresh ledger validation report and its release evidence archive.

Validation: all 27 binding tests and 147 core tests passed. The fresh ledger passed its 25-test suite and independent adversarial evaluation; its suite detected 14/14 sampled faulty variants. Formatting, Clippy, and skill validation passed for the binding.

Upgrade both peer binaries and installed skills together. Older binaries reject new requirements and can discard the new outcome field. Completion acknowledgments remain conversational; the CLI does not enforce them structurally. HACP does not prove exclusive editor ownership or exact file-write timing.

Installation:

```sh
cargo install --git https://github.com/manvendersingh21/hacp-skill --tag v0.1.1 --locked
hacp install --cli all
```

Native skill installation refuses conflicting existing files. Review and update older installed skills using `skills/hacp/SKILL.md` from this release. This release remains source-only; it contains no prebuilt platform binaries.
