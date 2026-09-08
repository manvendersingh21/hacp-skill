"""Unit and subprocess-driven tests for task_cli.py and task_store.py."""
import json
import os
import subprocess
import sys
import tempfile
import unittest
from unittest import mock

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
CLI = os.path.join(ROOT, "task_cli.py")

if ROOT not in sys.path:
    sys.path.insert(0, ROOT)

import task_store  # noqa: E402  (path must be set up first)


def run_cli(args):
    return subprocess.run(
        [sys.executable, CLI, *args],
        capture_output=True,
        text=True,
    )


class TestCliAdd(unittest.TestCase):
    def test_add_returns_task_json(self):
        with tempfile.TemporaryDirectory() as d:
            db = os.path.join(d, "tasks.json")
            result = run_cli(["--db", db, "add", "buy milk"])
            self.assertEqual(result.returncode, 0, result.stderr)
            task = json.loads(result.stdout)
            self.assertEqual(task, {"id": 1, "title": "buy milk", "done": False})

    def test_add_increments_id(self):
        with tempfile.TemporaryDirectory() as d:
            db = os.path.join(d, "tasks.json")
            run_cli(["--db", db, "add", "first"])
            result = run_cli(["--db", db, "add", "second"])
            task = json.loads(result.stdout)
            self.assertEqual(task["id"], 2)

    def test_add_blank_title_fails(self):
        with tempfile.TemporaryDirectory() as d:
            db = os.path.join(d, "tasks.json")
            result = run_cli(["--db", db, "add", ""])
            self.assertNotEqual(result.returncode, 0)
            self.assertEqual(result.stdout, "")
            self.assertNotIn("Traceback", result.stderr)
            self.assertTrue(result.stderr.strip())

    def test_add_whitespace_title_fails(self):
        with tempfile.TemporaryDirectory() as d:
            db = os.path.join(d, "tasks.json")
            result = run_cli(["--db", db, "add", "   "])
            self.assertNotEqual(result.returncode, 0)
            self.assertEqual(result.stdout, "")

    def test_add_creates_parent_dirs(self):
        with tempfile.TemporaryDirectory() as d:
            db = os.path.join(d, "nested", "sub", "tasks.json")
            result = run_cli(["--db", db, "add", "task"])
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertTrue(os.path.exists(db))


class TestCliList(unittest.TestCase):
    def test_missing_db_lists_empty(self):
        with tempfile.TemporaryDirectory() as d:
            db = os.path.join(d, "does_not_exist.json")
            result = run_cli(["--db", db, "list"])
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertEqual(json.loads(result.stdout), [])

    def test_list_default_excludes_done(self):
        with tempfile.TemporaryDirectory() as d:
            db = os.path.join(d, "tasks.json")
            run_cli(["--db", db, "add", "open task"])
            run_cli(["--db", db, "add", "closed task"])
            run_cli(["--db", db, "done", "2"])
            result = run_cli(["--db", db, "list"])
            tasks = json.loads(result.stdout)
            self.assertEqual([t["id"] for t in tasks], [1])

    def test_list_all_includes_done(self):
        with tempfile.TemporaryDirectory() as d:
            db = os.path.join(d, "tasks.json")
            run_cli(["--db", db, "add", "open task"])
            run_cli(["--db", db, "add", "closed task"])
            run_cli(["--db", db, "done", "2"])
            result = run_cli(["--db", db, "list", "--all"])
            tasks = json.loads(result.stdout)
            self.assertEqual([t["id"] for t in tasks], [1, 2])

    def test_list_sorted_by_id(self):
        with tempfile.TemporaryDirectory() as d:
            db = os.path.join(d, "tasks.json")
            for title in ("a", "b", "c"):
                run_cli(["--db", db, "add", title])
            result = run_cli(["--db", db, "list", "--all"])
            tasks = json.loads(result.stdout)
            self.assertEqual([t["id"] for t in tasks], sorted(t["id"] for t in tasks))

    def test_stdout_only_json_on_success(self):
        with tempfile.TemporaryDirectory() as d:
            db = os.path.join(d, "tasks.json")
            result = run_cli(["--db", db, "add", "solo"])
            lines = result.stdout.splitlines()
            self.assertEqual(len(lines), 1)
            json.loads(lines[0])


