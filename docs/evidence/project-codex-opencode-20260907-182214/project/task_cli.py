#!/usr/bin/env python3
"""Task Pocket: a tiny local task CLI.

Flags come before the subcommand:
    python3 task_cli.py --db PATH add "title"
    python3 task_cli.py --db PATH list [--all]
    python3 task_cli.py --db PATH done ID
"""

import argparse
import json
import sys

import task_store


def build_parser():
    parser = argparse.ArgumentParser(
        prog="task_cli.py",
        description="A tiny local task tracker.",
    )
    parser.add_argument("--db", required=True, help="path to the JSON task database")
    subparsers = parser.add_subparsers(dest="command", required=True)

    add_parser = subparsers.add_parser("add", help="add a new open task")
    add_parser.add_argument("title", help="task title (nonempty, trimmed)")

    list_parser = subparsers.add_parser("list", help="list open tasks, sorted by id")
    list_parser.add_argument(
        "--all", action="store_true", help="include completed tasks"
    )

    done_parser = subparsers.add_parser("done", help="mark a task completed")
    done_parser.add_argument("id", help="task id (positive integer)")

    return parser


def fail(message):
    print("task_cli.py: error: %s" % message, file=sys.stderr)
    return 1


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
            try:
                task_id = int(args.id)
            except ValueError:
                return fail("invalid task id: %r (expected an integer)" % args.id)
            task = task_store.complete_task(args.db, task_id)
            print(json.dumps(task))
        else:  # pragma: no cover - argparse rejects unknown commands
            return fail("unknown command: %s" % args.command)
    except task_store.TaskStoreError as exc:
        return fail(str(exc))

    return 0


if __name__ == "__main__":
    sys.exit(main())
