#!/usr/bin/env python3
"""Verify active vs template/scripts/ bytes match for DEC-0063 intake gate modules (BUG-0001).

Scoped modes (DEC-0073 §10 / US-0090):
  --scope=intake          (default) DEC-0063 intake pair table.
  --scope=caveman-compress DEC-0073 caveman input-compression pair table.
  --scope=all              union of both tables.
"""

from __future__ import annotations

import argparse
import sys
from pathlib import Path

INTAKE_TEMPLATE_PAIRS: tuple[tuple[str, str], ...] = (
    ("scripts/intake_evidence_validate.py", "template/scripts/intake_evidence_validate.py"),
    ("scripts/intake_evidence_lib.py", "template/scripts/intake_evidence_lib.py"),
    ("scripts/intake_bug_routing_guard.py", "template/scripts/intake_bug_routing_guard.py"),
    ("scripts/intake_bug_resume_brief_refresh.py", "template/scripts/intake_bug_resume_brief_refresh.py"),
    ("scripts/check_intake_template_parity.py", "template/scripts/check_intake_template_parity.py"),
)

# DEC-0073 §10 / US-0090 — Caveman input-compression surface pairs. Contents
# must be byte-identical between active and template paths; installer delivers
# template copies (BUG-0003 / DEC-0066).
CAVEMAN_COMPRESS_PAIRS: tuple[tuple[str, str], ...] = (
    ("scripts/caveman_compress_input.py", "template/scripts/caveman_compress_input.py"),
    ("docs/engineering/context/installer-owned-paths.manifest",
     "template/docs/engineering/context/installer-owned-paths.manifest"),
    ("docs/engineering/runbook.md", "template/docs/engineering/runbook.md"),
    ("docs/engineering/auto-orchestration-reference.md",
     "template/docs/engineering/auto-orchestration-reference.md"),
)

SCOPES: dict[str, tuple[tuple[str, str], ...]] = {
    "intake": INTAKE_TEMPLATE_PAIRS,
    "caveman-compress": CAVEMAN_COMPRESS_PAIRS,
    "all": INTAKE_TEMPLATE_PAIRS + CAVEMAN_COMPRESS_PAIRS,
}


def main() -> int:
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument(
        "--repo",
        type=Path,
        default=Path(__file__).resolve().parent.parent,
        help="Repository root",
    )
    p.add_argument(
        "--scope",
        choices=sorted(SCOPES.keys()),
        default="intake",
        help="Parity pair table to verify.",
    )
    args = p.parse_args()
    root: Path = args.repo
    pairs = SCOPES[args.scope]
    failed = False
    for rel_active, rel_tpl in pairs:
        a = root / rel_active
        t = root / rel_tpl
        if not a.is_file() or not t.is_file():
            print(f"[INTAKE_TEMPLATE_PARITY_ERROR] missing file: {rel_active} or {rel_tpl}")
            failed = True
            continue
        ba = a.read_bytes()
        bt = t.read_bytes()
        if ba != bt:
            print(
                f"[INTAKE_TEMPLATE_PARITY_ERROR] mismatch: {rel_active} ({len(ba)}b) "
                f"!= {rel_tpl} ({len(bt)}b)"
            )
            failed = True
    if failed:
        return 2
    print(f"[INTAKE_TEMPLATE_PARITY_OK] scope={args.scope}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
