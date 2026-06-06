# Release Notes (Legacy Compatibility Pointer)

This file remains backward-compatible for workflows that read
`handoffs/release_notes.md` as the latest release summary.

Canonical sprint history lives under:
- `handoffs/releases/Sxxxx-release-notes.md`

Canonical quick-task / bug-fix history:
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

- **Latest released sprint:** S0012
- **Latest sprint notes:** [handoffs/releases/S0012-release-notes.md](releases/S0012-release-notes.md)
- **Latest sprint date:** 2026-06-03
- **Latest sprint story:** US-0012
- **Sprint release version:** `0.12.0-us0012`

- **Latest bug fix:** BUG-0007 / Q0017 (2026-06-08)
- **Latest bug notes:** [handoffs/releases/Q0017-release-notes.md](releases/Q0017-release-notes.md)
- **Bug status:** DONE; acceptance S/T/U checked
- **Open bug queue:** BUG-0008, BUG-0011
- **Deferred epics:** US-0013 (ML production), US-0014 (planning UX), US-0015 (AI bucket mapping)

## Unreleased queue visibility

No rows with `status=unreleased` or `status=blocked`. See `handoffs/release_queue.md` for full queue.

## Latest operator summary (Run/Connect/Verify) — BUG-0007 / Q0017

- **Deploy:** `docker compose -f docker-compose.yml -f docker-compose.external.yml --profile external up -d --build flow-finance-ai`
- **Operator gate:** `--force-recreate flow-finance-ai` (BACKEND_DEPLOY)
- **Endpoint:** `https://financegnome.omniflow.cc`
- **Profile rule:** `external` only
- **Env:** `DATABASE_HOST=postgres`
- **Verify:** `cargo test --lib` (150/150); `cargo test --test bug0007_ai_discovery` (8/8); omniflow AI Chat S/U smoke
- **Full steps:** [handoffs/releases/Q0017-release-notes.md](releases/Q0017-release-notes.md)

## Historical references

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
- S0012: `handoffs/releases/S0012-release-notes.md`
- S0011: `handoffs/releases/S0011-release-notes.md`
- S0010: `handoffs/releases/S0010-release-notes.md`
- S0009–S0001: `handoffs/releases/S0009-release-notes.md` … `S0001-release-notes.md`

---

## Compatibility behavior contract

- Keep this file as a pointer/summary; do not treat it as canonical historical storage.
- `/release` must update task-scoped notes first (`Qxxxx` or `Sxxxx`), then refresh this pointer.
- Never delete or destructively rewrite historical sprint-scoped note files through this legacy path.
