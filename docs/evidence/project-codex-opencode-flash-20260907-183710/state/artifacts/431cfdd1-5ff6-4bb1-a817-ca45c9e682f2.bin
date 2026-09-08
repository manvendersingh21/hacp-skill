# Task Pocket

A tiny, dependency-free task tracker CLI (Python 3 standard library only).

## Requirements

- Python 3.9 or later (verified against the installed `python3` runtime); nothing newer is required.
- No third-party packages, no network, no daemon.

## Usage

Flags go before the subcommand. All state lives in one JSON database file that
`--db PATH` points at; it is created (including parent directories) on first
write.

### Add a task

```console
$ python3 task_cli.py --db tasks.json add "write the spec"
{"id": 1, "title": "write the spec", "done": false}
```

Titles are trimmed and must be nonempty after trimming.

### List open tasks

```console
$ python3 task_cli.py --db tasks.json list
[{"id": 1, "title": "write the spec", "done": false}]
```

`list` prints open tasks only, sorted by id. Include completed tasks too:

```console
$ python3 task_cli.py --db tasks.json list --all
[{"id": 1, "title": "write the spec", "done": true}, {"id": 2, "title": "review", "done": false}]
```

### Complete a task

```console
$ python3 task_cli.py --db tasks.json done 1
{"id": 1, "title": "write the spec", "done": true}
```

Completing an already-completed task succeeds idempotently. Ids increase
forever and are never reused, even for completed tasks.

## Errors

- Missing task ids, malformed ids, blank titles, corrupt databases, and
  unwritable paths all fail with exit code `1` and a single `Error: ...`
  line on stderr. No tracebacks are printed.
- A missing database is not an error: `list` prints `[]`, and `add` creates
  the database along with any missing parent directories.
- A corrupt or invalid-shaped database fails without being modified
  (byte-for-byte unchanged).
- Successful commands print only the specified JSON on stdout.

Data persists across separate CLI invocations; each command is an independent
process reading and atomically updating the same file.

## Storage layout

The database is a JSON array of task objects, each with exactly:

- `id`: positive integer
- `title`: trimmed, nonempty string
- `done`: boolean

Writes replace the database atomically (write to a temp file in the same
directory, then `os.replace`), so readers never observe a half-written file.

## Tests

```console
$ python3 -m unittest discover -s tests -v
```

The suite drives the real CLI in subprocesses against temporary databases and
covers happy paths, filtering, sorting, persistence, idempotence, id reuse
rules, error exits, and corruption preservation.
