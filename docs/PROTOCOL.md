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
| `session.close` | `{ "reason": "..." }` | Explicit ending. |

Envelopes must pass HACP shape/canonical checks, session membership and ID checks, and recipient checks. Incoming inbox files are read only for the requesting peer. Duplicate message IDs have no further effect, even if their bodies differ. An answer must address the original question author and cannot replace an earlier answer.

Unknown kinds are valid, recoverable messages. Incoming lifecycle notifications never drive contract transitions: command handlers perform transitions under the authoritative lock and then emit notifications. Writing an envelope cannot forge bilateral acceptance. Peer flags and files still rely on trusted access to the local project, not cryptographic authentication.

## Persistence

A single snapshot transaction records transitions, deduplication IDs, deliveries, fetch history, and events together. All committed messages/events remain in the snapshot as durable projection intents. Recovery rewrites the derived contract and inbox files, and reconstructs the readable log from each event's stored wording. It does not consume messages merely because they were fetched. Polling `--all` recovers fetched envelopes; outstanding questions and required contract actions remain visible.

The metadata lock file is opened once per command and is never replaced. Its advisory lock dies with the process. Snapshot/projection files are written beside their destinations, synced, and atomically renamed, then their parent directory is synced. Crash tests cover before snapshot commit, after commit, and after delivery but before log projection. Test crash injection is compiled into debug builds only.

Immutable artifact copies are synced before their metadata is committed. Verification uses a separate process-released attempt lease, so metadata access stays available during commands. Completed command measurements are committed individually. Recovery marks an abandoned lease interrupted and never reruns a command. Explicit retry creates a new verification ID. The result is applied only if the pending submission still matches the one measured.

Ownership is checked atomically with initial and amendment freezes, including filesystem aliases. During amendment negotiation, other contracts see the previous frozen output set. Terminal contracts release claims. Shell commands and editors are cooperative: this protocol does not prohibit out-of-band edits. Hash checks detect changes in the submitted artifacts; they are not a filesystem sandbox or a continuous mutation monitor.
