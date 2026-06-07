"""
Scratchpad merge and validation helpers (DEC-0055 / DEC-0058).

Minimal implementation for doc-profile and sync-push gate scripts when the full
its-magic installer package is not present in the repo root.
"""

from __future__ import annotations

import os
from typing import Dict, List, Tuple


def _parse_scratchpad_text(text: str, into: Dict[str, str]) -> None:
    for raw in text.splitlines():
        line = raw.strip()
        if not line or line.startswith("#"):
            continue
        if line.startswith("- "):
            continue
        if "=" not in line:
            continue
        key, _, val = line.partition("=")
        key, val = key.strip(), val.strip()
        if key and val:
            into[key] = val


def _scratchpad_paths(root: str) -> Dict[str, str]:
    cursor = os.path.join(root, ".cursor")
    return {
        "example": os.path.join(cursor, "scratchpad.local.example.md"),
        "baseline": os.path.join(cursor, "scratchpad.md"),
        "local": os.path.join(cursor, "scratchpad.local.md"),
    }


def merge_scratchpad_layers(root: str) -> Tuple[Dict[str, str], Dict[str, str]]:
    """
    Merge precedence: local > baseline > example (DEC-0055).
    Returns (merged_key_values, path_map).
    """
    paths = _scratchpad_paths(root)
    merged: Dict[str, str] = {}
    for layer in ("example", "baseline", "local"):
        path = paths[layer]
        if os.path.isfile(path):
            with open(path, "r", encoding="utf-8") as f:
                _parse_scratchpad_text(f.read(), merged)
    return merged, paths


def validate_merged_scratchpad(root: str) -> Tuple[bool, List[str]]:
    """
    Fail closed when required scratchpad layers are missing.
    """
    paths = _scratchpad_paths(root)
    diags: List[str] = []
    if not os.path.isfile(paths["example"]):
        diags.append(
            "[SCRATCHPAD_MERGE_ERROR] EXAMPLE_LAYER_MISSING: "
            f".cursor/scratchpad.local.example.md not found under {root}."
        )
    if not os.path.isfile(paths["baseline"]):
        diags.append(
            "[SCRATCHPAD_MERGE_ERROR] MATERIALIZED_BASELINE_MISSING: "
            f".cursor/scratchpad.md not found under {root}."
        )
    return (len(diags) == 0, diags)