class TestCliDone(unittest.TestCase):
    def test_done_marks_task(self):
        with tempfile.TemporaryDirectory() as d:
            db = os.path.join(d, "tasks.json")
            run_cli(["--db", db, "add", "task"])
            result = run_cli(["--db", db, "done", "1"])
            task = json.loads(result.stdout)
            self.assertEqual(task, {"id": 1, "title": "task", "done": True})

    def test_done_idempotent(self):
        with tempfile.TemporaryDirectory() as d:
            db = os.path.join(d, "tasks.json")
            run_cli(["--db", db, "add", "task"])
            first = run_cli(["--db", db, "done", "1"])
            second = run_cli(["--db", db, "done", "1"])
            self.assertEqual(second.returncode, 0, second.stderr)
            self.assertEqual(json.loads(first.stdout), json.loads(second.stdout))

    def test_done_missing_id_fails(self):
        with tempfile.TemporaryDirectory() as d:
            db = os.path.join(d, "tasks.json")
            run_cli(["--db", db, "add", "task"])
            result = run_cli(["--db", db, "done", "999"])
            self.assertNotEqual(result.returncode, 0)
            self.assertEqual(result.stdout, "")
            self.assertNotIn("Traceback", result.stderr)

    def test_done_malformed_id_fails(self):
        with tempfile.TemporaryDirectory() as d:
            db = os.path.join(d, "tasks.json")
            run_cli(["--db", db, "add", "task"])
            result = run_cli(["--db", db, "done", "abc"])
            self.assertNotEqual(result.returncode, 0)
            self.assertEqual(result.stdout, "")
            self.assertNotIn("Traceback", result.stderr)


class TestPersistenceAndCorruption(unittest.TestCase):
    def test_persistence_across_processes(self):
        with tempfile.TemporaryDirectory() as d:
            db = os.path.join(d, "tasks.json")
            run_cli(["--db", db, "add", "persisted"])
            result = run_cli(["--db", db, "list"])
            tasks = json.loads(result.stdout)
            self.assertEqual(len(tasks), 1)
            self.assertEqual(tasks[0]["title"], "persisted")

    def test_malformed_json_db_fails_and_unchanged(self):
        with tempfile.TemporaryDirectory() as d:
            db = os.path.join(d, "tasks.json")
            with open(db, "wb") as f:
                f.write(b"{not valid json")
            with open(db, "rb") as f:
                original = f.read()
            result = run_cli(["--db", db, "list"])
            self.assertNotEqual(result.returncode, 0)
            self.assertNotIn("Traceback", result.stderr)
            with open(db, "rb") as f:
                self.assertEqual(f.read(), original)

    def test_invalid_shape_db_fails_and_unchanged(self):
        with tempfile.TemporaryDirectory() as d:
            db = os.path.join(d, "tasks.json")
            with open(db, "w") as f:
                json.dump({"unexpected": "shape"}, f)
            with open(db, "rb") as f:
                original = f.read()
            result = run_cli(["--db", db, "add", "x"])
            self.assertNotEqual(result.returncode, 0)
            self.assertNotIn("Traceback", result.stderr)
            with open(db, "rb") as f:
                self.assertEqual(f.read(), original)


class TestTaskStoreEdgeCases(unittest.TestCase):
    def test_complete_task_rejects_bool(self):
        with tempfile.TemporaryDirectory() as d:
            path = os.path.join(d, "tasks.json")
            task_store.add_task(path, "sample")
            with self.assertRaises(ValueError):
                task_store.complete_task(path, True)

    def test_complete_task_rejects_nonpositive(self):
        with tempfile.TemporaryDirectory() as d:
            path = os.path.join(d, "tasks.json")
            task_store.add_task(path, "sample")
            with self.assertRaises(ValueError):
                task_store.complete_task(path, 0)
            with self.assertRaises(ValueError):
                task_store.complete_task(path, -1)

    def test_atomic_write_failure_preserves_original(self):
        with tempfile.TemporaryDirectory() as d:
            path = os.path.join(d, "tasks.json")
            task_store.add_task(path, "first")
            with open(path, "rb") as f:
                original = f.read()
            with mock.patch("os.replace", side_effect=OSError("boom")):
                with self.assertRaises(OSError):
                    task_store.add_task(path, "second")
            with open(path, "rb") as f:
                after = f.read()
            self.assertEqual(original, after)


if __name__ == "__main__":
    unittest.main()
