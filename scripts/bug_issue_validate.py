#!/usr/bin/env python3
"""Validate BUG-#### backlog blocks and acceptance reconciliation (DEC-0061 / US-0079)."""

from __future__ import annotations

import argparse
import sys
from pathlib import Path

_SCRIPT_DIR = Path(__file__).resolve().parent
if str(_SCRIPT_DIR) not in sys.path:
    sys.path.insert(0, str(_SCRIPT_DIR))

import bug_issue_lib  # noqa: E402


def main() -> int:
    p = argparse.ArgumentParser(description="Validate canonical bug issues in backlog.md")
    p.add_argument("--backlog", type=Path, help="Path to docs/product/backlog.md")
    p.add_argument("--acceptance", type=Path, default=Path("docs/product/acceptance.md"))
    p.add_argument("--check-acceptance", action="store_true")
    p.add_argument("--print-next-id", action="store_true")
    p.add_argument("--self-test", action="store_true")
    args = p.parse_args()

    if args.self_test:
        bug_issue_lib.self_test()
        print("[BUG_ISSUE_LIB_SELF_TEST_OK]")
        return 0

    if args.print_next_id:
        backlog_path = args.backlog or Path("docs/product/backlog.md")
        text = backlog_path.read_text(encoding="utf-8")
        bugs = bug_issue_lib.parse_bug_issues(text)
        print(bug_issue_lib.next_bug_id(bugs))
        return 0

    if not args.backlog:
        print("error: --backlog required unless --print-next-id or --self-test", file=sys.stderr)
        return 2

    backlog_text = args.backlog.read_text(encoding="utf-8")
    acceptance_text = None
    if args.check_acceptance:
        acceptance_path = args.acceptance
        if not acceptance_path.is_file():
            print(f"error: acceptance file not found: {acceptance_path}", file=sys.stderr)
            return 2
        acceptance_text = acceptance_path.read_text(encoding="utf-8")

    result = bug_issue_lib.validate_bugs(backlog_text, acceptance_text=acceptance_text)
    if result.ok:
        print("[BUG_VALIDATION_OK]")
        return 0

    print("BUG_VALIDATION_FAILED", file=sys.stderr)
    for code in result.codes:
        print(code, file=sys.stderr)
    for msg in result.messages:
        print(msg, file=sys.stderr)
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
