"""Parse and validate BUG-#### blocks in docs/product/backlog.md (DEC-0061 / US-0079)."""

from __future__ import annotations

import re
from dataclasses import dataclass, field

BUG_HEADER_RE = re.compile(r"^###\s+(BUG-\d{4})\s+—\s+(.+?)\s*$", re.MULTILINE)
STATUS_RE = re.compile(r"^\s*\*?\*?Status\*?\*?\s*:\s*(OPEN|DONE)\s*$", re.MULTILINE | re.IGNORECASE)
REQUIRED_FIELDS = (
    "environment",
    "steps_to_reproduce",
    "expected",
    "actual",
    "evidence_refs",
)
FIELD_RE_TEMPLATE = r"^\s*\*?\*?{name}\*?\*?\s*:\s*(.+?)\s*$"


@dataclass
class BugIssue:
    bug_id: str
    title: str
    status: str
    fields: dict[str, str]
    body: str


@dataclass
class ValidationResult:
    ok: bool
    codes: list[str] = field(default_factory=list)
    messages: list[str] = field(default_factory=list)


def _field_re(name: str) -> re.Pattern[str]:
    return re.compile(FIELD_RE_TEMPLATE.format(name=re.escape(name)), re.MULTILINE | re.IGNORECASE)


def _extract_canonical_section(text: str, heading: str) -> str:
    marker = f"## {heading}"
    start = text.find(marker)
    if start < 0:
        return ""
    rest = text[start + len(marker) :]
    next_h = re.search(r"\n## [^\n]+", rest)
    return rest[: next_h.start()] if next_h else rest


def parse_bug_issues(backlog_text: str) -> list[BugIssue]:
    section = _extract_canonical_section(backlog_text, "Bug issues (canonical)")
    if not section.strip():
        return []
    if "_(none)_" in section and not BUG_HEADER_RE.search(section):
        return []

    headers = list(BUG_HEADER_RE.finditer(section))
    bugs: list[BugIssue] = []
    for i, match in enumerate(headers):
        bug_id, title = match.group(1), match.group(2).strip()
        block_start = match.end()
        block_end = headers[i + 1].start() if i + 1 < len(headers) else len(section)
        body = section[block_start:block_end]
        status_match = STATUS_RE.search(body)
        status = status_match.group(1).upper() if status_match else ""
        fields: dict[str, str] = {}
        for name in REQUIRED_FIELDS:
            fm = _field_re(name).search(body)
            fields[name] = fm.group(1).strip() if fm else ""
        bugs.append(BugIssue(bug_id=bug_id, title=title, status=status, fields=fields, body=body))
    return bugs


def next_bug_id(bugs: list[BugIssue]) -> str:
    if not bugs:
        return "BUG-0001"
    nums = [int(b.bug_id.split("-")[1]) for b in bugs]
    return f"BUG-{max(nums) + 1:04d}"


def parse_bug_acceptance(acceptance_text: str) -> dict[str, bool]:
    section = _extract_canonical_section(acceptance_text, "Bug acceptance (canonical)")
    rows: dict[str, bool] = {}
    for line in section.splitlines():
        m = re.match(r"^\s*-\s+\[( |x|X)\]\s+\**\s*(BUG-\d{4})\b", line)
        if m:
            rows[m.group(2)] = m.group(1).lower() == "x"
    return rows


def validate_bugs(backlog_text: str, *, acceptance_text: str | None = None) -> ValidationResult:
    res = ValidationResult(ok=True)
    section = _extract_canonical_section(backlog_text, "Bug issues (canonical)")
    if not section.strip():
        res.ok = False
        res.codes.append("BUG_VALIDATION_SECTION_MISSING")
        res.messages.append("Missing ## Bug issues (canonical) section.")
        return res

    bugs = parse_bug_issues(backlog_text)
    if "_(none)_" in section and not bugs:
        res.ok = False
        res.codes.append("BUG_VALIDATION_SECTION_MISSING")
        res.messages.append("Bug section contains _(none)_ placeholder with no bug blocks.")

    prev_num = 0
    for bug in bugs:
        num = int(bug.bug_id.split("-")[1])
        if num <= prev_num:
            res.ok = False
            res.codes.append("BUG_VALIDATION_ORDER_INVERSION")
            res.messages.append(f"Bug ids must be ascending; inversion at {bug.bug_id}.")
        prev_num = num

        if bug.status not in {"OPEN", "DONE"}:
            res.ok = False
            res.codes.append("BUG_VALIDATION_STATUS_INVALID")
            res.messages.append(f"{bug.bug_id}: Status must be OPEN or DONE (got {bug.status!r}).")

        for fname in REQUIRED_FIELDS:
            if not bug.fields.get(fname, "").strip():
                res.ok = False
                res.codes.append("BUG_VALIDATION_FIELD_EMPTY")
                res.messages.append(f"{bug.bug_id}: missing or empty {fname}.")

    if acceptance_text is not None:
        acc = parse_bug_acceptance(acceptance_text)
        acc_section = _extract_canonical_section(acceptance_text, "Bug acceptance (canonical)")
        if bugs and not acc_section.strip():
            res.ok = False
            res.codes.append("BUG_RECONCILE_ACCEPTANCE_MISSING")
            res.messages.append("Missing ## Bug acceptance (canonical) section.")
        bug_ids = {b.bug_id for b in bugs}
        for bid in sorted(bug_ids):
            if bid not in acc:
                res.ok = False
                res.codes.append("BUG_RECONCILE_ACCEPTANCE_MISSING_ROW")
                res.messages.append(f"Acceptance missing checkbox row for {bid}.")
        for bid, checked in acc.items():
            if bid not in bug_ids:
                res.ok = False
                res.codes.append("BUG_RECONCILE_ACCEPTANCE_ORPHAN")
                res.messages.append(f"Acceptance row {bid} has no backlog bug block.")
            else:
                bug = next(b for b in bugs if b.bug_id == bid)
                done_expected = bug.status == "DONE"
                if checked != done_expected:
                    res.ok = False
                    res.codes.append("BUG_RECONCILE_ACCEPTANCE_STATUS_DRIFT")
                    res.messages.append(
                        f"{bid}: backlog Status={bug.status} but acceptance "
                        f"{'checked' if checked else 'unchecked'}."
                    )

    return res


def self_test() -> None:
    sample = """## Bug issues (canonical)

### BUG-0001 — Sample

Status: OPEN

**environment:** dev
**steps_to_reproduce:** click
**expected:** ok
**actual:** fail
**evidence_refs:** test.md
"""
    bugs = parse_bug_issues(sample)
    assert len(bugs) == 1
    assert bugs[0].bug_id == "BUG-0001"
    assert bugs[0].status == "OPEN"
    acc = """## Bug acceptance (canonical)

- [ ] **BUG-0001** — sample
"""
    r = validate_bugs(sample, acceptance_text=acc)
    assert r.ok, r.messages
