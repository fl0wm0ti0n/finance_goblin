#!/usr/bin/env python3
"""Refresh handoffs/resume_brief.md after bug intake (DEC-0069 / BUG-0005)."""

from __future__ import annotations

import argparse
import re
import sys
from datetime import datetime, timezone
from pathlib import Path

_SCRIPT_DIR = Path(__file__).resolve().parent
if str(_SCRIPT_DIR) not in sys.path:
    sys.path.insert(0, str(_SCRIPT_DIR))

import bug_issue_lib  # noqa: E402

BUG_ID_RE = re.compile(r"^BUG-\d{4}$")


def _parse_rfc3339z(value: str) -> datetime:
    raw = value.strip()
    if raw.endswith("Z"):
        raw = raw[:-1] + "+00:00"
    dt = datetime.fromisoformat(raw)
    if dt.tzinfo is None:
        dt = dt.replace(tzinfo=timezone.utc)
    return dt.astimezone(timezone.utc)


def _format_boundary(dt: datetime) -> str:
    return dt.astimezone(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


def build_resume_brief(
    *,
    bug_id: str,
    bug_title: str,
    boundary_utc: str,
    orchestrator_run_id: str | None,
    intake_evidence: str | None,
    sprint_id: str | None,
) -> str:
    run_line = orchestrator_run_id or "(none)"
    evidence_line = intake_evidence or "(none)"
    sprint_line = sprint_id or "(none)"
    return f"""# Resume Brief

## Current status

- **Active bug:** {bug_id} — {bug_title}
- **Bug status:** {bug_id} **OPEN**
- **Active story:** none
- **Latest release:** US-0012 (`0.12.0-us0012`, 2026-06-03)
- **Orchestrator run:** {run_line}
- **Last completed phase:** intake ({boundary_utc})

## Next actions

1. Run **`/discovery`** on {bug_id} (omniflow production regression fixes)
2. Architecture/research for Grafana subpath asset URLs vs Q0005 AuthProvider guard pattern
3. Operator re-smoke on `financegnome.omniflow.cc` after fix deploy

## Intended resume phase

**discovery** — {bug_id} defect triage and scope refinement

## Resolution metadata

- `resolution_source`: resume_brief
- `resolved_start_phase`: discovery
- `segment_work_item_kind`: bug
- `active_bug_id`: {bug_id}
- `bug_id`: {bug_id}
- `bug_queue_position`: 1
- `bug_queue_remaining`: 0
- `intake_boundary_utc`: {boundary_utc}
- `intake_evidence`: {evidence_line}
- `sprint_id`: {sprint_line}
- `backlog_drain_active`: false
- `context_refreshed`: true (post-bug-intake, {boundary_utc})
"""


def validate_brief_content(text: str, bug_id: str) -> list[str]:
    errors: list[str] = []
    if f"**discovery**" not in text and "discovery" not in text:
        errors.append("INTAKE_RESUME_BRIEF_PHASE_MISSING: intended phase must be discovery")
    if f"`resolved_start_phase`: discovery" not in text and "resolved_start_phase`: discovery" not in text:
        errors.append("INTAKE_RESUME_BRIEF_RESOLVED_PHASE: resolved_start_phase must be discovery")
    if bug_id not in text:
        errors.append(f"INTAKE_RESUME_BRIEF_BUG_ID: {bug_id} not found in brief")
    if "resolution_source`: resume_brief" not in text:
        errors.append("INTAKE_RESUME_BRIEF_SOURCE: resolution_source must be resume_brief")
    return errors


def main() -> int:
    p = argparse.ArgumentParser(description="Refresh resume brief after bug intake")
    p.add_argument("--bug-id", required=True)
    p.add_argument("--backlog", type=Path, default=Path("docs/product/backlog.md"))
    p.add_argument("--resume-brief", type=Path, default=Path("handoffs/resume_brief.md"))
    p.add_argument("--intake-boundary-utc", required=False)
    p.add_argument("--orchestrator-run-id")
    p.add_argument("--intake-evidence")
    p.add_argument("--sprint-id")
    p.add_argument("--validate-file", action="store_true", help="Validate existing brief only")
    args = p.parse_args()

    if not BUG_ID_RE.match(args.bug_id):
        print(f"INTAKE_RESUME_BRIEF_BUG_ID_INVALID: {args.bug_id}", file=sys.stderr)
        return 1

    backlog_text = args.backlog.read_text(encoding="utf-8")
    bugs = bug_issue_lib.parse_bug_issues(backlog_text)
    match = next((b for b in bugs if b.bug_id == args.bug_id), None)
    if not match:
        print(f"INTAKE_RESUME_BRIEF_BUG_NOT_FOUND: {args.bug_id}", file=sys.stderr)
        return 1
    if match.status != "OPEN":
        print(f"INTAKE_RESUME_BRIEF_BUG_NOT_OPEN: {args.bug_id} status={match.status}", file=sys.stderr)
        return 1

    if args.validate_file:
        if not args.resume_brief.is_file():
            print(f"INTAKE_RESUME_BRIEF_FILE_MISSING: {args.resume_brief}", file=sys.stderr)
            return 1
        errors = validate_brief_content(args.resume_brief.read_text(encoding="utf-8"), args.bug_id)
        if errors:
            for e in errors:
                print(e, file=sys.stderr)
            return 1
        print("[INTAKE_RESUME_BRIEF_VALIDATION_OK]")
        return 0

    if not args.intake_boundary_utc:
        print("INTAKE_RESUME_BRIEF_BOUNDARY_MISSING: --intake-boundary-utc required", file=sys.stderr)
        return 1

    try:
        boundary_dt = _parse_rfc3339z(args.intake_boundary_utc)
    except ValueError as exc:
        print(f"INTAKE_RESUME_BRIEF_BOUNDARY_INVALID: {exc}", file=sys.stderr)
        return 1

    boundary = _format_boundary(boundary_dt)
    content = build_resume_brief(
        bug_id=args.bug_id,
        bug_title=match.title,
        boundary_utc=boundary,
        orchestrator_run_id=args.orchestrator_run_id,
        intake_evidence=args.intake_evidence,
        sprint_id=args.sprint_id,
    )
    errors = validate_brief_content(content, args.bug_id)
    if errors:
        for e in errors:
            print(e, file=sys.stderr)
        return 1

    target = args.resume_brief
    tmp = target.with_suffix(target.suffix + ".tmp")
    tmp.write_text(content, encoding="utf-8")
    tmp.replace(target)
    print("[INTAKE_RESUME_BRIEF_REFRESH_OK]")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
