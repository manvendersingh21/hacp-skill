#!/usr/bin/env python3
"""Command-line interface for the Task Pocket tracker."""
import argparse
import json
import sys

import task_store


def build_parser():
    parser = argparse.ArgumentParser(prog="task_cli.py")
    parser.add_argument("--db", required=True, help="path to the task database file")

    subparsers = parser.add_subparsers(dest="command", required=True)

    add_parser = subparsers.add_parser("add", help="add a new task")
    add_parser.add_argument("title", help="task title")

    list_parser = subparsers.add_parser("list", help="list tasks")
    list_parser.add_argument(
        "--all", action="store_true", dest="all", help="include completed tasks"
    )

    done_parser = subparsers.add_parser("done", help="mark a task as done")
    done_parser.add_argument("id", type=int, help="task id")

    return parser


def main(argv=None):
    parser = build_parser()
    args = parser.parse_args(argv)

    try:
        if args.command == "add":
            task = task_store.add_task(args.db, args.title)
            print(json.dumps(task))
        elif args.command == "list":
            tasks = task_store.list_tasks(args.db, include_done=args.all)
            print(json.dumps(tasks))
        elif args.command == "done":
            task = task_store.complete_task(args.db, args.id)
            print(json.dumps(task))
    except (ValueError, OSError) as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 1

    return 0


if __name__ == "__main__":
    sys.exit(main())
