# Task Pocket

A tiny, dependency-free local task tracker: one Python CLI backed by a
single JSON database file. Works with any installed Python 3.

## Quick start

```sh
# add tasks (created tasks print as JSON)
python3 task_cli.py --db ~/tasks.json add "buy milk"
python3 task_cli.py --db ~/tasks.json add "write report"

# list open tasks (JSON array, sorted by id)
python3 task_cli.py --db ~/tasks.json list

# complete a task (prints the completed task as JSON)
python3 task_cli.py --db ~/tasks.json done 1

# include completed tasks in the listing
python3 task_cli.py --db ~/tasks.json list --all
```

Example session:

```sh
$ python3 task_cli.py --db ~/tasks.json add "buy milk"
{"id": 1, "title": "buy milk", "done": false}
$ python3 task_cli.py --db ~/tasks.json add "write report"
{"id": 2, "title": "write report", "done": false}
$ python3 task_cli.py --db ~/tasks.json done 1
{"id": 1, "title": "buy milk", "done": true}
$ python3 task_cli.py --db ~/tasks.json list
[{"id": 2, "title": "write report", "done": false}]
$ python3 task_cli.py --db ~/tasks.json list --all
[{"id": 1, "title": "buy milk", "done": true}, {"id": 2, "title": "write report", "done": false}]
```

## Commands

All commands take `--db PATH` (before the subcommand) to choose the
database file.

| Command | Effect |
| --- | --- |
| `--db PATH add "title"` | Create a task; prints the created task as JSON. Titles are trimmed and must be nonempty. |
| `--db PATH list` | Print a JSON array of open tasks, sorted by id. |
| `--db PATH list --all` | Same, but completed tasks are included too. |
| `--db PATH done ID` | Mark task `ID` done; prints the completed task as JSON. Repeating it succeeds idempotently. |

A task is a JSON object `{"id": <int>, "title": <str>, "done": <bool>}`.
IDs start at 1, always increase, and are never reused.

## Behavior details

- Data persists across CLI invocations in a single JSON file.
- A missing database lists as `[]`; `add` creates missing parent
  directories for the `--db` path automatically.
- Errors (missing task id, malformed id, blank title, malformed or
  invalid database) exit nonzero with a message on stderr — no
  traceback, and the database file is left byte-for-byte unchanged.
- Successful stdout contains only the specified JSON.

## Tests

```sh
python3 -m unittest discover -s tests -v
```

The suite runs the CLI as real subprocesses and covers adding, listing,
completing, idempotency, sorting, persistence, parent-directory
creation, and all documented error paths.

## Project layout

- `task_cli.py` — command-line interface (argument parsing, JSON output,
  error handling).
- `task_store.py` — storage layer: validation, atomic writes,
  persistence.
- `tests/test_tasks.py` — end-to-end unittest coverage.
