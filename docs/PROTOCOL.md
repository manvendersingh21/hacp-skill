# HACP shared-directory binding, v0.1

The dependency is HACP 1.1.0's `hacp::v2` module at revision `f3d30794b2b502f65b7bd28fd666d7e33c888c59`. The envelope protocol identifier is `HACP/2.0`; its specification remains a draft. Terms/revision digests use library canonicalization. Artifact byte digests are SHA-256. Sessions and contracts use the library lifecycle API; this package adds filesystem ownership, CLI identity, delivery, and measured execution.

Peers map to `urn:hacp:agent:a` and `urn:hacp:agent:b`. The project has one session, with a unique session ID and exactly two participants. `start` registers a, while `join` calls `Session::accept` as b. Only active sessions form contracts.

## Envelopes

HACP permits application-defined kind strings. This binding uses:

| Kind | Body | Meaning |
| --- | --- | --- |
| `hacp.skill.ask` | `{ "text": "..." }` | Question to the other registered peer. |
| `hacp.skill.answer` | `{ "text": "..." }` | Reply; envelope `in_reply_to` must reference the other peer's question. |
| `contract.proposed`, `contract.countered`, `contract.amendment.proposed` | Contract ID, task, terms, digest, revision, state, next action | Durable notification of an already committed transition. |
| `contract.frozen`, `contract.no_agreement`, `contract.declined` | Same | Agreement, exhausted bounds, or refusal. `contract.declined` is an application extension. |
| `submission.delivered`, `verification.delivered` | Same | The snapshot holds artifact and measured verification records. |
| `session.close` | `{ "reason": "...", "outcome": "completed" \| "terminated" }` | Binding outcome from `complete` or `close`; legacy notifications omit `outcome`. |

Envelopes must pass HACP shape/canonical checks, session membership and ID checks, and recipient checks. Incoming inbox files are read only for the requesting peer. Duplicate message IDs have no further effect, even if their bodies differ. An answer must address the original question author and cannot replace an earlier answer.

Unknown kinds are valid, recoverable messages. Incoming lifecycle notifications never drive contract transitions: command handlers perform transitions under the authoritative lock and then emit notifications. Writing an envelope cannot forge bilateral acceptance. Peer flags and files still rely on trusted access to the local project, not cryptographic authentication.

## Terms and completion

The binding's strict terms object requires `inputs`, `outputs`, and `acceptance`, and optionally accepts `requirements` as structured JSON. Unknown top-level fields remain errors; nested requirements may use arbitrary keys and values allowed by core canonicalization, including ordered arrays. HACP/2.0 permits integer numbers only. Omitted or top-level null `requirements` is treated as absent; nested null values are preserved. No coding schema is added to core HACP.

`ask`/`answer` = deliberation; `requirements` in frozen terms = binding agreement. Agents must promote resolved correctness-relevant decisions before acceptance. The existing core content digest covers the full serialized terms and the unchanged revision preimage is `{contract_id, revision, content}`. Proposal, counter, freeze, and amendment all use that content. Changes to frozen requirements use bilateral amendment; messages never mutate a revision. Submission and verification remain bound to the current frozen revision.

`complete` checks under the metadata lock that the session is active, at least one contract exists, every recorded contract is `settled`, and its latest verification is a valid `accept` by the counterparty matching the latest submission's artifact set, contract ID, and current frozen revision. All unanswered `hacp.skill.ask` messages block completion, even if fetched; only a validated linked answer resolves them. There is no optional-contract or nonblocking-question flag. Failed checks list the exact blockers and do not close the session. Incoming lifecycle messages cannot establish completion.

On success, `complete` calls core `Session::close`, sets snapshot `outcome: completed`, and commits a `complete` event and close notification together. `close --reason TEXT` retains generic termination semantics, setting `outcome: terminated` and a `close` event without requiring settled work. Opening sessions still become `abandoned` through `close`; active sessions become `closed` through either operation. Terminal outcomes cannot be overwritten through these commands.

Completion acknowledgment has no structural representation in the current binding. This patch enforces the objectively checkable subset; explicit bilateral acknowledgment enforcement is deferred. No conversational text, including an incoming close notification or a reason claiming success, establishes acknowledgment or a successful outcome.

The additive optional `outcome` field keeps snapshot format 1 readable without migrating revisions. Missing outcomes in old snapshots remain unspecified. Omitted requirements preserve old terms serialization and all old digests. Older binaries reject new requirements and can lose the new outcome field on writes; upgrade both peers together. Core Rust APIs, wire schemas, digest preimages, and lifecycle states are unchanged.

## Acceptance audit

Core `agreed_by` and `agreed_terms_digest` are pending negotiation votes. Freeze and successful amendment deliberately clear them. Names are retained for serialization/schema compatibility; they are not historical acceptance receipts.

The snapshot retains the originating proposal/counter/amendment notification with its author, full terms and terms digest, followed by the counterparty's `contract.frozen` notification with the terms and revision digests. Associated events retain actor and timestamp. Following these events for a contract reconstructs who accepted each revision and when it froze, even after later amendments. The core library itself has no persistent event store; persistence is this binding's responsibility. A dedicated revision-level acceptance receipt is deferred. Neither the projected log nor the authoritative local snapshot is a cryptographically authenticated or tamper-proof audit trail, and a receipt alone would not make it one.

## Persistence

A single snapshot transaction records transitions, deduplication IDs, deliveries, fetch history, and events together. All committed messages/events remain in the snapshot as durable projection intents. Recovery rewrites the derived contract and inbox files, and reconstructs the readable log from each event's stored wording. It does not consume messages merely because they were fetched. Polling `--all` recovers fetched envelopes; outstanding questions and required contract actions remain visible.

The metadata lock file is opened once per command and is never replaced. Its advisory lock dies with the process. Snapshot/projection files are written beside their destinations, synced, and atomically renamed, then their parent directory is synced. Crash tests cover before snapshot commit, after commit, and after delivery but before log projection. Test crash injection is compiled into debug builds only.

Immutable artifact copies are synced before their metadata is committed. Verification uses a separate process-released attempt lease, so metadata access stays available during commands. Completed command measurements are committed individually. Recovery marks an abandoned lease interrupted and never reruns a command. Explicit retry creates a new verification ID. The result is applied only if the pending submission still matches the one measured.

Ownership is checked atomically with initial and amendment freezes, including filesystem aliases. During amendment negotiation, other contracts see the previous frozen output set. Terminal contracts release claims. Shell commands and editors are cooperative: this protocol does not prohibit out-of-band edits. Hash checks detect changes in the submitted artifacts; they are not a filesystem sandbox or a continuous mutation monitor.
