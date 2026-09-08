"""End-to-end tests for the Task Pocket CLI (task_cli.py + task_store.py)."""

import json
import os
import subprocess
import sys
import tempfile
import unittest

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
CLI = os.path.join(ROOT, "task_cli.py")


class TaskCliTestCase(unittest.TestCase):
    """Runs the CLI as a fresh subprocess so persistence and exit codes
    are exercised exactly as a user would see them."""

    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.db = os.path.join(self.tmp.name, "tasks.json")

    # -- helpers ---------------------------------------------------------

    def run_cli(self, *args, db=None):
        return subprocess.run(
            [sys.executable, CLI, "--db", db or self.db] + list(args),
            capture_output=True,
            text=True,
        )

    def assert_ok(self, proc):
        self.assertEqual(
            proc.returncode, 0,
            "expected success, got stderr: %s" % proc.stderr,
        )
        self.assertNotIn("Traceback", proc.stderr)
        return proc

    def assert_fails(self, proc):
        self.assertNotEqual(proc.returncode, 0)
        self.assertTrue(proc.stderr.strip(), "expected a stderr message")
        self.assertNotIn("Traceback", proc.stderr)
        self.assertEqual(proc.stdout, "", "stdout must stay empty on failure")
        return proc

    def add(self, title):
        proc = self.assert_ok(self.run_cli("add", title))
        return json.loads(proc.stdout)

    # -- add -------------------------------------------------------------

    def test_add_prints_created_task_as_json(self):
        proc = self.assert_ok(self.run_cli("add", "  buy milk  "))
        self.assertEqual(
            json.loads(proc.stdout),
            {"id": 1, "title": "buy milk", "done": False},
        )
        self.assertEqual(json.loads(proc.stdout), json.loads(proc.stdout.strip()))

    def test_add_assigns_increasing_ids(self):
        first = self.add("one")
        second = self.add("two")
        self.assertEqual(first["id"], 1)
        self.assertEqual(second["id"], 2)

    def test_add_creates_missing_parent_directories(self):
        nested = os.path.join(self.tmp.name, "a", "b", "tasks.json")
        proc = self.run_cli("add", "deep", db=nested)
        self.assert_ok(proc)
        self.assertTrue(os.path.isfile(nested))
        self.assertEqual(json.loads(proc.stdout)["id"], 1)

    def test_add_blank_title_fails(self):
        self.assert_fails(self.run_cli("add", ""))

    def test_add_whitespace_title_fails(self):
        self.assert_fails(self.run_cli("add", "   \t "))

    # -- list ------------------------------------------------------------

    def test_list_missing_db_prints_empty_array(self):
        proc = self.assert_ok(self.run_cli("list"))
        self.assertEqual(json.loads(proc.stdout), [])
        self.assertFalse(os.path.exists(self.db), "list must not create the db")

    def test_list_sorted_by_id_and_hides_done(self):
        self.add("one")
        two = self.add("two")
        self.add("three")
        self.run_cli("done", str(two["id"]))
        open_tasks = json.loads(self.assert_ok(self.run_cli("list")).stdout)
        self.assertEqual([t["id"] for t in open_tasks], [1, 3])
        all_tasks = json.loads(self.assert_ok(self.run_cli("list", "--all")).stdout)
        self.assertEqual([t["id"] for t in all_tasks], [1, 2, 3])
        self.assertTrue(all_tasks[1]["done"])

    def test_list_output_is_json_array_of_task_dicts(self):
        self.add("solo")
        tasks = json.loads(self.assert_ok(self.run_cli("list")).stdout)
        self.assertIsInstance(tasks, list)
        self.assertEqual(
            tasks,
            [{"id": 1, "title": "solo", "done": False}],
        )

    # -- done ------------------------------------------------------------

    def test_done_prints_completed_task_as_json(self):
        task = self.add("write tests")
        proc = self.assert_ok(self.run_cli("done", str(task["id"])))
        self.assertEqual(
            json.loads(proc.stdout),
            {"id": 1, "title": "write tests", "done": True},
        )

    def test_done_is_idempotent(self):
        task = self.add("once")
        self.assert_ok(self.run_cli("done", str(task["id"])))
        proc = self.assert_ok(self.run_cli("done", str(task["id"])))
        self.assertEqual(json.loads(proc.stdout)["done"], True)
        # ids are never reused even after completion
        self.assertEqual(self.add("after")["id"], 2)

    def test_done_missing_id_fails(self):
        self.add("exists")
        self.assert_fails(self.run_cli("done", "99"))

    def test_done_malformed_id_fails(self):
        self.add("exists")
        self.assert_fails(self.run_cli("done", "abc"))
        self.assert_fails(self.run_cli("done", "1.5"))

    # -- persistence and database safety ----------------------------------

    def test_data_persists_across_processes(self):
        self.add("stays")
        self.run_cli("add", "around")
        self.run_cli("done", "1")
        tasks = json.loads(self.assert_ok(self.run_cli("list", "--all")).stdout)
        self.assertEqual(
            tasks,
            [
                {"id": 1, "title": "stays", "done": True},
                {"id": 2, "title": "around", "done": False},
            ],
        )

    def read_db_bytes(self):
        with open(self.db, "rb") as f:
            return f.read()

    def test_malformed_json_db_fails_and_stays_unchanged(self):
        with open(self.db, "w", encoding="utf-8") as f:
            f.write("{not valid json!!!")
        before = self.read_db_bytes()
        self.assert_fails(self.run_cli("list"))
        self.assert_fails(self.run_cli("add", "x"))
        self.assertEqual(self.read_db_bytes(), before)

    def test_invalid_db_shape_fails_and_stays_unchanged(self):
        bad_shapes = [
            "[1, 2, 3]",
            '{"foo": 1}',
            '{"next_id": "x", "tasks": []}',
            '{"next_id": 1, "tasks": {"a": 1}}',
            '{"next_id": 1, "tasks": [{"id": "a", "title": 1, "done": "x"}]}',
        ]
        for shape in bad_shapes:
            with open(self.db, "w", encoding="utf-8") as f:
                f.write(shape)
            before = self.read_db_bytes()
            self.assert_fails(self.run_cli("list"))
            self.assert_fails(self.run_cli("add", "x"))
            self.assert_fails(self.run_cli("done", "1"))
            self.assertEqual(
                self.read_db_bytes(), before,
                "db must stay byte-for-byte unchanged for shape %r" % shape,
            )

    def test_successful_stdout_is_only_json(self):
        proc = self.assert_ok(self.run_cli("add", "clean"))
        parsed = json.loads(proc.stdout)
        self.assertEqual(proc.stdout.strip(), json.dumps(parsed))
        self.assertEqual(proc.stdout.count("\n"), 1)


if __name__ == "__main__":
    unittest.main()
