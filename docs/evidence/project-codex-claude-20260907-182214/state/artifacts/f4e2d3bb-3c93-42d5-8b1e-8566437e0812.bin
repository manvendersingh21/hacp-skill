"""Validated JSON storage for Task Pocket, using atomic file replacement.

Public operations raise ValueError for invalid input or database contents and
OSError for filesystem failures. Concurrent writers are outside this API's scope.
"""

import json
import os
from pathlib import Path
import tempfile


def _positive_integer(value):
    return type(value) is int and value > 0


def _unique_object(pairs):
    result = {}
    for key, value in pairs:
        if key in result:
            raise ValueError("Invalid database: duplicate JSON key {!r}".format(key))
        result[key] = value
    return result


def _validate(database):
    if not isinstance(database, dict) or set(database) != {"next_id", "tasks"}:
        raise ValueError("Invalid database: expected an object with next_id and tasks")
    if not _positive_integer(database["next_id"]):
        raise ValueError("Invalid database: next_id must be a positive integer")
    if not isinstance(database["tasks"], list):
        raise ValueError("Invalid database: tasks must be an array")
    seen = set()
    for task in database["tasks"]:
        if not isinstance(task, dict) or set(task) != {"id", "title", "done"}:
            raise ValueError("Invalid database: each task must contain id, title and done")
        if not _positive_integer(task["id"]) or task["id"] in seen:
            raise ValueError("Invalid database: task IDs must be unique positive integers")
        seen.add(task["id"])
        title = task["title"]
        if not isinstance(title, str) or not title or title != title.strip():
            raise ValueError("Invalid database: task titles must be trimmed nonempty strings")
        if type(task["done"]) is not bool:
            raise ValueError("Invalid database: done must be a boolean")
    if seen and database["next_id"] <= max(seen):
        raise ValueError("Invalid database: next_id must be greater than every task ID")
    return database


def _read(path):
    try:
        with path.open("r", encoding="utf-8") as stream:
            database = json.load(stream, object_pairs_hook=_unique_object)
    except FileNotFoundError:
        return {"next_id": 1, "tasks": []}
    except (ValueError, UnicodeError) as error:
        raise ValueError("Invalid database JSON: {}".format(error)) from None
    return _validate(database)


def _write(path, database):
    """Replace only after the complete new JSON file has been flushed to disk."""
    path.parent.mkdir(parents=True, exist_ok=True)
    temporary_path = None
    try:
        with tempfile.NamedTemporaryFile(
            mode="w", encoding="utf-8", dir=str(path.parent),
            prefix=".task-pocket-", suffix=".tmp", delete=False
        ) as stream:
            temporary_path = stream.name
            json.dump(database, stream, ensure_ascii=True, allow_nan=False)
            stream.write("\n")
            stream.flush()
            os.fsync(stream.fileno())
        os.replace(temporary_path, str(path))
    finally:
        if temporary_path is not None:
            try:
                os.unlink(temporary_path)
            except FileNotFoundError:
                pass


def add_task(path, title):
    """Add a trimmed, nonempty title and return its new task dictionary."""
    if not isinstance(title, str) or not title.strip():
        raise ValueError("Task title must be a nonempty string")
    path = Path(path)
    database = _read(path)
    task = {"id": database["next_id"], "title": title.strip(), "done": False}
    database["tasks"].append(task)
    database["next_id"] += 1
    _write(path, database)
    return dict(task)


def list_tasks(path, include_done=False):
    """Return tasks ordered by ID, excluding completed tasks by default."""
    database = _read(Path(path))
    return [
        dict(task)
        for task in sorted(database["tasks"], key=lambda task: task["id"])
        if include_done or not task["done"]
    ]


def complete_task(path, task_id):
    """Mark a positive integer task ID complete; repeated completion succeeds."""
    if not _positive_integer(task_id):
        raise ValueError("Task ID must be a positive integer")
    path = Path(path)
    database = _read(path)
    for task in database["tasks"]:
        if task["id"] == task_id:
            if not task["done"]:
                task["done"] = True
                _write(path, database)
            return dict(task)
    raise ValueError("Task ID {} not found".format(task_id))
