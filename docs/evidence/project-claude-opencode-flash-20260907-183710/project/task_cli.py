#!/usr/bin/env python3
"""Task Pocket: a tiny local task tracker CLI.

Usage:
    python3 task_cli.py --db PATH add "title"
    python3 task_cli.py --db PATH list [--all]
    python3 task_cli.py --db PATH done ID

Successful commands print only JSON on stdout. Errors print a message
on stderr and exit nonzero without a traceback.
"""

import argparse
import json
import sys

import task_store


def build_parser():
    parser = argparse.ArgumentParser(
        prog="task_cli.py",
        description="Task Pocket: tiny local task tracker",
    )
    parser.add_argument("--db", required=True, help="path to the JSON database file")
    sub = parser.add_subparsers(dest="command", required=True)

    p_add = sub.add_parser("add", help="add a new task")
    p_add.add_argument("title", help="task title (trimmed, must be nonempty)")

    p_list = sub.add_parser("list", help="list open tasks sorted by id")
    p_list.add_argument(
        "--all", action="store_true", dest="include_done",
        help="include completed tasks",
    )

    p_done = sub.add_parser("done", help="mark a task as done")
    p_done.add_argument("id", help="task id (integer)")

    return parser


def main(argv=None):
    args = build_parser().parse_args(argv)
    try:
        if args.command == "add":
            result = task_store.add_task(args.db, args.title)
        elif args.command == "list":
            result = task_store.list_tasks(args.db, include_done=args.include_done)
        else:
            try:
                task_id = int(args.id)
            except ValueError:
                raise ValueError(
                    "invalid task id %r: expected an integer" % (args.id,)
                )
            result = task_store.complete_task(args.db, task_id)
    except (ValueError, TypeError, OSError) as exc:
        print("error: %s" % exc, file=sys.stderr)
        return 1
    print(json.dumps(result))
    return 0


if __name__ == "__main__":
    sys.exit(main())
