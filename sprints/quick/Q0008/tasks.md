# Tasks — Q0008 (BUG-0002)

**Bug:** BUG-0002  
**Task count:** 5 (within `SPRINT_MAX_TASKS=12`)

| ID | Title | Est. | Status | Acceptance |
|----|-------|------|--------|------------|
| C2 | Empty PAT env guard + sync fail-fast | 3h | open | **(C)** code |
| D1 | Risk-score 200 empty-state + Planning types | 4h | open | **(D)** |
| E1 | Effective enabled in settings_view + mirror | 2h | open | **(E)** |
| E2 | default.toml binance.enabled=false | 0.5h | open | **(E)** greenfield |
| C1 | Operator PAT + runbook/compose verification | 1h | open | **(C)** operator |

---

## C2 — Empty PAT env guard + sync fail-fast

**Status:** open  
**Depends on:** —  
**Estimate:** 3h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0002 **(C)** — code path (no `Authorization: Bearer ` 401 when var present-but-empty)

### Description

| Layer | Change |
|-------|--------|
| `backend/src/config/mod.rs` ~894 | Skip `set_override` when `FIREFLY_PERSONAL_ACCESS_TOKEN` trim is empty |
| `FireflyConfig` | `pat_configured() -> bool` |
| `backend/src/sync/mod.rs` | Before Firefly HTTP: if sync will run and `!pat_configured()`, fail with `error_message` containing `firefly_personal_access_token_missing` + runbook hint (no token in logs) |
| Optional | `/health/ready` → `firefly_pat_configured: bool` |

### Done when

- [ ] Unit test: empty PAT env → `pat_configured() == false`
- [ ] Unit/integration: sync preflight sets stable error code when PAT missing
- [ ] `cargo test` for touched modules PASS
- [ ] No PAT value in logs or error responses

---

## D1 — Risk-score 200 tagged empty-state + Planning types

**Status:** open  
**Depends on:** —  
**Estimate:** 4h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0002 **(D)** — `GET /api/v1/plans/risk-score` returns **200** with `status: ok` or `no_score` + `reason`

### Description

**Backend** (`backend/src/api/plans.rs`, optional `plan/risk.rs`):

- Always HTTP **200**
- `status: "ok"` — score, band, components, plan_computation_id
- `status: "no_score"` — `reason: "no_active_plan" | "not_computed"`

**Frontend:**

- `frontend/src/lib/api.ts` — `PlanRiskScoreResponse` discriminated union
- `frontend/src/pages/PlanningPage.tsx` — badge only when `status === "ok"`; no query hard-error on `no_score`

### Done when

- [ ] Rust test or curl fixture: 200 + `no_score` when no active plan / not computed
- [ ] Rust test: 200 + `ok` when score exists (regression)
- [ ] `npm run build` PASS
- [ ] Planning page tolerates empty score (no 404-driven error UI)

---

## E1 — Effective enabled in settings_view + startup mirror

**Status:** open  
**Depends on:** —  
**Estimate:** 2h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0002 **(E)** — Bitunix-only env → `enabled: true`, `configured: true`

### Description

```rust
fn effective_enabled(ex) -> bool {
    ex.configured() || ex.enabled
}
```

Apply in `ExchangesConfig::settings_view()` and `ExchangeService::mirror_enabled_at_startup()`.

**Files:** `backend/src/config/mod.rs`, `backend/src/exchanges/service.rs`

### Done when

- [ ] Unit test or integration: configured-without-toml-enabled → effective true
- [ ] `GET /api/v1/settings` reflects effective enabled for Bitunix-only fixture
- [ ] Sync still rejects missing/invalid API keys (no bypass)

---

## E2 — default.toml binance.enabled=false

**Status:** open  
**Depends on:** E1 (same PR recommended)  
**Estimate:** 0.5h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0002 **(E)** — greenfield: Binance not falsely enabled+configured without env

### Description

Set `[exchanges.binance] enabled = false` in `backend/config/default.toml`. Bybit/bitunix defaults unchanged.

### Done when

- [ ] Greenfield settings: Binance `enabled=false` without env keys (with E1: not shown as enabled+configured)
- [ ] Shipped in same deploy as E1

---

## C1 — Operator PAT + runbook/compose verification

**Status:** open  
**Depends on:** C2 deployed to target host  
**Estimate:** 1h (operator)  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0002 **(C)** — manual/scheduled sync success; entity counts; no blocking sync API 404

### Description

| Step | Action |
|------|--------|
| 1 | Non-empty `FIREFLY_PERSONAL_ACCESS_TOKEN` in operator `.env` |
| 2 | Recreate `flow-finance-ai`; `printenv FIREFLY_PERSONAL_ACCESS_TOKEN` — non-empty (**do not log value**) |
| 3 | Manual sync success; Sync Status entity counts > 0; no 401 in `last_run.error_message` |

**Docs:** `docs/engineering/runbook.md` § Omniflow PAT, `.env.example` comment

### Done when

- [ ] Runbook + `.env.example` updated (names only; no secrets)
- [ ] Operator evidence: PAT var present in container; sync completed on omniflow (or documented blocker)

---

## Execution order

1. **C2**, **D1**, **E1+E2** — parallel-safe dev work → single image deploy
2. **C1** — operator PAT after deploy
3. **Smoke** — BUG-0002 rows C, D, E + regression footer

## Split decision

- **Why 5 tasks:** Maps architecture slices C1/C2, D1, E1/E2; C1 separated as ops gate.
- **Why not split Q0008a/b:** All slices share one omniflow deploy acceptance contract; 5 ≪ 12 threshold.
- **No new DEC:** Behavioral fixes under DEC-0004 / DEC-0054 / R-0032.
