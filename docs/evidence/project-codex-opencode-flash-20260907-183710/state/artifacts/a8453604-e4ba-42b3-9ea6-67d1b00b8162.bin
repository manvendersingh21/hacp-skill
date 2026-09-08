#!/usr/bin/env python3
"""Task Pocket CLI: a tiny local task tracker backed by task_store.

Usage:
    python3 task_cli.py --db PATH add "title"
    python3 task_cli.py --db PATH list [--all]
    python3 task_cli.py --db PATH done ID

Successful commands print only JSON on stdout. Errors print a single
"Error: ..." line on stderr and exit 1.
"""

import argparse
import json
import sys

from task_store import TaskStoreError, add_task, complete_task, list_tasks


def parse_task_id(raw):
    """Convert a CLI id argument into a positive integer task id."""
    try:
        task_id = int(raw, 10)
    except (TypeError, ValueError):
        raise TaskStoreError("invalid task id: {0!r} (expected a positive integer)".format(raw))
    if task_id < 1:
        raise TaskStoreError("invalid task id: {0!r} (expected a positive integer)".format(raw))
    return task_id


def build_parser():
    parser = argparse.ArgumentParser(
        prog="task_cli.py",
        description="Task Pocket: a tiny dependency-free task tracker.",
    )
    parser.add_argument("--db", required=True, help="path to the JSON database file")
    sub = parser.add_subparsers(dest="command", required=True)

    add_parser = sub.add_parser("add", help="add a task and print it as JSON")
    add_parser.add_argument("title", help="task title (trimmed, must be nonempty)")

    list_parser = sub.add_parser("list", help="list tasks sorted by id")
    list_parser.add_argument(
        "--all", action="store_true", dest="include_done",
        help="include completed tasks",
    )

    done_parser = sub.add_parser("done", help="complete a task and print it as JSON")
    done_parser.add_argument("id", help="task id to complete")

    return parser


def main(argv=None):
    args = build_parser().parse_args(argv)
    try:
        if args.command == "add":
            result = add_task(args.db, args.title)
        elif args.command == "list":
            result = list_tasks(args.db, include_done=args.include_done)
        else:
            result = complete_task(args.db, parse_task_id(args.id))
    except (TaskStoreError, OSError) as exc:
        sys.stderr.write("Error: {0}\n".format(exc))
        return 1
    print(json.dumps(result))
    return 0


if __name__ == "__main__":
    sys.exit(main())
