"""End-to-end unittest coverage for task_cli.py.

Every test drives the real CLI in a fresh subprocess against a temporary
database, so persistence and multi-process behavior are exercised.
"""

import glob
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


class TaskCliTestCase(unittest.TestCase):
    def setUp(self):
        self._tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self._tmp.cleanup)
        self.db = os.path.join(self._tmp.name, "tasks.db")

    def run_cli(self, *args, db=None):
        cmd = [sys.executable, CLI, "--db", db if db is not None else self.db]
        cmd.extend(args)
        return subprocess.run(cmd, capture_output=True, text=True, cwd=ROOT, timeout=60)

    def assert_ok(self, proc):
        self.assertEqual(proc.returncode, 0, proc.stderr)
        self.assertEqual(proc.stderr, "")
        self.assertNotIn("Traceback", proc.stderr)
        return proc

    def assert_json_stdout(self, proc):
        self.assert_ok(proc)
        return json.loads(proc.stdout)

    def assert_fails_cleanly(self, proc):
        self.assertNotEqual(proc.returncode, 0)
        self.assertNotIn("Traceback", proc.stderr)
        self.assertTrue(proc.stderr.strip(), "expected a useful stderr message")
        self.assertEqual(proc.stdout, "")
        return proc

    # --- happy paths -----------------------------------------------------

    def test_add_prints_created_task(self):
        proc = self.run_cli("add", "  write spec  ")
        task = self.assert_json_stdout(proc)
        self.assertEqual(task, {"id": 1, "title": "write spec", "done": False})

    def test_list_missing_db_is_empty_array(self):
        self.assertEqual(self.assert_json_stdout(self.run_cli("list")), [])

    def test_list_sorted_by_id_and_persists_across_processes(self):
        self.assert_ok(self.run_cli("add", "second thought"))
        self.assert_ok(self.run_cli("add", "first thing"))
        tasks = self.assert_json_stdout(self.run_cli("list"))
        self.assertEqual([t["id"] for t in tasks], [1, 2])
        self.assertEqual([t["title"] for t in tasks], ["second thought", "first thing"])
        for task in tasks:
            self.assertEqual(set(task), {"id", "title", "done"})
            self.assertFalse(task["done"])

    def test_list_sorts_hand_seeded_reversed_ids(self):
        seeded = json.dumps([
            {"id": 3, "title": "third", "done": False},
            {"id": 1, "title": "first", "done": False},
            {"id": 2, "title": "second", "done": False},
        ]).encode("utf-8")
        with open(self.db, "wb") as handle:
            handle.write(seeded)
        tasks = self.assert_json_stdout(self.run_cli("list"))
        self.assertEqual([(t["id"], t["title"]) for t in tasks],
                         [(1, "first"), (2, "second"), (3, "third")])
        fourth = self.assert_json_stdout(self.run_cli("add", "fourth"))
        self.assertEqual(fourth["id"], 4)

    def test_done_completes_and_list_filters(self):
        self.assert_ok(self.run_cli("add", "ship it"))
        done = self.assert_json_stdout(self.run_cli("done", "1"))
        self.assertEqual(done, {"id": 1, "title": "ship it", "done": True})
        self.assertEqual(self.assert_json_stdout(self.run_cli("list")), [])
        all_tasks = self.assert_json_stdout(self.run_cli("list", "--all"))
        self.assertEqual(all_tasks, [{"id": 1, "title": "ship it", "done": True}])

    def test_done_is_idempotent(self):
        self.assert_ok(self.run_cli("add", "once only"))
        first = self.assert_json_stdout(self.run_cli("done", "1"))
        second = self.assert_json_stdout(self.run_cli("done", "1"))
        self.assertEqual(first, second)
        self.assertTrue(second["done"])

    def test_ids_increase_and_are_never_reused(self):
        self.assert_ok(self.run_cli("add", "a"))
        self.assert_ok(self.run_cli("add", "b"))
        self.assert_ok(self.run_cli("done", "1"))
        third = self.assert_json_stdout(self.run_cli("add", "c"))
        self.assertEqual(third["id"], 3)
        ids = [t["id"] for t in self.assert_json_stdout(self.run_cli("list", "--all"))]
        self.assertEqual(ids, [1, 2, 3])

    def test_add_creates_missing_parent_directories(self):
        nested = os.path.join(self._tmp.name, "deep", "deeper", "tasks.db")
        proc = self.run_cli("add", "nested", db=nested)
        self.assert_json_stdout(proc)
        self.assertTrue(os.path.isfile(nested))

    # --- failure paths ---------------------------------------------------

    def test_add_blank_title_fails_without_db_creation(self):
        self.assert_fails_cleanly(self.run_cli("add", "   "))
        self.assertFalse(os.path.exists(self.db))

    def test_done_missing_id_fails(self):
        self.assert_ok(self.run_cli("add", "exists"))
        self.assert_fails_cleanly(self.run_cli("done", "99"))

    def test_done_malformed_id_fails(self):
        self.assert_ok(self.run_cli("add", "target"))
        for bad in ("abc", "1.5", "0", "-3", ""):
            with self.subTest(bad=bad):
                self.assert_fails_cleanly(self.run_cli("done", bad))

    def test_malformed_json_db_fails_and_is_preserved_byte_for_byte(self):
        corrupt = b"not json at all{"
        with open(self.db, "wb") as handle:
            handle.write(corrupt)
        for command in (["list"], ["add", "never lands"], ["done", "1"]):
            with self.subTest(command=command):
                self.assert_fails_cleanly(self.run_cli(*command))
                with open(self.db, "rb") as handle:
                    self.assertEqual(handle.read(), corrupt)

    def test_invalid_db_shapes_fail_and_are_preserved_byte_for_byte(self):
        bad_shapes = [
            b'{"tasks": [], "next_id": 1}',
            b"{}",
            b'[{"id": 1, "title": "missing done"}]',
            b'[{"id": 1, "title": "extra", "done": false, "priority": 1}]',
            b'[{"id": "1", "title": "string id", "done": false}]',
            b'[{"id": true, "title": "bool id", "done": false}]',
            b'[{"id": 1, "title": 7, "done": false}]',
            b'[{"id": 1, "title": " padded ", "done": false}]',
            b'[{"id": 1, "title": "done not bool", "done": "no"}]',
            b'[{"id": 1, "title": "first", "done": false}, {"id": 1, "title": "dupe", "done": false}]',
            b'["not a task"]',
        ]
        for bad in bad_shapes:
            with self.subTest(db=bad.decode("utf-8")):
                with open(self.db, "wb") as handle:
                    handle.write(bad)
                for command in (["list"], ["add", "never lands"], ["done", "1"]):
                    self.assert_fails_cleanly(self.run_cli(*command))
                with open(self.db, "rb") as handle:
                    self.assertEqual(handle.read(), bad)

    def test_atomic_write_failure_preserves_db_and_leaves_no_temp_files(self):
        import task_store

        self.assert_ok(self.run_cli("add", "original"))
        with open(self.db, "rb") as handle:
            original = handle.read()
        with mock.patch.object(task_store.os, "replace", side_effect=OSError("disk on fire")):
            with self.assertRaises(task_store.TaskStoreError):
                task_store.add_task(self.db, "should not land")
        with open(self.db, "rb") as handle:
            self.assertEqual(handle.read(), original)
        leftovers = glob.glob(os.path.join(self._tmp.name, ".task-pocket-*"))
        self.assertEqual(leftovers, [])
        tasks = self.assert_json_stdout(self.run_cli("list"))
        self.assertEqual([t["title"] for t in tasks], ["original"])

    def test_stdout_is_only_json(self):
        self.assert_ok(self.run_cli("add", "clean output"))
        lines = [line for line in self.stdout_lines(self.run_cli("list")) if line]
        self.assertEqual(len(lines), 1)
        json.loads(lines[0])

    @staticmethod
    def stdout_lines(proc):
        return proc.stdout.splitlines()


if __name__ == "__main__":
    unittest.main()
