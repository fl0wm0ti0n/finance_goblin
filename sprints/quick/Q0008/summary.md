# Sprint Summary — Q0008 (BUG-0002)

**Bug:** BUG-0002  
**Sprint:** Q0008 (`/quick`)  
**Execute date:** 2026-06-04  
**Release date:** 2026-06-05  
**Status:** **DONE** — verify-work PASS; release finalized

## Tasks delivered

| ID | Status | Summary |
|----|--------|---------|
| C2 | done | Empty PAT env guard; `FireflyConfig::pat_configured()`; sync preflight `firefly_personal_access_token_missing`; `/health/ready` `firefly_pat_configured` |
| D1 | done | `GET /api/v1/plans/risk-score` always **200** with `status: ok` \| `no_score` + `reason`; frontend `PlanRiskScoreResponse` + Planning badge gating |
| E1 | done | `effective_enabled()` in settings_view + `mirror_enabled_at_startup` |
| E2 | done | `[exchanges.binance] enabled = false` in `default.toml` (+ code default) |
| C1 | done | Runbook + `.env.example` PAT verification; operator deploy + sync verified at verify-work |

## Tests

| Check | Result |
|-------|--------|
| `cargo test --lib` | **PASS** (103 tests at release) |
| `npm test` (frontend) | **PASS** (2/2) |
| `npm run build` (frontend) | **PASS** |

## Acceptance (verify-work + release)

| Row | Verdict | Evidence |
|-----|---------|----------|
| **(C)** | **PASS** | Sync success; 922 transactions; no 401; sync routes 200 |
| **(D)** | **PASS** | `GET /api/v1/plans/risk-score` → 200 `no_score` / `no_active_plan` |
| **(E)** | **PASS** | Bitunix enabled+configured; Binance disabled; bitunix test 200 |

## Files changed (primary)

- `backend/src/config/mod.rs`
- `backend/src/sync/mod.rs`
- `backend/src/firefly/mod.rs`
- `backend/src/health/mod.rs`
- `backend/src/api/plans.rs`
- `backend/src/exchanges/service.rs`
- `backend/config/default.toml`
- `frontend/src/lib/api.ts`
- `frontend/src/pages/PlanningPage.tsx`
- `docs/engineering/runbook.md`
- `.env.example`

## Release

- **Notes:** `handoffs/releases/Q0008-release-notes.md`
- **Acceptance:** `docs/product/acceptance.md` BUG-0002 checked
- **Backlog:** BUG-0002 DONE — closure_note: verify-work PASS Q0008, 2026-06-05
- **Publish:** skipped (`RELEASE_PUBLISH_MODE=disabled`)

## Next

- **BUG-0003** — omniflow API 500 cascade / Bitunix test / Grafana SQL (Q0009); operator `DATABASE_HOST=postgres` recovery before verify-work
