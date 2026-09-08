# Transactional ledger: HACP collaboration performance report

**Experiment date:** September 7, 2026, America/Los_Angeles (September 8 UTC).

**Evaluation:** Post-run forensic review by a separate evaluator session that did not implement either task artifact.

**Overall score:** **93.75/100** across the seven categories below.

The ledger works, and the record shows substantive negotiation and cross-verification. HACP formally froze the file-and-command contracts, but those revisions did not contain the detailed behavioral agreement. A separate evaluator reproduction also found that the CLI can close a session with unfinished work and unanswered questions.

The empty `agreed_by` and `agreed_terms_digest` fields are **not evidence that freezing failed**. The evaluated implementation deliberately clears them after creating a frozen revision.

This is a performance report for one collaboration experiment, not a model ranking, throughput benchmark, or estimate of reliability across repeated runs. Scores are evaluator judgments under the requested rubric, not statistical confidence estimates. The evaluator used recorded peer identities `a` and `b`; the persisted HACP records alone do not establish the exact client/model behind each identity.

## Original task and evaluation scope

Two agents collaboratively implemented a transactional bank ledger. Peer A owned only `ledger.py`; Peer B owned only `test_ledger.py`. Before coding, both had to negotiate and explicitly agree on:

1. Exception types for each failure.
2. `transfer()` return value on first success.
3. `transfer()` return value on exact idempotent retry.
4. Whether failed transactions reserve or consume their `tx_id`.
5. Validation order when multiple conditions are invalid.
6. Exact normalization rules for account IDs and transaction IDs.

Peer B additionally had to challenge at least two ambiguous edge cases. Following agreement, the agents had to implement their respective files, test the integrated implementation, communicate genuine failures if found, cross-verify, explicitly agree overall completion, and remain available until both sides were verified.

The evaluator inspected both Python files, HACP session state, both projected contracts, all inbox messages, the collaboration log, preserved artifact copies, and saved verification output. Agent messages were evidence of expressed proposals and acceptances, not proof that an implementation, test run, or protocol transition happened. Those claims were checked against the files and measured records.

The original experiment files were not modified during evaluation. Additional ledger tests and mutations ran in memory. Temporary HACP reproduction projects were removed when finished.

### Evaluated versions

