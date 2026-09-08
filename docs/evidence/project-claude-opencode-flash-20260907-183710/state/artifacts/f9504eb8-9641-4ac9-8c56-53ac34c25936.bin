"""Storage layer for Task Pocket: a tiny local task tracker.

Public API:
    add_task(path, title) -> dict
    list_tasks(path, include_done=False) -> list[dict]
    complete_task(path, task_id) -> dict

The database is a single JSON file: {"next_id": int, "tasks": [task, ...]}.
Each task is {"id": int, "title": str, "done": bool}.
Writes are atomic (write to a temp file in the same directory, then
os.replace). On any error, the database file is left byte-for-byte
unchanged.
"""

import json
import os
import tempfile

_EMPTY_DB = {"next_id": 1, "tasks": []}


def _load(path):
    if not os.path.exists(path):
        return dict(_EMPTY_DB)
    try:
        with open(path, "r", encoding="utf-8") as f:
            raw = f.read()
    except OSError as e:
        raise ValueError(f"cannot read database {path!r}: {e}") from e

    try:
        data = json.loads(raw)
    except json.JSONDecodeError as e:
        raise ValueError(f"database {path!r} contains malformed JSON: {e}") from e

    _validate_shape(data, path)
    return data


def _validate_shape(data, path):
    if not isinstance(data, dict):
        raise ValueError(f"database {path!r} has invalid shape: expected an object")
    if "next_id" not in data or "tasks" not in data:
        raise ValueError(
            f"database {path!r} has invalid shape: missing 'next_id' or 'tasks'"
        )
    if not isinstance(data["next_id"], int) or isinstance(data["next_id"], bool):
        raise ValueError(f"database {path!r} has invalid shape: 'next_id' must be an int")
    if not isinstance(data["tasks"], list):
        raise ValueError(f"database {path!r} has invalid shape: 'tasks' must be a list")

    seen_ids = set()
    for task in data["tasks"]:
        if not isinstance(task, dict):
            raise ValueError(f"database {path!r} has invalid shape: task entry is not an object")
        if set(task.keys()) != {"id", "title", "done"}:
            raise ValueError(f"database {path!r} has invalid shape: bad task keys {sorted(task.keys())}")
        if not isinstance(task["id"], int) or isinstance(task["id"], bool):
            raise ValueError(f"database {path!r} has invalid shape: task id must be an int")
        if not isinstance(task["title"], str):
            raise ValueError(f"database {path!r} has invalid shape: task title must be a string")
        if not isinstance(task["done"], bool):
            raise ValueError(f"database {path!r} has invalid shape: task done must be a bool")
        if task["id"] in seen_ids:
            raise ValueError(f"database {path!r} has invalid shape: duplicate task id {task['id']}")
        seen_ids.add(task["id"])


def _save(path, data):
    parent = os.path.dirname(path)
    if parent:
        os.makedirs(parent, exist_ok=True)
    else:
        parent = "."

    fd, tmp_path = tempfile.mkstemp(prefix=".task_store_", dir=parent)
    try:
        with os.fdopen(fd, "w", encoding="utf-8") as f:
            json.dump(data, f, indent=2, sort_keys=True)
            f.write("\n")
        os.replace(tmp_path, path)
    except BaseException:
        try:
            os.remove(tmp_path)
        except OSError:
            pass
        raise


def add_task(path, title):
    """Create a new task with the given title and persist it.

    Raises ValueError if title is blank/whitespace-only, or if the
    database is malformed/invalid.
    """
    if not isinstance(title, str):
        raise ValueError("title must be a string")
    trimmed = title.strip()
    if not trimmed:
        raise ValueError("title must be a nonempty string after trimming whitespace")

    data = _load(path)
    task = {"id": data["next_id"], "title": trimmed, "done": False}
    data["tasks"].append(task)
    data["next_id"] += 1
    _save(path, data)
    return dict(task)


def list_tasks(path, include_done=False):
    """Return tasks sorted by id. Missing database yields []."""
    if not os.path.exists(path):
        return []
    data = _load(path)
    tasks = data["tasks"]
    if not include_done:
        tasks = [t for t in tasks if not t["done"]]
    return [dict(t) for t in sorted(tasks, key=lambda t: t["id"])]


def complete_task(path, task_id):
    """Mark the task with the given id as done and persist it.

    Idempotent: completing an already-done task succeeds and returns it
    unchanged. Raises ValueError if the id is not an int or no task
    with that id exists, or if the database is malformed/invalid.
    """
    if not isinstance(task_id, int) or isinstance(task_id, bool):
        raise ValueError("task_id must be an int")

    data = _load(path)
    for task in data["tasks"]:
        if task["id"] == task_id:
            if not task["done"]:
                task["done"] = True
                _save(path, data)
            return dict(task)

    raise ValueError(f"no task with id {task_id}")
