#!/usr/bin/env python3
import sys
import json
from task_store import add_task, list_tasks, complete_task, TaskStoreError

def main():
    args = sys.argv[1:]
    if len(args) < 3 or args[0] != "--db":
        sys.stderr.write("Error: Usage: python3 task_cli.py --db PATH <subcommand> [args...]\n")
        sys.exit(1)

    db_path = args[1]
    subcommand = args[2]
    sub_args = args[3:]

    try:
        if subcommand == "add":
            if len(sub_args) != 1:
                sys.stderr.write("Error: 'add' subcommand requires exactly one title argument.\n")
                sys.exit(1)
            title = sub_args[0]
            if not title or not title.strip():
                sys.stderr.write("Error: Task title cannot be blank.\n")
                sys.exit(1)
            task = add_task(db_path, title)
            print(json.dumps(task))
        elif subcommand == "list":
            include_done = False
            if len(sub_args) == 1:
                if sub_args[0] == "--all":
                    include_done = True
                else:
                    sys.stderr.write(f"Error: Unknown option for list: {sub_args[0]}\n")
                    sys.exit(1)
            elif len(sub_args) > 1:
                sys.stderr.write("Error: Unexpected arguments for list.\n")
                sys.exit(1)

            tasks = list_tasks(db_path, include_done=include_done)
            print(json.dumps(tasks))
        elif subcommand == "done":
            if len(sub_args) != 1:
                sys.stderr.write("Error: 'done' subcommand requires exactly one ID argument.\n")
                sys.exit(1)
            try:
                task_id = int(sub_args[0])
                if isinstance(sub_args[0], bool):
                    raise ValueError
            except (ValueError, TypeError):
                sys.stderr.write(f"Error: Task ID must be an integer, got '{sub_args[0]}'.\n")
                sys.exit(1)
            if task_id <= 0:
                sys.stderr.write(f"Error: Task ID must be positive, got {task_id}.\n")
                sys.exit(1)

            task = complete_task(db_path, task_id)
            print(json.dumps(task))
        else:
            sys.stderr.write(f"Error: Unknown subcommand '{subcommand}'.\n")
            sys.exit(1)
    except TaskStoreError as e:
        sys.stderr.write(f"Error: {e}\n")
        sys.exit(1)
    except ValueError as e:
        sys.stderr.write(f"Error: {e}\n")
        sys.exit(1)
    except OSError as e:
        sys.stderr.write(f"Error: {e}\n")
        sys.exit(1)
    except Exception as e:
        sys.stderr.write(f"Error: {e}\n")
        sys.exit(1)

if __name__ == "__main__":
    main()
