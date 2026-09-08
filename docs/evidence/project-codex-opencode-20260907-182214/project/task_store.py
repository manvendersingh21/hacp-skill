"""Validated JSON storage for Task Pocket (Python 3.9+, standard library only)."""

import json
import os
from pathlib import Path
import tempfile


class TaskStoreError(Exception):
    """A user-facing input, database, or filesystem error."""


def _positive_int(value):
    return type(value) is int and value > 0


def _validate(data):
    if not isinstance(data, dict) or set(data) != {"next_id", "tasks"}:
        raise TaskStoreError("Invalid database shape: expected next_id and tasks")
    if not _positive_int(data["next_id"]) or not isinstance(data["tasks"], list):
        raise TaskStoreError("Invalid database shape: invalid next_id or tasks")
    seen = set()
    for task in data["tasks"]:
        if not isinstance(task, dict) or set(task) != {"id", "title", "done"}:
            raise TaskStoreError("Invalid database shape: invalid task fields")
        task_id = task["id"]
        if not _positive_int(task_id) or task_id in seen:
            raise TaskStoreError("Invalid database shape: task IDs must be unique positive integers")
        if task_id >= data["next_id"]:
            raise TaskStoreError("Invalid database shape: next_id must exceed all task IDs")
        title = task["title"]
        if not isinstance(title, str) or not title.strip() or title != title.strip():
            raise TaskStoreError("Invalid database shape: titles must be trimmed nonempty strings")
        if type(task["done"]) is not bool:
            raise TaskStoreError("Invalid database shape: done must be boolean")
        seen.add(task_id)
    return data


def _object(pairs):
    result = {}
    for key, value in pairs:
        if key in result:
            raise TaskStoreError("Invalid database shape: duplicate JSON key " + key)
        result[key] = value
    return result


def _load(path):
    try:
        with path.open("r", encoding="utf-8") as stream:
            data = json.load(stream, object_pairs_hook=_object)
    except FileNotFoundError:
        return {"next_id": 1, "tasks": []}
    except (ValueError, UnicodeError, RecursionError) as exc:
        raise TaskStoreError("Invalid database JSON: " + str(exc)) from exc
    except OSError as exc:
        raise TaskStoreError("Cannot read database: " + str(exc)) from exc
    return _validate(data)


def _save(path, data):
    """Write beside the destination, then atomically replace the database."""
    temporary = None
    try:
        path.parent.mkdir(parents=True, exist_ok=True)
        with tempfile.NamedTemporaryFile(
            mode="w", encoding="utf-8", dir=str(path.parent),
            prefix=".task-pocket-", suffix=".tmp", delete=False
        ) as stream:
            temporary = stream.name
            json.dump(data, stream, ensure_ascii=True, indent=2)
            stream.write("\n")
            stream.flush()
            os.fsync(stream.fileno())
        os.replace(temporary, path)
        temporary = None
    except (OSError, ValueError, UnicodeError) as exc:
        raise TaskStoreError("Cannot write database: " + str(exc)) from exc
    finally:
        if temporary is not None:
            try:
                os.unlink(temporary)
            except OSError:
                pass


def add_task(path, title):
    """Persist and return a new task, trimming its nonempty string title."""
    if not isinstance(title, str) or not title.strip():
        raise TaskStoreError("Task title must be a nonempty string")
    path = Path(path)
    data = _load(path)
    task = {"id": data["next_id"], "title": title.strip(), "done": False}
    data["tasks"].append(task)
    data["next_id"] += 1
    _save(path, data)
    return dict(task)


def list_tasks(path, include_done=False):
    """Return tasks sorted by ID; a missing database is an empty list."""
    data = _load(Path(path))
    return [dict(task) for task in sorted(data["tasks"], key=lambda task: task["id"])
            if include_done or not task["done"]]


def complete_task(path, task_id):
    """Complete an existing positive integer ID, idempotently."""
    if not _positive_int(task_id):
        raise TaskStoreError("Task ID must be a positive integer")
    path = Path(path)
    data = _load(path)
    for task in data["tasks"]:
        if task["id"] == task_id:
            if not task["done"]:
                task["done"] = True
                _save(path, data)
            return dict(task)
    raise TaskStoreError("Task ID {} not found".format(task_id))
