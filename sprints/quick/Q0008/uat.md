# UAT — Q0008 (BUG-0002)

**Status:** Verify-work **PASS** (2026-06-05 re-run) — ready for `/release`  
**Orchestrator:** `auto-20260605-bug0002-002`  
**Acceptance:** `docs/product/acceptance.md` — BUG-0002 rows **(C)**, **(D)**, **(E)** (checkbox pending release)  
**Plan-verify:** `sprints/quick/Q0008/plan-verify.json` — PASS  
**QA:** `sprints/quick/Q0008/qa-findings.md` — PASS (code)  
**Verify-work:** `sprints/quick/Q0008/verify-work-findings.md` — **PASS** (live)

| Row | Task(s) | QA (code) | Verify-work (live 2026-06-05 re-run) |
|-----|---------|-----------|----------------------------------------|
| **(C)** | C1, C2 | **PASS** code/docs | **PASS** — sync success; 922 transactions; no 401 |
| **(D)** | D1 | **PASS** unit/static | **PASS** — `risk-score` HTTP **200** `no_score` |
| **(E)** | E1, E2 | **PASS** unit/static | **PASS** — Bitunix enabled+configured; Binance disabled |
| Regression | all | **PASS** static | **DEFERRED** — OIDC browser advisory |

### Verify-work automated evidence

| Check | Result |
|-------|--------|
| `cargo test --lib` | **PASS** (103/103) |
| `npm test` | **PASS** (2/2) |
| `npm run build` | **PASS** |
| Omniflow curl C/D/E | **PASS** |

### Live smoke summary

| Endpoint | HTTP | Key fields |
|----------|------|------------|
| `/api/v1/sync/status` | 200 | `last_run.status: success`, `error_message: null` |
| `/api/v1/sync/entities` | 200 | transactions: 922 |
| `/api/v1/plans/risk-score` | 200 | `status: no_score`, `reason: no_active_plan` |
| `/api/v1/settings` | 200 | bitunix enabled+configured; binance disabled |
| `/api/v1/exchanges/bitunix/test` | 200 | `ok: true` |
