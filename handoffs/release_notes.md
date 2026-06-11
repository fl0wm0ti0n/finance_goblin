# Release Notes (Legacy Compatibility Pointer)

This file remains backward-compatible for workflows that read
`handoffs/release_notes.md` as the latest release summary.

Canonical sprint history lives under:
- `handoffs/releases/Sxxxx-release-notes.md`

Canonical quick-task / bug-fix history:
- `handoffs/releases/Q0029-release-notes.md` (BUG-0021)
- `handoffs/releases/Q0028-release-notes.md` (BUG-0020)
- `handoffs/releases/Q0027-release-notes.md` (BUG-0019)
- `handoffs/releases/Q0026-release-notes.md` (BUG-0018)
- `handoffs/releases/Q0025-release-notes.md` (BUG-0017)
- `handoffs/releases/Q0024-release-notes.md` (BUG-0016)
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

- **Latest released sprint:** S0019
- **Latest sprint notes:** [handoffs/releases/S0019-release-notes.md](releases/S0019-release-notes.md)
- **Latest sprint date:** 2026-06-10
- **Latest sprint story:** US-0020
- **Sprint release version:** `0.20.0-us0020`
- **Open stories:** (empty — intake bundle backlog drain complete)
- **Open bug queue:** (empty — intake bundle drain complete)

- **Latest released quick task:** Q0029 / BUG-0021 (2026-06-11)
- **Latest quick-task notes:** [handoffs/releases/Q0029-release-notes.md](releases/Q0029-release-notes.md)
- **Bug status:** DONE; acceptance BK, BL checked (browser/API/snapshot deploy operator-deferred)
- **Release version:** `bug0021-q0029`

- **Prior quick task:** Q0028 / BUG-0020 (2026-06-11)
- **Prior quick-task notes:** [handoffs/releases/Q0028-release-notes.md](releases/Q0028-release-notes.md)
- **Release version:** `bug0020-q0028`

- **Prior quick task:** Q0027 / BUG-0019 (2026-06-10)
- **Prior quick-task notes:** [handoffs/releases/Q0027-release-notes.md](releases/Q0027-release-notes.md)
- **Release version:** `bug0019-q0027`

- **Prior quick task:** Q0026 / BUG-0018 (2026-06-10)
- **Prior quick-task notes:** [handoffs/releases/Q0026-release-notes.md](releases/Q0026-release-notes.md)
- **Release version:** `bug0018-q0026`

- **Prior quick task:** Q0025 / BUG-0017 (2026-06-10)
- **Prior quick-task notes:** [handoffs/releases/Q0025-release-notes.md](releases/Q0025-release-notes.md)
- **Release version:** `bug0017-q0025`

- **Prior quick task:** Q0024 / BUG-0016 (2026-06-09)
- **Prior quick-task notes:** [handoffs/releases/Q0024-release-notes.md](releases/Q0024-release-notes.md)
- **Release version:** `bug0016-q0024`

## Unreleased queue visibility

No rows with `status=unreleased` or `status=blocked`. See `handoffs/release_queue.md` for full queue.

## Latest operator summary (Run/Connect/Verify) — BUG-0020 / Q0028

- **Prerequisite:** Fix `ForecastPage.tsx` TS6133 (`hasForecast` unused) — blocks `docker build`
- **Deploy:** `docker compose -f docker-compose.yml -f docker-compose.external.yml --profile external up -d --build flow-finance-ai` (after TS6133 fix)
- **Migration:** migration 016 applied manually at verify-work; register via `sqlx migrate run` after resolving migration 15 checksum conflict
- **Sync:** `curl -X POST http://localhost:18080/api/v1/sync/trigger` (detection regression — REG-DETECT)
- **Verify:** `cd backend && cargo test --test bug0020_subscription_list_quality` (7/7); operator smoke per `sprints/quick/Q0028/uat.json`
- **Endpoint:** `http://localhost:18080/subscriptions` / `https://financegnome.omniflow.cc`
- **Full steps:** [handoffs/releases/Q0028-release-notes.md](releases/Q0028-release-notes.md)

## Prior operator summary — BUG-0019 / Q0027

- **Deploy:** `docker compose restart grafana` (provisioning reload only — no backend rebuild)
- **Sync:** `curl -X POST http://localhost:18080/api/v1/sync/trigger` (Full sync + 0-new-tx incremental for BH regression)
- **Verify:** `cd backend && cargo test --test grafana_provisioning_bug0009` (6/6); operator 8-step smoke per `sprints/quick/Q0027/uat.json`
- **Endpoint:** `http://localhost:13000` (Grafana) / `http://localhost:18080` / `https://financegnome.omniflow.cc`
- **Full steps:** [handoffs/releases/Q0027-release-notes.md](releases/Q0027-release-notes.md)

## Prior operator summary — BUG-0018 / Q0026

- **Deploy:** `docker compose -f docker-compose.yml -f docker-compose.external.yml --profile external up -d --force-recreate flow-finance-ai`
- **Sync:** `curl -X POST http://localhost:18080/api/v1/sync/trigger` (after deploy — **FULL_FIREFLY_SYNC**)
- **Verify:** `cd backend && cargo test --test wealth_alerts_integration` (3/3); operator 7-step smoke per `sprints/quick/Q0026/uat.json` after deploy gates
- **Endpoint:** `http://localhost:18080` / `https://financegnome.omniflow.cc`
- **Full steps:** [handoffs/releases/Q0026-release-notes.md](releases/Q0026-release-notes.md)

## Prior operator summary — BUG-0017 / Q0025

- **Deploy:** `docker compose -f docker-compose.yml -f docker-compose.external.yml --profile external up -d --force-recreate flow-finance-ai`
- **Sync:** `curl -X POST http://localhost:18080/api/v1/sync/trigger` (after deploy — **FULL_FIREFLY_SYNC**)
- **Verify:** `cd backend && cargo test --test forecast_integration` (3/3); operator 9-step smoke per `sprints/quick/Q0025/uat.json` after deploy gates
- **Endpoint:** `http://localhost:18080` / `https://financegnome.omniflow.cc`
- **Full steps:** [handoffs/releases/Q0025-release-notes.md](releases/Q0025-release-notes.md)

## Prior operator summary — US-0020 / S0019

- **Deploy:** `docker compose -f docker-compose.yml -f docker-compose.external.yml --profile external up -d --build flow-finance-ai`
- **Verify:** `cd backend && cargo test --lib` (213/213); `cd frontend && npm test -- --run` (9/9); operator 8-step OIDC checklist per `sprints/S0019/uat.md` after deploy gates
- **Endpoint:** `https://financegnome.omniflow.cc`
- **Full steps:** [handoffs/releases/S0019-release-notes.md](releases/S0019-release-notes.md)

## Historical references

- Q0027 / BUG-0019: `handoffs/releases/Q0027-release-notes.md`
- Q0026 / BUG-0018: `handoffs/releases/Q0026-release-notes.md`
- Q0025 / BUG-0017: `handoffs/releases/Q0025-release-notes.md`
- S0019 / US-0020: `handoffs/releases/S0019-release-notes.md`
- S0018 / US-0019: `handoffs/releases/S0018-release-notes.md`
- S0017 / US-0018: `handoffs/releases/S0017-release-notes.md`
- Q0024 / BUG-0016: `handoffs/releases/Q0024-release-notes.md`
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
