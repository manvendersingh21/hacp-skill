import json
import os
import subprocess
import sys
import tempfile
import unittest
from unittest import mock

import task_store

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
CLI = os.path.join(ROOT, "task_cli.py")


def run_cli(*args, db=None):
    cmd = [sys.executable, CLI]
    if db is not None:
        cmd += ["--db", db]
    cmd += list(args)
    return subprocess.run(cmd, capture_output=True, text=True)


class TaskCliTest(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.db = os.path.join(self.tmp.name, "tasks.json")

    def read_db(self):
        with open(self.db, "rb") as fh:
            return fh.read()

    def test_add_prints_created_task_json(self):
        result = run_cli("add", "  Buy milk  ", db=self.db)
        self.assertEqual(result.returncode, 0, result.stderr)
        task = json.loads(result.stdout)
        self.assertEqual(task, {"id": 1, "title": "Buy milk", "done": False})

    def test_add_creates_missing_parent_directories(self):
        nested = os.path.join(self.tmp.name, "a", "b", "tasks.json")
        result = run_cli("add", "deep", db=nested)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertTrue(os.path.exists(nested))

    def test_list_missing_db_prints_empty_array(self):
        missing = os.path.join(self.tmp.name, "nope", "tasks.json")
        result = run_cli("list", db=missing)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(json.loads(result.stdout), [])

    def test_data_persists_across_processes(self):
        run_cli("add", "one", db=self.db)
        run_cli("add", "two", db=self.db)
        result = run_cli("list", db=self.db)
        tasks = json.loads(result.stdout)
        self.assertEqual([t["id"] for t in tasks], [1, 2])
        self.assertEqual([t["title"] for t in tasks], ["one", "two"])

    def test_list_sorted_by_id(self):
        for title in ("alpha", "beta", "gamma"):
            run_cli("add", title, db=self.db)
        run_cli("done", "2", db=self.db)
        run_cli("add", "delta", db=self.db)
        result = run_cli("list", "--all", db=self.db)
        self.assertEqual(result.returncode, 0, result.stderr)
        tasks = json.loads(result.stdout)
        self.assertEqual([t["id"] for t in tasks], [1, 2, 3, 4])

    def test_list_excludes_done_and_all_includes(self):
        run_cli("add", "keep", db=self.db)
        run_cli("add", "finish", db=self.db)
        run_cli("done", "2", db=self.db)
        open_tasks = json.loads(run_cli("list", db=self.db).stdout)
        all_tasks = json.loads(run_cli("list", "--all", db=self.db).stdout)
        self.assertEqual([t["id"] for t in open_tasks], [1])
        self.assertEqual([t["id"] for t in all_tasks], [1, 2])
        self.assertTrue(all_tasks[1]["done"])

    def test_done_prints_completed_task_json(self):
        run_cli("add", "task", db=self.db)
        result = run_cli("done", "1", db=self.db)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(
            json.loads(result.stdout), {"id": 1, "title": "task", "done": True}
        )

    def test_done_is_idempotent(self):
        run_cli("add", "task", db=self.db)
        first = run_cli("done", "1", db=self.db)
        second = run_cli("done", "1", db=self.db)
        self.assertEqual(first.returncode, 0, first.stderr)
        self.assertEqual(second.returncode, 0, second.stderr)
        self.assertEqual(json.loads(first.stdout), json.loads(second.stdout))

    def test_ids_are_never_reused(self):
        run_cli("add", "one", db=self.db)
        run_cli("add", "two", db=self.db)
        run_cli("done", "1", db=self.db)
        result = run_cli("add", "three", db=self.db)
        self.assertEqual(json.loads(result.stdout)["id"], 3)

    def test_missing_id_fails_without_traceback_and_keeps_db(self):
        run_cli("add", "only", db=self.db)
        before = self.read_db()
        result = run_cli("done", "99", db=self.db)
        self.assertNotEqual(result.returncode, 0)
        self.assertNotIn("Traceback", result.stderr)
        self.assertIn("error", result.stderr)
        self.assertTrue(result.stderr.strip())
        self.assertEqual(self.read_db(), before)

    def test_malformed_id_fails_cleanly(self):
        run_cli("add", "only", db=self.db)
        for bad in ("abc", "1.5", ""):
            result = run_cli("done", bad, db=self.db)
            self.assertNotEqual(result.returncode, 0, bad)
            self.assertNotIn("Traceback", result.stderr)
            self.assertIn("error", result.stderr)

    def test_blank_title_fails_cleanly(self):
        result = run_cli("add", "   ", db=self.db)
        self.assertNotEqual(result.returncode, 0)
        self.assertNotIn("Traceback", result.stderr)
        self.assertIn("error", result.stderr)

    def test_malformed_json_db_fails_and_bytes_unchanged(self):
        with open(self.db, "w", encoding="utf-8") as fh:
            fh.write("{oops this is not json")
        before = self.read_db()
        for cmd in (("list",), ("add", "x"), ("done", "1")):
            result = run_cli(*cmd, db=self.db)
            self.assertNotEqual(result.returncode, 0, cmd)
            self.assertNotIn("Traceback", result.stderr)
        self.assertEqual(self.read_db(), before)

    def test_invalid_schema_db_fails_and_bytes_unchanged(self):
        payload = json.dumps({"evil": True}).encode("utf-8")
        with open(self.db, "wb") as fh:
            fh.write(payload)
        result = run_cli("list", db=self.db)
        self.assertNotEqual(result.returncode, 0)
        self.assertNotIn("Traceback", result.stderr)
        self.assertEqual(self.read_db(), payload)

    def test_successful_stdout_contains_only_json(self):
        add_result = run_cli("add", "solo", db=self.db)
        list_result = run_cli("list", db=self.db)
        done_result = run_cli("done", "1", db=self.db)
        for result in (add_result, list_result, done_result):
            self.assertEqual(result.returncode, 0, result.stderr)
            parsed = json.loads(result.stdout)
            self.assertEqual(result.stdout, json.dumps(parsed) + "\n")

    def test_missing_db_flag_is_usage_error(self):
        result = subprocess.run(
            [sys.executable, CLI, "list"], capture_output=True, text=True
        )
        self.assertEqual(result.returncode, 2)
        self.assertIn("--db", result.stderr)


    def test_list_sorts_regardless_of_on_disk_order(self):
        payload = json.dumps(
            {
                "next_id": 4,
                "tasks": [
                    {"id": 3, "title": "c", "done": False},
                    {"id": 1, "title": "a", "done": True},
                    {"id": 2, "title": "b", "done": False},
                ],
            }
        ).encode("utf-8")
        with open(self.db, "wb") as fh:
            fh.write(payload)
        open_result = run_cli("list", db=self.db)
        all_result = run_cli("list", "--all", db=self.db)
        self.assertEqual(open_result.returncode, 0, open_result.stderr)
        self.assertEqual(all_result.returncode, 0, all_result.stderr)
        self.assertEqual(
            [t["id"] for t in json.loads(open_result.stdout)], [2, 3]
        )
        self.assertEqual(
            [t["id"] for t in json.loads(all_result.stdout)], [1, 2, 3]
        )

    def test_add_appends_after_shuffled_tasks_and_next_id_advances(self):
        payload = json.dumps(
            {
                "next_id": 4,
                "tasks": [
                    {"id": 3, "title": "c", "done": False},
                    {"id": 1, "title": "a", "done": False},
                ],
            }
        )
        with open(self.db, "w", encoding="utf-8") as fh:
            fh.write(payload)
        result = run_cli("add", "d", db=self.db)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(json.loads(result.stdout)["id"], 4)
        with open(self.db, encoding="utf-8") as fh:
            self.assertEqual(json.load(fh)["next_id"], 5)

    def test_invalid_nested_schema_fails_and_bytes_unchanged(self):
        cases = {
            "duplicate_ids": {
                "next_id": 3,
                "tasks": [
                    {"id": 1, "title": "a", "done": False},
                    {"id": 1, "title": "b", "done": False},
                ],
            },
            "bool_id": {
                "next_id": 2,
                "tasks": [
                    {"id": True, "title": "a", "done": False},
                ],
            },
            "next_id_le_max": {
                "next_id": 1,
                "tasks": [
                    {"id": 2, "title": "a", "done": False},
                ],
            },
            "nonbool_done": {
                "next_id": 2,
                "tasks": [
                    {"id": 1, "title": "a", "done": "no"},
                ],
            },
            "nonstring_title": {
                "next_id": 2,
                "tasks": [
                    {"id": 1, "title": 5, "done": False},
                ],
            },
            "blank_title": {
                "next_id": 2,
                "tasks": [
                    {"id": 1, "title": "   ", "done": False},
                ],
            },
            "missing_tasks": {"next_id": 1},
        }
        for name, doc in cases.items():
            with self.subTest(name):
                payload = json.dumps(doc).encode("utf-8")
                with open(self.db, "wb") as fh:
                    fh.write(payload)
                before = self.read_db()
                for cmd in (("list",), ("add", "x"), ("done", "1")):
                    result = run_cli(*cmd, db=self.db)
                    self.assertNotEqual(result.returncode, 0, cmd)
                    self.assertNotIn("Traceback", result.stderr)
                self.assertEqual(self.read_db(), before)

    def test_invalid_utf8_db_fails_and_bytes_unchanged(self):
        payload = b'{"next_id": 1, "tasks": [{"id": 1, "title": "\xff\xfe", "done": false}]}'
        with open(self.db, "wb") as fh:
            fh.write(payload)
        result = run_cli("list", db=self.db)
        self.assertNotEqual(result.returncode, 0)
        self.assertNotIn("Traceback", result.stderr)
        self.assertEqual(self.read_db(), payload)

    def test_failed_atomic_write_keeps_original_db_and_no_temp(self):
        run_cli("add", "first", db=self.db)
        before = self.read_db()
        parent = os.path.dirname(self.db)
        real_replace = os.replace

        def boom(src, dst, *args, **kwargs):
            raise OSError("injected os.replace failure")

        with mock.patch("task_store.os.replace", side_effect=boom):
            with self.assertRaises(task_store.TaskStoreError):
                task_store.add_task(self.db, "second")
        self.assertEqual(self.read_db(), before)
        leftover = [
            name
            for name in os.listdir(parent)
            if name != os.path.basename(self.db)
        ]
        self.assertEqual(leftover, [])
        self.assertIs(os.replace, real_replace)


if __name__ == "__main__":
    unittest.main()
