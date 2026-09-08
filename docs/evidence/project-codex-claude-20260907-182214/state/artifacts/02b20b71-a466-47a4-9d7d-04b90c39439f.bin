# Task Pocket

A tiny, dependency-free local task tracker for Python 3.

## Usage

Flags come before the subcommand. `--db PATH` selects the JSON database file
(created automatically, including any missing parent directories).

```sh
# Add a task
python3 task_cli.py --db tasks.json add "buy milk"
# {"id": 1, "title": "buy milk", "done": false}

# List open tasks (sorted by id)
python3 task_cli.py --db tasks.json list
# [{"id": 1, "title": "buy milk", "done": false}]

# Mark a task done
python3 task_cli.py --db tasks.json done 1
# {"id": 1, "title": "buy milk", "done": true}

# List all tasks, including completed ones
python3 task_cli.py --db tasks.json list --all
# [{"id": 1, "title": "buy milk", "done": true}]
```

Marking an already-completed task done again succeeds idempotently and
returns the same task. Missing task IDs, malformed IDs, and blank titles
exit with a nonzero status and an error message on stderr; the database is
left untouched. A missing database file lists as `[]`.

## Running the tests

```sh
python3 -m unittest discover -s tests -v
```