- `hacp-skill` source: [`e2e8a30929e9d7cfbb216c0f01ac73d60fb91795`](https://github.com/manvendersingh21/hacp-skill/tree/e2e8a30929e9d7cfbb216c0f01ac73d60fb91795).
- Pinned HACP protocol dependency: [`f3d30794b2b502f65b7bd28fd666d7e33c888c59`](https://github.com/manvendersingh21/hacp/tree/f3d30794b2b502f65b7bd28fd666d7e33c888c59).
- Findings describe these versions. Recommendations below are not claims that fixes have shipped.

### Evidence availability and limitations

The original experiment directory and raw HACP records were inspected locally. This publication contains the report, selected evidence excerpts, identifiers, hashes, and measured results; it does not publish the complete raw experiment archive or the temporary evaluator harness. Public source links are pinned to the evaluated commits. Thus readers can inspect the implementation behind protocol findings, but cannot independently replay the entire experiment from this report alone.

The experiment directory had no Git history or per-agent file-write audit. Filesystem timestamps establish when the surviving files appeared, but do not independently identify their writers or exclude earlier drafts elsewhere. The evaluator did not infer write authorship from a submission's producer label.

HACP uses cooperative local identities. As the [evaluated README explains](https://github.com/manvendersingh21/hacp-skill/blob/e2e8a30929e9d7cfbb216c0f01ac73d60fb91795/README.md#recovery-and-scope), peer flags are not authentication and file claims do not intercept editor writes. Matching local records establish internal consistency, not tamper-proof external attestation.

## Evidence reconciliation

The following cross-checks passed:

- All **22 persisted messages** match their inbox copies.
- All **31 log events** match the authoritative session snapshot.
- Both projected contract files match the session snapshot.
- Both current Python files exactly match their preserved submitted artifacts and SHA-256 hashes.
- Both frozen revision digests recompute correctly from canonical revision content.
- Submissions, artifacts, and verification records reference the correct contracts and revisions.
- Saved verification stdout/stderr match the recorded command measurements.

| Record | Observed value |
| --- | --- |
| Session | `s-1a47341cc9b745b3b15d6d3bae20f79a` |
| Final session state | `closed` |
| Implementation contract | `c-7524785247ef4a5283668455e521ae42` |
| Test contract | `c-f8e163acfc6d48de8686957eae678053` |
| Final contract states | Both `settled` |
| Frozen revisions | One per contract |
| Post-freeze amendments | Zero |
| Submissions / verification records | One of each per contract |
| Questions / linked answers | Six / six |
| Unanswered questions at close | Zero |

### Artifact and revision fingerprints

| Artifact | Recorded producer | Size | SHA-256 |
| --- | --- | ---: | --- |
| `ledger.py` | `urn:hacp:agent:a` | 2,786 bytes | `9986e74c798d8ffcbb1773e95085f23f7364b387ee3384065d470cb14f7ec3af` |
| `test_ledger.py` | `urn:hacp:agent:b` | 20,441 bytes | `8b34a4061b76eff56ae88de8d279c31ff05b3efb626690afda65b722e54c91ad` |

Implementation frozen revision:

```text
7566fb8032f82d9737d52fc10b60585c374e5d4c04c560e813ad10014d7516a4
```

Test frozen revision:

```text
448e79e068429707f81f2f10e98c9343be308650a20d31c83cf12c64e8a870a2
```

The terms-content digest and revision digest are intentionally different: the latter hashes canonical `{contract_id, revision, content}`, binding the terms to a specific contract and revision number.

## Evaluation 1: Artifact correctness

**ARTIFACT CORRECTNESS: 100/100**

The evaluator ran the complete supplied test suite:

```text
python3 -B -m unittest discover -v
Ran 36 tests
OK
```

Independent adversarial evaluation also passed:

| Evaluator check | Result |
| --- | --- |
| 3,200 transfer combinations | Correct success/error behavior, precedence, failure atomicity, and failed-ID reuse |
| 48 account-creation combinations | Correct types, precedence, duplicate handling, and unchanged state after rejection |
| 5,000 deterministic operations against an independent model | Balances, conservation, transaction history, conflicts, and retries matched |
| Replay of all 957 successful transactions from that model | No additional transfers or history changes |
| Literal IDs, including whitespace, Unicode variants, and embedded NUL | Remained distinct |
| Integers on the order of `10**200` | Exact arithmetic |
| Two simultaneously existing `Ledger` instances | Independent balances and transaction histories |

The transfer matrix included fresh and committed IDs, malformed IDs, builtin-string subclasses, missing accounts, equal sender/receiver, invalid amount types, very large amounts, and objects whose equality/hash methods raise if called. Expected domain exceptions and unchanged balances/history were checked after rejections. Valid failed IDs were then reused successfully. The account matrix independently exercised invalid ID/balance combinations and duplicate accounts. The stateful model used a fixed random seed, tracked balances and committed transactions separately, and checked conservation and nonnegative balances throughout.

The implementation correctly handles all requested categories:

- Successful and full-balance transfers.
- Insufficient funds, zero/negative amounts, and invalid amount types.
- Unknown sender and unknown receiver.
- Self-transfer and duplicate account creation.
- Exact retries returning the singleton `True` without moving money again.
- Committed IDs reused with different values or types.
- Reuse of failed transaction IDs with corrected or different parameters.
- Conservation and preservation of balances/history after domain failures.
- Literal ID handling and the negotiated validation order.

Source inspection confirmed that domain validation occurs before debit, credit, and history update, and committed replay occurs before checking current funds.

**Deductions: none.** This score applies to the negotiated sequential, in-memory API. Concurrency and crash durability were outside the agreed scope. The tests do not establish correctness under injected process failures or resource exhaustion.

## Evaluation 2: Negotiation quality

**NEGOTIATION QUALITY: 100/100**

The principal behavioral negotiation records were:

| Reference | UTC time | Peer | Message ID | Content |
| --- | --- | --- | --- | --- |
| N1 | 03:00:35 | A | `m-ea89a3d025064f89b7e87c365a1a777c` | Original detailed behavioral proposal |
| N2 | 03:00:51 | B | `m-ea1679b0626b42c6a1b91c373361c1ee` | Explicit objections and proposed corrections |
| N3 | 03:01:20 | A | `m-4723efdbe5a34786b184e1759a71ca77` | Acceptance of corrections and revised specification |
| N4 | 03:01:56 | B | `m-3754cde7dee84c24ae0089f7cdaa2dbc` | Explicit acceptance with corrected self-transfer interpretation |
| N5 | 03:02:06 | A | `m-3811d8e9c957417cbaf78a7703d1238b` | Confirmation of B's interpretation and correction of the typo |

### Resolution of the six required questions

| Required question | Proposed and finally accepted behavior | A accepted? | B accepted? | Evidence and artifact match |
| --- | --- | --- | --- | --- | --- |
| Exceptions | Eight specific subclasses of `LedgerError`; failures raise rather than return `False` | Yes | Yes | N1–N5; implementation and hierarchy/error tests match |
| First successful transfer | Return singleton `True` | Yes | Yes | N1, N2, N4, N5; implementation/tests match |
| Exact retry | Return singleton `True`, without moving money or checking current funds again | Yes | Yes | N1–N5; implementation/tests match |
| Failed transaction IDs | Not reserved; subsequent calls may use corrected or different parameters | Yes | Yes | N1–N5; implementation/tests match |
| Validation order | Transaction-ID format → committed replay/conflict → sender format → receiver format → self-transfer → amount → sender existence → receiver existence → funds | Yes | Yes | N1, N3–N5; implementation matches; tests cover most distinctions |
| Normalization | None; nonempty builtin strings, case-sensitive, whitespace preserved; subclasses rejected | Yes | Yes | N1–N5; implementation/tests match |

The agreed exception mapping was:

| Failure | Exception |
| --- | --- |
| Malformed account ID | `InvalidAccountIdError` |
| Duplicate account creation | `AccountAlreadyExistsError` |
| Well-formed but unknown account | `AccountNotFoundError` |
| Self-transfer | `SameAccountError` |
| Invalid amount or initial balance | `InvalidAmountError` |
| Insufficient sender funds | `InsufficientFundsError` |
| Malformed transaction ID | `InvalidTransactionIdError` |
| Committed transaction-ID conflict | `DuplicateTransactionError` |

### Genuine challenges from B

B challenged more than two ambiguities:

1. **Equality-only replay:** `True == 1` and `1.0 == 1` could incorrectly qualify as exact retries.
2. **Builtin types versus subclasses:** inconsistent `str`/`type==str` and `isinstance(int)` wording left subclasses and custom equality ambiguous.
3. **Multiple-invalid-condition precedence:** self-transfer involving unknown accounts, and invalid amounts involving unknown accounts.
4. **Self-transfer wording:** B caught A's contradictory “else” wording; A explicitly corrected it.

B also requested replacing `pytest` with the specified standard-library `unittest` command. The formal counterproposal contains that change.

Selected excerpts establish that this was explicit negotiation:

> B, N2: “I cannot accept the first proposal unchanged.”

> A, N3: “All three challenges accepted. Revised final contract:”

> B, N4: “Explicit agreement: I accept your revised behavioral contract in m-4723efdbe5a34786b184e1759a71ca77 plus original exception hierarchy/no normalization/sequential scope.”

> A, N5: “Confirmed -- that was a typo, thank you for catching it: step 5 raises SameAccountError WHEN sender == receiver (not the inverse). All other points confirmed exactly as you restated.”

**Deductions: none.** Initial ambiguities were resolved before the surviving source files appeared. Failure to include this discussion in frozen terms is assessed under formal compliance and state-machine integrity.

## Evaluation 3: Contract convergence

**CONTRACT CONVERGENCE: 100/100**

A's expressed understanding was reconstructed from N1, N3, and N5. B's was reconstructed from N2 and N4. Passing tests were not used as proof of agreement. The runner decision is additionally supported by B's 03:01:11 request and A's 03:01:36 answer/counterproposal.

| Contract dimension | Peer A | Peer B | Match? |
| --- | --- | --- | --- |
| Failure exceptions | Named `LedgerError` subclasses | Explicitly accepts hierarchy | Yes |
| First-success result | Singleton `True` | Singleton `True` | Yes |
| Exact-retry result | Singleton `True`; no additional transfer | Same | Yes |
| Failed-ID consumption | Unreserved; fresh validation on reuse | Same | Yes |
| Transfer validation order | Ordered checks listed above | Accepts order after self-transfer correction | Yes |
| ID normalization | None; literal nonempty builtin strings | Same, explicitly including whitespace | Yes |
| Money types | Builtin `int`; reject bool/subclasses | Same | Yes |
| Committed replay mismatch | Any value/type mismatch raises `DuplicateTransactionError` before other checks | Same | Yes |
| Account-creation API | `create_account(account_id, balance=0)` returns `None`; ID → balance → duplicate checks | Same | Yes |
| Balance API | `balance(account_id)` returns builtin `int`; ID format before existence | Same | Yes |
| Atomicity scope | Sequential calls; failures preserve balances/history | Explicitly accepts sequential scope | Yes |
| Acceptance runner | Standard-library `unittest` | Requested and accepted `unittest` | Yes |

```text
Required dimensions: 6 / 6
Extended comparison: 12 / 12
Contract Convergence = 12 / 12 × 100 = 100%
```

This establishes convergence of the peers' **expressed behavioral contract**, not completeness of the hashed HACP terms.

## Evaluation 4: Protocol compliance

**PROTOCOL COMPLIANCE: 90/100**

### Complete event timeline

All times are **UTC on September 8, 2026**, corresponding to September 7 evening in America/Los_Angeles. Related events at the same timestamp are grouped. Source creation times are filesystem observations; other entries are persisted protocol events.

| Time | Event |
| --- | --- |
| 02:58:58 | A starts; declares ownership of `ledger.py` |
| 02:59:11 | B joins; declares ownership of `test_ledger.py` |
| 02:59:20 | B asks all contract questions and initial adversarial questions |
| 03:00:35 | A proposes detailed behavior |
| 03:00:45 | A formally proposes implementation contract |
| 03:00:51 | B challenges replay, type handling, and precedence |
| 03:01:11 | B requests corrected acceptance runner and complete behavioral terms |
| 03:01:20 | A accepts corrections and provides revised behavior |
| 03:01:36 | A acknowledges runner change and formally counters its proposal |
| 03:01:56 | B explicitly accepts behavior, freezes A's contract, and proposes B's contract |
| 03:02:06 | A confirms B's interpretation and corrects the self-transfer typo |
| 03:02:12 | A freezes B's contract |
| 03:02:21.734 | Surviving `ledger.py` created |
| 03:02:48 | A submits; implementation artifact preserved |
| 03:02:56 | A asks B to verify and requests notification when tests are ready |
| 03:05:33.674 | Surviving `test_ledger.py` created |
| 03:06:01 | B reports completed tests and intended submission/verification |
| 03:06:02 | B submits; test artifact preserved; B's verification of A starts |
| 03:06:04 | B's measured verification accepts A's submission |
| 03:06:13 | A's verification of B starts |
| 03:06:14 | A's measured verification accepts B's submission |
| 03:06:20 | B explicitly agrees overall completion |
| 03:06:30 | A explicitly agrees overall completion |
| 03:06:31 | A sends close notification and closes session |

The recorded start-to-close interval is **7 minutes 33 seconds**. This is one observed session duration, not a controlled speed comparison; token usage and cost were not established by this evaluation.

A's final typo clarification occurred after A's narrow file-and-command contract froze, but before either source file appeared. Both formal contracts froze before either surviving file's creation timestamp.

### Compliance findings

| Check | Finding |
| --- | --- |
| Negotiation before implementation | Supported by explicit messages and both source creation timestamps |
| Explicit bilateral agreement before coding | Supported for the surviving artifacts |
| Formal bilateral acceptance | Yes, for both file-and-command contracts |
| Frozen digests created | Yes; one revision per contract |
| `agreed_by` contains both peers now | No; deliberately cleared at freeze |
| Artifacts appeared before formal agreement | No such evidence; surviving files appeared afterward |
| A edited only A's file; B only B's | Submission ownership matches; actual write exclusivity cannot be proved |
| Correct contract/revision binding | Yes, recomputed and cross-checked |
| Verification after valid submission | Yes |
| Both peers cross-verified | Yes |
| Explicit mutual completion | Yes |
| Close after required verification | Yes |
| Unanswered questions at close | None: six questions, six linked answers |
| Post-freeze amendments | None; both contracts have `amendments = 0` |

**Deduction: −10, MAJOR — incomplete formal contract coverage.** Both frozen revisions contain only paths and acceptance commands. The six behavioral decisions were not included or referenced by immutable content digest, despite B requesting their inclusion.

**INFORMATIONAL — provenance limitation:** artifact producer labels identify who submitted each artifact under a peer identity, not every process that edited it. No wrong-owner edit was demonstrated; no score deduction was made for an unproven edit violation.

### What was actually frozen

Implementation revision content, copied from the inspected contract:

```json
{
  "acceptance": [
    "python3 -c \"import ledger; assert hasattr(ledger, 'Ledger')\"",
    "python3 -B -m unittest -v test_ledger.py"
  ],
  "inputs": ["test_ledger.py"],
  "outputs": ["ledger.py"]
}
```

Test revision content:

```json
{
  "acceptance": ["python3 -B -m unittest -v test_ledger.py"],
  "inputs": ["ledger.py"],
  "outputs": ["test_ledger.py"]
}
```

The respective terms-content digests were:

```text
Implementation: 79293d8ed048105722f9fdd5c39e667eb56ec66ce4858194ab5e757f1e86a558
Tests:          3891ae3efceae3c63c04aea81eb9e40375826c7270a2ce756b00cdc9895c4fb6
```

These content objects do not include the exception hierarchy, replay semantics, normalization decisions, or behavioral message references.

### Measured cross-verification

| Verifier | Subject | Verification ID | Result |
| --- | --- | --- | --- |
| B | A's `ledger.py` submission | `v-25f9f5db376349cb8353c263afa7bed1` | `accept` |
| A | B's `test_ledger.py` submission | `v-eb55f7b1bb4940b28796785060b7eff2` | `accept` |

| Verification | Command | Recorded elapsed | Exit | Timeout |
| --- | --- | ---: | ---: | --- |
| B verifies A | `python3 -c "import ledger; assert hasattr(ledger, 'Ledger')"` | 77 ms | 0 | No |
| B verifies A | `python3 -B -m unittest -v test_ledger.py` | 98 ms | 0 | No |
| A verifies B | `python3 -B -m unittest -v test_ledger.py` | 108 ms | 0 | No |

The verification records include before/after artifact hash checks. Both measured suite runs reported 36 passing tests. The evaluator separately reran the suite; the conclusion does not rely solely on those stored results.

## Evaluation 5: HACP state-machine integrity

**STATE-MACHINE INTEGRITY: 75/100**

### Empty agreement fields: explanation and classification

The contract engine records acceptance against a canonical terms digest, requires both participants before freeze, creates a revision, then executes:

```rust
self.agreed_by.clear();
self.agreed_terms_digest = None;
self.state = ContractState::Executing;
```

See the evaluated [contract engine](https://github.com/manvendersingh21/hacp/blob/f3d30794b2b502f65b7bd28fd666d7e33c888c59/src/v2/contract.rs#L326). The CLI performs counterparty acceptance and freeze before committing the updated snapshot: [acceptance handler](https://github.com/manvendersingh21/hacp-skill/blob/e2e8a30929e9d7cfbb216c0f01ac73d60fb91795/src/contracts.rs#L146), [snapshot commit](https://github.com/manvendersingh21/hacp-skill/blob/e2e8a30929e9d7cfbb216c0f01ac73d60fb91795/src/store.rs#L94).

The relevant lifecycle is:

```text
proposed / countered
  → both accept identical terms
  → agreed
  → freeze revision; clear temporary votes
  → executing
  → verifying
  → settled
```

`agreed` is transitional. A proposed guard requiring `state == agreed` during implementation or submission would incorrectly reject valid work in this implementation.

Of the proposed explanations—agent error, state-machine bug, persistence bug, serialization/display bug, wrong ID, or insufficient evidence—the empty fields are closest to **misleading display/schema semantics**. Serialization itself is accurate. There was no evidence that these fields became empty through an agent bypass, lost persistence, or wrong-contract-ID lookup. The underlying audit weakness is that a frozen revision does not retain a durable, explicit acceptance receipt of its own.

### Independent CLI reproductions

The evaluator exercised the installed binary in temporary projects, separate from the original experiment:

| Attempt | Observed result |
| --- | --- |
| Submit before bilateral acceptance | Rejected: `contract is not executing` |
| Verify before bilateral acceptance | Rejected: `no pending submission to verify` |
| Proposer supplies the second acceptance | Rejected |
| Accept wrong terms digest | Rejected |
| Non-owner submits | Rejected |
| Submit wrong revision | Rejected |
| Self-verification | Rejected |
| Valid submission and counterparty verification | Accepted; correct revision bindings |
| Submit while amendment pending | Rejected |
| Submit old revision after accepted amendment | Rejected |
| Accept amendment bilaterally | Creates revision 2; preserves revision 1 |
| Close with unfinished contract and unanswered question | **Accepted** |

The last fixture ended with:

```text
session.state = closed
contract.state = executing
submissions = 0
verifications = 0
unanswered questions = 1
```

A later submission was rejected with `session must be active`. The [close handler](https://github.com/manvendersingh21/hacp-skill/blob/e2e8a30929e9d7cfbb216c0f01ac73d60fb91795/src/main.rs#L258) requires a reason but does not require contract outcomes or resolved questions.

This is a **workflow enforcement gap relative to the experiment's completion requirements**. Generic session termination can legitimately abort work, and the README describes `close` as ending a session. The finding is not that every termination must represent success, or that this violates the low-level session specification. It is that the workflow lacks a separately guarded successful-completion operation, so a close reason claiming success is not validated as such.

The original ledger run **did not exercise this premature-close path**: both artifacts were settled and all questions answered before closure.

### Findings and deductions

| Severity | Finding | Deduction |
| --- | --- | ---: |
| **MAJOR** | Detailed behavioral agreement is outside canonical frozen terms; the CLI schema accepts only three fields | −10 |
| **MAJOR** | Normal close can terminate unfinished work with unanswered questions; independently reproduced | −10 |
| **MINOR** | Agreement fields are misleading after freeze, and revisions lack durable per-revision acceptance receipts | −5 |

The terms limitation is visible in the [strict `Terms` structure](https://github.com/manvendersingh21/hacp-skill/blob/e2e8a30929e9d7cfbb216c0f01ac73d60fb91795/src/contracts.rs#L8).

Further conclusions:

- Supported CLI changes to frozen terms require amendment. No silent post-freeze revision mutation was found in this run.
- Artifact binding was correct: artifacts reference the negotiated formal revisions, although those revisions omit the behavioral discussion.
- A digest cannot establish when code was written. HACP gates submission; it does not prove implementation started after agreement.
- B's completion message calls its test submission “pending” six seconds after A had settled it. This is an **INFORMATIONAL stale conversational description**, corrected by A's reply, not evidence of stale persisted state.
- No **CRITICAL** finding was established.

## Evaluation 6: Collaboration value

**COLLABORATION QUALITY: 100/100**

The record contains six question/answer pairs. Their value is visible in decisions and artifacts:

| Collaboration evidence | Result |
| --- | --- |
| B challenges equality-only retry | Implementation requires exact builtin types before replay equality |
| B challenges subclass ambiguity | IDs and money use strict builtin-type rules |
| B challenges precedence | A explicitly confirms unusual multi-error ordering |
| B catches contradictory self-transfer wording | A corrects it before source creation |
| B rejects the proposed test runner | Formal acceptance command changes from `pytest` to `unittest` |
| A supplies API/error details | B's suite asserts exact exceptions, return identity, and ordering |
| Both perform measured verification | Both artifacts settle before mutual completion and close |

These interactions changed the proposal, acceptance criteria, implementation, and tests. They were not merely acknowledgments or communication volume.

There were no recorded failing integration runs or rework verdicts. Independent testing also found no implementation defect. There is no basis to allege concealed failures or award credit for a failure-repair cycle that did not occur.

**Deductions: none.** Useful bidirectional collaboration is supported by the record and resulting artifacts.

## Evaluation 7: Test quality

**TEST QUALITY: 90/100**

The 36-test suite was substantially stronger than happy-path coverage:

- `assertIs(..., True)` and exact exception-class checks.
- Balances checked before and after rejected calls.
- Prior committed transactions used as sentinels for history preservation.
- Failed IDs reused with corrected parameters.
- Retries tested after funds had moved elsewhere.
- Equality traps, bools, subclasses, Unicode distinctions, and large integers.
- Conservation checks and a deterministic 150-step model with final history replay.
- Predominantly public-API assertions rather than coupling to internal dictionaries.

The evaluator tested 14 temporary faulty variants. The supplied suite detected 12:

| Temporary mutation | Detected by supplied suite? |
| --- | --- |
| Retry returns integer `1` instead of singleton `True` | Yes |
| First success returns integer `1` instead of singleton `True` | Yes |
| Retry moves money again | Yes |
| Insufficient-funds failure reserves the transaction ID | Yes |
| Invalid-amount failure clears transaction history | Yes |
| Missing receiver causes sender debit before rejection | Yes |
| Receiver credited one unit too little | Yes |
| Replay accepts amount by loose equality alone | Yes |
| Transaction IDs stripped/case-folded | Yes |
| Account IDs stripped/case-folded on creation | Yes |
| Initial balances stored as floats | Yes |
| Duplicate account creation silently overwrites | Yes |
| Receiver existence checked before sender existence | **No** |
| Separate ledger instances share/reset state | **No** |

The first-success-only return mutation was also checked separately to ensure it altered the final success return rather than the earlier retry return. These are selected mutations, not an exhaustive mutation score or population estimate.

### Deductions

| Severity | Surviving defect | Why it passes | Deduction |
| --- | --- | --- | ---: |
| **MINOR** | Receiver existence checked before sender existence | The tests do not distinguish the two when both accounts are missing | −5 |
| **MINOR** | Separate ledger instances share/reset state | The suite never holds two independent ledgers alive together | −5 |

The actual implementation passed independent checks for both weaknesses. Deductions concern the suite's ability to detect faulty implementations, not defects in `ledger.py`.

Both missing-account branches raise the same exception class. A stronger precedence test needs an agreed observable diagnostic, or explicit instrumentation of lookup order. An exact exception-message assertion would otherwise introduce a requirement the peers never negotiated.

## Final scorecard

Scores reflect this experiment under the requested rubric. The same contract-coverage weakness affects both experiment compliance and the capability of the enforcement system; those categories deliberately assess different aspects of that weakness.

| Category | Score | Weight | Contribution |
| --- | ---: | ---: | ---: |
| Artifact Correctness | 100/100 | 20% | 20.00 |
| Negotiation Quality | 100/100 | 15% | 15.00 |
| Contract Convergence | 100/100 | 15% | 15.00 |
| Protocol Compliance | 90/100 | 20% | 18.00 |
| State-Machine Integrity | 75/100 | 15% | 11.25 |
| Collaboration Quality | 100/100 | 10% | 10.00 |
| Test Quality | 90/100 | 5% | 4.50 |
| **OVERALL HACP EXPERIMENT SCORE** | **93.75/100** | **100%** | **93.75** |

## Most important conclusions

1. **DID THE SOFTWARE TASK SUCCEED? YES.** The supplied suite and independent adversarial evaluation passed.
2. **DID THE TWO AGENTS ACTUALLY COLLABORATE? YES.** Specific objections produced agreed changes reflected in both artifacts.
3. **DID THEY REACH A SHARED CONTRACT BEFORE IMPLEMENTATION? YES, on the recorded artifact timeline.** Explicit agreement precedes both surviving files' creation timestamps. Earlier drafts and individual write operations are not audited.
4. **DID HACP FORMALLY ENFORCE THAT CONTRACT? NO, for the complete behavioral contract.** It enforced bilateral acceptance, revisions, ownership of submissions, and cross-verification for the narrower file-and-command terms.
5. **WAS THERE ANY HACP PROTOCOL VIOLATION? POSSIBLE, but none demonstrated in the recorded transitions.** Strict exclusive-write compliance cannot be certified without a write audit. No early submission, wrong-owner submission, self-verification, or premature close occurred in this run. “Possible” denotes missing evidence, not an accusation that such a violation occurred.
6. **IS THERE EVIDENCE OF A HACP IMPLEMENTATION/STATE-MACHINE BUG? YES, at the workflow enforcement layer relative to the experiment's requirements.** Premature close with unresolved work is reproducible. This is a missing successful-completion gate, not a demonstrated defect in the low-level session termination semantics. The empty agreement fields do not demonstrate a consensus or persistence bug.
7. **WOULD PASSING UNIT TESTS ALONE HAVE HIDDEN ANY PROTOCOL PROBLEMS? YES.** They reveal neither incomplete formal terms nor missing completion gates or write provenance.
8. **WHAT IS THE SINGLE MOST IMPORTANT ISSUE TO FIX?** Include the complete behavioral specification in the content both peers formally accept and freeze.

## Recommended HACP fixes

**P0: None justified by this evaluation.** No critical corruption or agreement-gate bypass was demonstrated through supported commands.

| Priority | Observed problem | Desired invariant | Suggested enforcement point | How to test the fix |
| --- | --- | --- | --- | --- |
| **P1** | Behavioral decisions remain outside frozen terms | Every required behavioral decision belongs to the accepted canonical revision | Extend `Terms` with a structured specification or immutable specification reference; validate at proposal/freeze | Changing replay or validation semantics changes the digest and requires fresh bilateral acceptance |
| **P1** | Close accepts unfinished work and unanswered questions | Successful completion requires required accepted verifications, resolved questions, and both peers' completion acknowledgments | Introduce a guarded `complete` operation; preserve a separately labelled abort/termination path | Reject successful completion with one unverified contract, unanswered question, or missing peer acknowledgment; allow explicit abort |
| **P2** | Acceptance evidence is cleared and fields look contradictory | Each frozen revision retains acceptance identities, content digest, and freeze timestamp | Freeze/amendment transition and status serialization | Serialize/restore initial and amended contracts; acceptance receipts remain attached to the correct revision |
| **P2** | Producer labels cannot prove exclusive editing or implementation start | Claims about write ownership/timing are supported by an attributable edit record | CLI/workspace audit integration; record pre-work baseline and revision-linked start events | An early write or wrong-peer write becomes detectable; otherwise report provenance as unverified |
| **P2** | Two concrete test defects escaped the supplied suite | Required ordering and instance isolation are independently observable and tested | Experiment acceptance suite | Reverse sender/receiver lookup and introduce shared instance state; both mutants must fail |

For the precedence test, first specify how the offending account is reported. Provenance auditing improves what HACP can establish about a run; it is distinct from physically preventing an agent from writing a file.
