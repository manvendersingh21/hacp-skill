"""Validated JSON persistence for Task Pocket (single writer, standard library)."""

import json
import os
import tempfile
from pathlib import Path


class TaskStoreError(ValueError):
    """An invalid task request or database that cannot safely be used."""


def _positive_integer(value):
    return type(value) is int and value > 0


def _object(pairs):
    result = {}
    for key, value in pairs:
        if key in result:
            raise TaskStoreError("Invalid database: duplicate JSON key {!r}".format(key))
        result[key] = value
    return result


def _invalid_constant(value):
    raise TaskStoreError("Invalid database: non-JSON value {}".format(value))


def _validate(data):
    if not isinstance(data, dict) or set(data) != {"tasks", "next_id"}:
        raise TaskStoreError("Invalid database: expected tasks and next_id fields")
    if not isinstance(data["tasks"], list) or not _positive_integer(data["next_id"]):
        raise TaskStoreError("Invalid database: tasks must be an array and next_id a positive integer")
    seen = set()
    for task in data["tasks"]:
        if not isinstance(task, dict) or set(task) != {"id", "title", "done"}:
            raise TaskStoreError("Invalid database: each task needs id, title, and done fields")
        if not _positive_integer(task["id"]) or task["id"] in seen:
            raise TaskStoreError("Invalid database: task IDs must be unique positive integers")
        title = task["title"]
        if not isinstance(title, str) or not title.strip() or title != title.strip():
            raise TaskStoreError("Invalid database: task titles must be trimmed nonempty strings")
        if type(task["done"]) is not bool:
            raise TaskStoreError("Invalid database: task done must be boolean")
        seen.add(task["id"])
    if seen and data["next_id"] <= max(seen):
        raise TaskStoreError("Invalid database: next_id must exceed every task ID")
    return data


def _read(path):
    try:
        with Path(path).open("r", encoding="utf-8") as stream:
            data = json.load(stream, object_pairs_hook=_object,
                             parse_constant=_invalid_constant)
    except FileNotFoundError:
        return {"tasks": [], "next_id": 1}
    except (ValueError, UnicodeError, RecursionError) as error:
        raise TaskStoreError("Invalid database: {}".format(error)) from None
    return _validate(data)


def _write(path, data):
    destination = Path(path)
    destination.parent.mkdir(parents=True, exist_ok=True)
    temporary = None
    try:
        # A temporary sibling guarantees replacement stays on the same filesystem.
        with tempfile.NamedTemporaryFile(mode="w", encoding="utf-8",
                                         dir=str(destination.parent),
                                         prefix=".task-pocket-", suffix=".tmp",
                                         delete=False) as stream:
            temporary = stream.name
            json.dump(data, stream, ensure_ascii=True, allow_nan=False)
            stream.write("\n")
            stream.flush()
            os.fsync(stream.fileno())
        os.replace(temporary, str(destination))
    finally:
        if temporary is not None:
            try:
                os.unlink(temporary)
            except FileNotFoundError:
                pass


def add_task(path, title):
    """Add a trimmed, nonempty title and return the newly persisted task."""
    if not isinstance(title, str) or not title.strip():
        raise TaskStoreError("Task title must be a nonempty string")
    data = _read(path)
    task = {"id": data["next_id"], "title": title.strip(), "done": False}
    data["tasks"].append(task)
    data["next_id"] += 1
    _write(path, data)
    return dict(task)


def list_tasks(path, include_done=False):
    """Return tasks ordered by ID; a missing database is an empty collection."""
    data = _read(path)
    return [dict(task) for task in sorted(data["tasks"], key=lambda task: task["id"])
            if include_done or not task["done"]]


def complete_task(path, task_id):
    """Complete a positive integer task ID, succeeding on repeated completion."""
    if not _positive_integer(task_id):
        raise TaskStoreError("Task ID must be a positive integer")
    data = _read(path)
    for task in data["tasks"]:
        if task["id"] == task_id:
            if not task["done"]:
                task["done"] = True
                _write(path, data)
            return dict(task)
    raise TaskStoreError("Task ID {} not found".format(task_id))
