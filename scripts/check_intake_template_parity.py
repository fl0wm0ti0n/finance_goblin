#!/usr/bin/env python3
"""Verify active vs template/scripts/ bytes match for DEC-0063 intake gate modules (BUG-0001).

Scoped modes (DEC-0073 §10 / US-0090):
  --scope=intake          (default) DEC-0063 intake pair table.
  --scope=caveman-compress DEC-0073 caveman input-compression pair table.
  --scope=readme-feature-coverage DEC-0074 README feature-coverage pair table.
  --scope=downstream-ci-guard   DEC-0075 downstream CI guard script pair table.
  --scope=us-0092               DEC-0078 full-autonomy outer driver + probe surfaces.
  --scope=all              union of all tables.
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

README_FEATURE_COVERAGE_PAIRS: tuple[tuple[str, str], ...] = (
    (
        "scripts/validate_readme_feature_coverage.py",
        "template/scripts/validate_readme_feature_coverage.py",
    ),
    (
        "scripts/readme_feature_coverage_lib.py",
        "template/scripts/readme_feature_coverage_lib.py",
    ),
    (
        "docs/engineering/context/readme-section-affinity.json",
        "template/docs/engineering/context/readme-section-affinity.json",
    ),
    (".cursor/commands/release.md", "template/.cursor/commands/release.md"),
    ("docs/engineering/runbook.md", "template/docs/engineering/runbook.md"),
    (
        "docs/engineering/context/installer-owned-paths.manifest",
        "template/docs/engineering/context/installer-owned-paths.manifest",
    ),
    (
        "scripts/check_intake_template_parity.py",
        "template/scripts/check_intake_template_parity.py",
    ),
)

US0092_PAIRS: tuple[tuple[str, str], ...] = (
    ("scripts/auto_outer_driver.py", "template/scripts/auto_outer_driver.py"),
    ("scripts/uat_probe_lib.py", "template/scripts/uat_probe_lib.py"),
    (
        "docs/engineering/context/installer-owned-paths.manifest",
        "template/docs/engineering/context/installer-owned-paths.manifest",
    ),
    (".cursor/commands/auto.md", "template/.cursor/commands/auto.md"),
    (".cursor/commands/verify-work.md", "template/.cursor/commands/verify-work.md"),
    (".cursor/commands/qa.md", "template/.cursor/commands/qa.md"),
    (
        "docs/engineering/auto-orchestration-reference.md",
        "template/docs/engineering/auto-orchestration-reference.md",
    ),
    ("docs/engineering/runbook.md", "template/docs/engineering/runbook.md"),
)

DOWNSTREAM_CI_GUARD_PAIRS: tuple[tuple[str, str], ...] = (
    (
        "scripts/check_downstream_ci_guard.py",
        "template/scripts/check_downstream_ci_guard.py",
    ),
    (
        "scripts/downstream_ci_guard_lib.py",
        "template/scripts/downstream_ci_guard_lib.py",
    ),
)

SCOPES: dict[str, tuple[tuple[str, str], ...]] = {
    "intake": INTAKE_TEMPLATE_PAIRS,
    "caveman-compress": CAVEMAN_COMPRESS_PAIRS,
    "readme-feature-coverage": README_FEATURE_COVERAGE_PAIRS,
    "downstream-ci-guard": DOWNSTREAM_CI_GUARD_PAIRS,
    "us-0092": US0092_PAIRS,
    "all": (
        INTAKE_TEMPLATE_PAIRS
        + CAVEMAN_COMPRESS_PAIRS
        + README_FEATURE_COVERAGE_PAIRS
        + DOWNSTREAM_CI_GUARD_PAIRS
        + US0092_PAIRS
    ),
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
