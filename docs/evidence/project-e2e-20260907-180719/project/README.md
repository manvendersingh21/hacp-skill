# Task Pocket

A tiny dependency-free local task CLI written in Python 3.

## Usage

All commands require the `--db PATH` option preceding the subcommand.

### Add a Task
```bash
python3 task_cli.py --db ./tasks.json add "Buy groceries"
```
Outputs the newly created task as JSON:
```json
{"id": 1, "title": "Buy groceries", "done": false}
```

### List Open Tasks
```bash
python3 task_cli.py --db ./tasks.json list
```
Outputs a JSON array of all open tasks, sorted by `id`.

### List All Tasks (Including Completed)
```bash
python3 task_cli.py --db ./tasks.json list --all
```
Outputs a JSON array of all tasks (open and completed), sorted by `id`.

### Complete a Task
```bash
python3 task_cli.py --db ./tasks.json done 1
```
Outputs the updated completed task as JSON:
```json
{"id": 1, "title": "Buy groceries", "done": true}
```

## Running Tests
To run the test suite:
```bash
python3 -m unittest discover -s tests -v
```
