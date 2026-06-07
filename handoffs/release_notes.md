# Release Notes (Legacy Compatibility Pointer)

This file remains backward-compatible for workflows that read
`handoffs/release_notes.md` as the latest release summary.

Canonical sprint history lives under:
- `handoffs/releases/Sxxxx-release-notes.md`

Canonical quick-task / bug-fix history:
- `handoffs/releases/Q0023-release-notes.md` (BUG-0015)
- `handoffs/releases/Q0022-release-notes.md` (BUG-0014)
- `handoffs/releases/Q0021-release-notes.md` (US-0017)
- `handoffs/releases/Q0020-release-notes.md` (BUG-0013)
- `handoffs/releases/Q0019-release-notes.md` (BUG-0011)
- `handoffs/releases/Q0018-release-notes.md` (BUG-0008)
- `handoffs/releases/Q0017-release-notes.md` (BUG-0007)
- `handoffs/releases/Q0016-release-notes.md` (BUG-0009)
- `handoffs/releases/Q0014-release-notes.md` (BUG-0012)
- `handoffs/releases/Q0013-release-notes.md` (BUG-0010)
- `handoffs/releases/Q0010-release-notes.md` (BUG-0006)
- `handoffs/releases/Q0012-release-notes.md` (BUG-0005)
- `handoffs/releases/Q0011-release-notes.md` (BUG-0004)
- `handoffs/releases/Q0009-release-notes.md` (BUG-0003)
- `handoffs/releases/Q0008-release-notes.md` (BUG-0002)
- `handoffs/releases/Q0007-release-notes.md` (BUG-0001)

Canonical queue state:
- `handoffs/release_queue.md`

---

## Latest finalized release pointer

- **Latest released quick task:** Q0023 / BUG-0015 (2026-06-07)
- **Latest quick-task notes:** [handoffs/releases/Q0023-release-notes.md](releases/Q0023-release-notes.md)
- **Bug status:** DONE; acceptance AU–AW checked (runtime operator-deferred)
- **Release version:** `bug0015-q0023`
- **Open stories:** (empty — backlog drain complete for current scope)
- **Open bug queue:** (empty — defect drain complete)

- **Latest released sprint:** S0016
- **Latest sprint notes:** [handoffs/releases/S0016-release-notes.md](releases/S0016-release-notes.md)
- **Latest sprint date:** 2026-06-06
- **Latest sprint story:** US-0015
- **Sprint release version:** `0.16.0-us0015`

## Unreleased queue visibility

No rows with `status=unreleased` or `status=blocked`. See `handoffs/release_queue.md` for full queue.

## Latest operator summary (Run/Connect/Verify) — BUG-0015 / Q0023

- **Deploy:** `docker compose -f docker-compose.yml -f docker-compose.external.yml --profile external up -d --build flow-finance-ai`
- **Verify:** `cd backend && cargo test --lib` (187/187); operator 10-step rebuild smoke per `sprints/quick/Q0023/uat.json` after deploy gates
- **Endpoint:** `https://financegnome.omniflow.cc`
- **Full steps:** [handoffs/releases/Q0023-release-notes.md](releases/Q0023-release-notes.md)

## Prior operator summary — BUG-0014 / Q0022

- **Deploy:** `docker compose -f docker-compose.yml -f docker-compose.external.yml --profile external up -d --build flow-finance-ai grafana stats-forecast`
- **Verify:** `cd backend && cargo test --lib` (177/177); operator 14-step smoke per `sprints/quick/Q0022/uat.json` after deploy gates
- **Endpoint:** `https://financegnome.omniflow.cc`
- **Full steps:** [handoffs/releases/Q0022-release-notes.md](releases/Q0022-release-notes.md)

## Prior operator summary — US-0017 / Q0021

- **Deploy:** unchanged — `docker compose -f docker-compose.yml -f docker-compose.external.yml --profile external up -d` (see README Quickstart)
- **Verify:** `python3 scripts/validate_doc_profile.py --repo . --no-template-parity` (exit 0); optional omniflow smoke curls from README `### Omniflow smoke (external profile)`
- **Endpoint:** `https://financegnome.omniflow.cc` or `http://localhost:8080`
- **Full steps:** [handoffs/releases/Q0021-release-notes.md](releases/Q0021-release-notes.md)

## Historical references

- Q0023 / BUG-0015: `handoffs/releases/Q0023-release-notes.md`
- Q0022 / BUG-0014: `handoffs/releases/Q0022-release-notes.md`
- Q0021 / US-0017: `handoffs/releases/Q0021-release-notes.md`
- Q0020 / BUG-0013: `handoffs/releases/Q0020-release-notes.md`
- S0016: `handoffs/releases/S0016-release-notes.md`
- Q0019 / BUG-0011: `handoffs/releases/Q0019-release-notes.md`
- Q0018 / BUG-0008: `handoffs/releases/Q0018-release-notes.md`
- Q0017 / BUG-0007: `handoffs/releases/Q0017-release-notes.md`
- Q0016 / BUG-0009: `handoffs/releases/Q0016-release-notes.md`
- Q0014 / BUG-0012: `handoffs/releases/Q0014-release-notes.md`
- Q0013 / BUG-0010: `handoffs/releases/Q0013-release-notes.md`
- Q0010 / BUG-0006: `handoffs/releases/Q0010-release-notes.md`
- Q0012 / BUG-0005: `handoffs/releases/Q0012-release-notes.md`
- Q0011 / BUG-0004: `handoffs/releases/Q0011-release-notes.md`
- Q0009 / BUG-0003: `handoffs/releases/Q0009-release-notes.md`
- Q0008 / BUG-0002: `handoffs/releases/Q0008-release-notes.md`
- Q0007 / BUG-0001: `handoffs/releases/Q0007-release-notes.md`
- S0015: `handoffs/releases/S0015-release-notes.md`
- S0014: `handoffs/releases/S0014-release-notes.md`
- S0013: `handoffs/releases/S0013-release-notes.md`
- S0012: `handoffs/releases/S0012-release-notes.md`
- S0011: `handoffs/releases/S0011-release-notes.md`
- S0010: `handoffs/releases/S0010-release-notes.md`
- S0009–S0001: `handoffs/releases/S0009-release-notes.md` … `S0001-release-notes.md`

---

## Compatibility behavior contract

- Keep this file as a pointer/summary; do not treat it as canonical historical storage.
- `/release` must update task-scoped notes first (`Qxxxx` or `Sxxxx`), then refresh this pointer.
- Never delete or destructively rewrite historical sprint-scoped note files through this legacy path.
