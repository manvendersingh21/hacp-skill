import os
import sys
import json
import shutil
import tempfile
import unittest
import subprocess
from unittest.mock import patch

class TestTaskPocket(unittest.TestCase):
    def setUp(self):
        self.test_dir = tempfile.mkdtemp()
        self.db_path = os.path.join(self.test_dir, "tasks.json")

    def tearDown(self):
        shutil.rmtree(self.test_dir)

    def run_cli(self, *args):
        cmd = [sys.executable, "task_cli.py"] + list(args)
        res = subprocess.run(cmd, capture_output=True, text=True)
        return res

    def assert_error_response(self, res):
        self.assertNotEqual(res.returncode, 0)
        self.assertEqual(res.stdout, "")
        self.assertTrue(len(res.stderr) > 0)
        self.assertNotIn("Traceback", res.stderr)

    def test_missing_db_list_returns_empty_array(self):
        res = self.run_cli("--db", self.db_path, "list")
        self.assertEqual(res.returncode, 0)
        self.assertEqual(json.loads(res.stdout), [])
        self.assertFalse(os.path.exists(self.db_path))

    def test_add_task_creates_parent_dir_and_task(self):
        nested_db = os.path.join(self.test_dir, "sub", "dir", "tasks.json")
        res = self.run_cli("--db", nested_db, "add", "  Buy milk  ")
        self.assertEqual(res.returncode, 0)
        task = json.loads(res.stdout)
        self.assertEqual(task, {"id": 1, "title": "Buy milk", "done": False})
        self.assertTrue(os.path.exists(nested_db))

    def test_list_open_and_all(self):
        self.run_cli("--db", self.db_path, "add", "Task 1")
        self.run_cli("--db", self.db_path, "add", "Task 2")
        self.run_cli("--db", self.db_path, "done", "1")

        res_open = self.run_cli("--db", self.db_path, "list")
        self.assertEqual(res_open.returncode, 0)
        open_tasks = json.loads(res_open.stdout)
        self.assertEqual(len(open_tasks), 1)
        self.assertEqual(open_tasks[0]["id"], 2)

        res_all = self.run_cli("--db", self.db_path, "list", "--all")
        self.assertEqual(res_all.returncode, 0)
        all_tasks = json.loads(res_all.stdout)
        self.assertEqual(len(all_tasks), 2)
        self.assertEqual([t["id"] for t in all_tasks], [1, 2])

    def test_done_idempotence(self):
        res_add = self.run_cli("--db", self.db_path, "add", "Clean room")
        task1 = json.loads(res_add.stdout)
        self.assertFalse(task1["done"])

        res_done1 = self.run_cli("--db", self.db_path, "done", "1")
        self.assertEqual(res_done1.returncode, 0)
        task_done1 = json.loads(res_done1.stdout)
        self.assertTrue(task_done1["done"])

        res_done2 = self.run_cli("--db", self.db_path, "done", "1")
        self.assertEqual(res_done2.returncode, 0)
        task_done2 = json.loads(res_done2.stdout)
        self.assertTrue(task_done2["done"])

    def test_id_increment_and_non_reuse(self):
        self.run_cli("--db", self.db_path, "add", "Task 1")
        self.run_cli("--db", self.db_path, "done", "1")
        res_add2 = self.run_cli("--db", self.db_path, "add", "Task 2")
        task2 = json.loads(res_add2.stdout)
        self.assertEqual(task2["id"], 2)

    def test_fresh_cli_process_persistence(self):
        res1 = subprocess.run([sys.executable, "task_cli.py", "--db", self.db_path, "add", "Persist test"], capture_output=True, text=True)
        self.assertEqual(res1.returncode, 0)
        res2 = subprocess.run([sys.executable, "task_cli.py", "--db", self.db_path, "list"], capture_output=True, text=True)
        self.assertEqual(res2.returncode, 0)
        tasks = json.loads(res2.stdout)
        self.assertEqual(len(tasks), 1)
        self.assertEqual(tasks[0]["title"], "Persist test")

    def test_blank_title_error(self):
        res = self.run_cli("--db", self.db_path, "add", "   ")
        self.assert_error_response(res)
        self.assertFalse(os.path.exists(self.db_path))

    def test_missing_id_error(self):
        self.run_cli("--db", self.db_path, "add", "Existing task")
        res = self.run_cli("--db", self.db_path, "done", "99")
        self.assert_error_response(res)

    def test_malformed_id_error(self):
        res = self.run_cli("--db", self.db_path, "done", "abc")
        self.assert_error_response(res)

    def test_missing_db_complete_error(self):
        res = self.run_cli("--db", self.db_path, "done", "1")
        self.assert_error_response(res)
        self.assertFalse(os.path.exists(self.db_path))

    def test_malformed_json_db_byte_unchanged_across_subcommands(self):
        bad_json = "{ invalid json content }"
        with open(self.db_path, "w") as f:
            f.write(bad_json)

        for subcmd in [["list"], ["add", "New task"], ["done", "1"]]:
            res = self.run_cli("--db", self.db_path, *subcmd)
            self.assert_error_response(res)
            with open(self.db_path, "r") as f:
                content = f.read()
            self.assertEqual(content, bad_json)

    def test_invalid_db_shape_byte_unchanged_across_subcommands(self):
        bad_shape = json.dumps({"tasks": "not a list", "next_id": 1})
        with open(self.db_path, "w") as f:
            f.write(bad_shape)

        for subcmd in [["list"], ["add", "New task"], ["done", "1"]]:
            res = self.run_cli("--db", self.db_path, *subcmd)
            self.assert_error_response(res)
            with open(self.db_path, "r") as f:
                content = f.read()
            self.assertEqual(content, bad_shape)

    def test_untrimmed_title_in_db_byte_unchanged(self):
        bad_title_db = json.dumps({"tasks": [{"id": 1, "title": "  untrimmed  ", "done": False}], "next_id": 2})
        with open(self.db_path, "w") as f:
            f.write(bad_title_db)

        res = self.run_cli("--db", self.db_path, "list")
        self.assert_error_response(res)
        with open(self.db_path, "r") as f:
            content = f.read()
        self.assertEqual(content, bad_title_db)

    def test_duplicate_ids_and_invalid_next_id_in_db(self):
        dup_db = json.dumps({"tasks": [{"id": 1, "title": "a", "done": False}, {"id": 1, "title": "b", "done": False}], "next_id": 2})
        with open(self.db_path, "w") as f:
            f.write(dup_db)
        res = self.run_cli("--db", self.db_path, "list")
        self.assert_error_response(res)

        bad_next_id_db = json.dumps({"tasks": [{"id": 5, "title": "a", "done": False}], "next_id": 3})
        with open(self.db_path, "w") as f:
            f.write(bad_next_id_db)
        res2 = self.run_cli("--db", self.db_path, "list")
        self.assert_error_response(res2)

    def test_direct_storage_validation(self):
        from task_store import complete_task, TaskStoreError
        with self.assertRaises(TaskStoreError):
            complete_task(self.db_path, True)

    def test_os_replace_failure_cleanup(self):
        from task_store import add_task, TaskStoreError
        with patch("os.replace", side_effect=OSError("Disk write failed")):
            with self.assertRaises((OSError, TaskStoreError)):
                add_task(self.db_path, "Test atomic failure")
        dir_files = os.listdir(self.test_dir)
        temp_files = [f for f in dir_files if f.startswith(".task-pocket-")]
        self.assertEqual(temp_files, [])

if __name__ == "__main__":
    unittest.main()
