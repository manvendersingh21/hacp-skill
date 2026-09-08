"""Validated JSON storage for Task Pocket, using atomic file replacement.

The database is an array of tasks with exactly id, title and done keys.
Completed tasks are retained so that their IDs are never reused. Public
operations raise TaskStoreError for invalid input or unusable storage.
Concurrent writers are outside this module's scope.
"""

import json
import os
import tempfile


class TaskStoreError(ValueError):
    """A task operation could not be completed safely."""


def _unique_object(pairs):
    result = {}
    for key, value in pairs:
        if key in result:
            raise TaskStoreError("Invalid database: duplicate field {!r}".format(key))
        result[key] = value
    return result


def _read_tasks(path):
    try:
        with open(path, "r", encoding="utf-8") as stream:
            tasks = json.load(stream, object_pairs_hook=_unique_object)
    except FileNotFoundError:
        return []
    except TaskStoreError:
        raise
    except (ValueError, UnicodeError, RecursionError) as exc:
        raise TaskStoreError("Invalid database JSON: {}".format(exc)) from exc
    except OSError as exc:
        raise TaskStoreError("Cannot read database: {}".format(exc)) from exc

    if not isinstance(tasks, list):
        raise TaskStoreError("Invalid database: expected a JSON array of tasks")
    seen = set()
    for task in tasks:
        if not isinstance(task, dict) or set(task) != {"id", "title", "done"}:
            raise TaskStoreError("Invalid database: each task needs exactly id, title, done")
        task_id = task["id"]
        if type(task_id) is not int or task_id < 1 or task_id in seen:
            raise TaskStoreError("Invalid database: task IDs must be unique positive integers")
        title = task["title"]
        if not isinstance(title, str) or not title.strip() or title != title.strip():
            raise TaskStoreError("Invalid database: task titles must be trimmed nonempty strings")
        if type(task["done"]) is not bool:
            raise TaskStoreError("Invalid database: done must be a boolean")
        seen.add(task_id)
    return sorted(tasks, key=lambda task: task["id"])


def _write_tasks(path, tasks):
    temporary_path = None
    try:
        destination = os.path.abspath(path)
        parent = os.path.dirname(destination)
        os.makedirs(parent, exist_ok=True)
        with tempfile.NamedTemporaryFile(
            mode="w", encoding="utf-8", dir=parent,
            prefix=".task-pocket-", suffix=".tmp", delete=False
        ) as stream:
            temporary_path = stream.name
            json.dump(tasks, stream, ensure_ascii=True, indent=2)
            stream.write("\n")
            stream.flush()
            os.fsync(stream.fileno())
        os.replace(temporary_path, destination)
        temporary_path = None
    except (OSError, ValueError) as exc:
        raise TaskStoreError("Cannot write database: {}".format(exc)) from exc
    finally:
        if temporary_path is not None:
            try:
                os.unlink(temporary_path)
            except OSError:
                pass


def add_task(path, title):
    """Add a trimmed nonempty title and return the new task dictionary."""
    if not isinstance(title, str) or not title.strip():
        raise TaskStoreError("Task title must be a nonempty string")
    tasks = _read_tasks(path)
    task = {
        "id": max((item["id"] for item in tasks), default=0) + 1,
        "title": title.strip(),
        "done": False,
    }
    tasks.append(task)
    _write_tasks(path, tasks)
    return task


def list_tasks(path, include_done=False):
    """Return tasks by ascending ID; missing databases have no tasks."""
    return [task for task in _read_tasks(path) if include_done or not task["done"]]


def complete_task(path, task_id):
    """Complete a positive integer ID, succeeding again if already done."""
    if type(task_id) is not int or task_id < 1:
        raise TaskStoreError("Task ID must be a positive integer")
    tasks = _read_tasks(path)
    for task in tasks:
        if task["id"] == task_id:
            if not task["done"]:
                task["done"] = True
                _write_tasks(path, tasks)
            return task
    raise TaskStoreError("Task ID {} not found".format(task_id))
