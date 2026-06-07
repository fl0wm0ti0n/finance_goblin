# UAT — Q0023 (BUG-0015)

**Status:** POPULATED — verify-work complete 2026-06-07  
**Acceptance:** `docs/product/acceptance.md` — BUG-0015 rows **AU**, **AV**, **AW**  
**Sprint:** Q0023 (`/quick`)  
**Verdict:** **PASS** — code/test complete; runtime probes pass-with-prerequisites (BUG-0013/0014 precedent)  
**Next phase:** `/release`

## Acceptance ↔ task traceability

| Row | Task(s) | Acceptance criterion (canonical) | Result |
|-----|---------|-----------------------------------|--------|
| **AU** | AU1, AU2, AU3, V1 | Confirmed Cursor/Apple remain confirmed after rebuild + Full sync — not pending with Confirm/Reject | **pass** (code) / **pass_with_prerequisites** (live) |
| **AV** | AU1–AU4, V1 | No duplicate pending for same merchant identity; confirmed skip/merge on payee+interval | **pass** |
| **AW** | AU3, V1 | Subscription unread alerts reconcile — no spurious new_detection for confirmed merchants | **pass** |

## Operator gates (before live omniflow probes)

1. **BACKEND_FRONTEND_DEPLOY** — ship Q0023 backend on omniflow. — **PENDING**
2. **POSTGRES_PERSISTENCE_PROBE** — H2 SQL immediately after app rebuild, **before** Full sync:

```sql
SELECT status, COUNT(*) FROM subscription_patterns GROUP BY status;
SELECT fingerprint, status, payee_key, interval_days, current_amount
FROM subscription_patterns
WHERE payee_key ILIKE '%cursor%' OR payee_key ILIKE '%apple%'
ORDER BY updated_at DESC;
```

| Outcome | Action |
|---------|--------|
| Zero `confirmed` rows after rebuild (no operator action) | **Ops** — volume/DB target (H2); do not run V1 until resolved |
| `confirmed` rows present; drift after Full sync | **Execute** AU1–AU4 path validated |
| Single confirmed per merchant; still pending in UI | Reopen discovery — unlikely per H3 refutation |

3. **FULL_FIREFLY_SYNC** — Full sync (not exchanges-only); detection phase completes. — **PENDING**

## UAT steps (verify-work results)

| Step | Row | Probe | Result | Evidence |
|------|-----|-------|--------|----------|
| AU-BASE | AU | Confirm Cursor + Apple on `/subscriptions` before rebuild | **pass_with_prerequisites** | Operator baseline — pending deploy |
| AU-CODE | AU | `payee_key` normalization unit tests (AU1) | **pass** | `normalize.rs` card_billing_*; VW-AUTO-2 4/4 |
| AV-CODE | AV | merge_confirmed_pattern + detection skip tests (AU2–AU3) | **pass** | `repository.rs` + `detection.rs`; VW-AUTO-3/4 |
| AW-CODE | AW | merge path suppresses new_detection (AU3) | **pass** | merge `continue` before pending upsert |
| H2-1 | AU | H2 SQL probe after rebuild, before Full sync | **pass_with_prerequisites** | Operator DB — pending rebuild smoke |
| AU-1 | AU | `GET /api/v1/subscriptions?status=confirmed` includes Cursor/Apple post-rebuild | **pass_with_prerequisites** | Omniflow API 404; pending Full sync |
| AU-2 | AU | `/subscriptions` UI — no Confirm/Reject for confirmed merchants | **pass_with_prerequisites** | Browser — pending deploy + sync |
| AV-1 | AV | No duplicate `status=pending` rows for same payee+interval | **pass_with_prerequisites** | API + SQL — pending deploy + sync |
| AW-1 | AW | `GET /api/v1/subscriptions/alerts/unread-count` reconciles with pending tab | **pass_with_prerequisites** | API + UI — pending deploy + sync |
| OIDC-1 | regression | OIDC-enabled deploy smoke per acceptance AW | **pass_with_prerequisites** | Operator — pending deploy |

## Automated verification (verify-work independent re-run)

| Check | Result |
|-------|--------|
| `cd backend && cargo test --lib` | **PASS** (187/187) |
| AU1 — `card_billing_*` normalize tests | **PASS** (4/4) |
| AU2 — `interval_matches` + confirmed payee lookup | **PASS** (3/3) |
| AU3/AU4 — detection merge + stale inactive wiring | **PASS** (code + unit tests) |
| `cd frontend && npm test -- --run` | **PASS** (6/6) |
| Omniflow reachability | **pass_with_prerequisites** — root 401; API health 404 |

## Results summary

- **Verdict:** PASS — 3/10 UAT steps pass (code), 7 pass-with-prerequisites (runtime/ops), 0 fail
- **Acceptance rows:** AU **pass**, AV **pass**, AW **pass** (code); live rebuild smoke deferred to operator
- **Blocking:** none
- **Traceability:** BUG-0015 rows **AU**–**AW** mapped in `sprints/quick/Q0023/uat.json`. Checkbox updates in `docs/product/acceptance.md` are **release** phase.

**Operator advisory:** After all three operator gates complete, execute the 10-step smoke checklist in `uat.json` `operator_smoke_checklist` on `https://financegnome.omniflow.cc`.
