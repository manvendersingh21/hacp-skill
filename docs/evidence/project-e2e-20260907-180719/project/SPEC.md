# Task Pocket: a tiny local task CLI

Build a dependency-free Python 3 task tracker. It must run with the installed python3 (do not assume Python 3.11+).

Public CLI (flags before subcommand):
- python3 task_cli.py --db PATH add "title": print the created task as JSON.
- python3 task_cli.py --db PATH list: print a JSON array of open tasks, sorted by id.
- python3 task_cli.py --db PATH list --all: include completed tasks, sorted by id.
- python3 task_cli.py --db PATH done ID: print the completed task as JSON.

Tasks have integer id, trimmed nonempty string title, and boolean done. First id is 1; IDs increase and are never reused after completion. New tasks start done=false. Repeating done on a completed task succeeds idempotently. Missing task IDs, malformed IDs, and blank titles fail with nonzero exit and a useful stderr message, without corrupting the database or printing a traceback. A missing database lists as []; adding creates its missing parent directories. Data persists across fresh CLI processes. Malformed JSON or an invalid database shape must fail clearly and remain byte-for-byte unchanged. Successful stdout must contain only the specified JSON.

Suggested storage interface to explicitly discuss and agree through hacp:
add_task(path, title) -> task dict; list_tasks(path, include_done=False) -> list of task dicts; complete_task(path, task_id) -> task dict. Use atomic database replacement for writes. No network, dependencies, daemon, or extra features. Concurrent writers are outside this small project's scope.

Peer a (Codex) owns task_store.py: storage, validation, persistence, API.
Peer b (AGY) owns task_cli.py, tests/test_tasks.py, README.md: CLI, unittest coverage, usable README examples.
Both peers may write only their own frozen outputs plus their own terms-a.json/terms-b.json coordination files. SPEC.md is the immutable brief. Do not add unrelated output files.
Acceptance for both contracts must include python3 -m unittest discover -s tests -v and meaningful nonzero test coverage. Wait for the peer's dependencies rather than treating zero discovered tests as completion.
