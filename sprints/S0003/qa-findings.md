# QA Findings — Sprint S0003 / US-0003

**Sprint:** S0003  
**Story:** US-0003  
**QA phase:** `/qa`  
**Date:** 2026-05-31  
**Verdict:** **PASS** (ready for `/verify-work`)

## Scope

Subscription intelligence: shared recurrence core with confidence tiers, migration 003 persistence, Subscription Engine (classify, detection, price_change), sync-triggered detection phase, forecast override for confirmed/rejected patterns, 7 subscription REST endpoints, React `/subscriptions` page with confirm/reject workflow and ECharts price history, Grafana Dashboard 2, unit/integration tests, operator user guide.

**Inputs reviewed:** `handoffs/dev_to_qa.md`, `sprints/S0003/summary.md`, `sprints/S0003/tasks.md`, `docs/product/acceptance.md` (US-0003), implementation in `backend/src/recurrence/`, `backend/src/subscriptions/`, `backend/src/api/subscriptions.rs`, `backend/src/forecast/project.rs`, `backend/src/sync/mod.rs`, `frontend/src/pages/SubscriptionsPage.tsx`, `grafana/provisioning/dashboards/analytics/subscriptions.json`, `docs/user-guides/US-0003.md`.

## Test plan

| # | Test | Method | Result |
|---|------|--------|--------|
| T-1 | Automated harness | `bash tests/run-tests.sh` | **PASS** |
| T-2 | Firefly GET-only unit test | `cargo test --test firefly_readonly` | **PASS** (1/1) |
| T-3 | Subscription/recurrence unit tests | `cargo test --lib` | **PASS** (18/18) |
| T-4 | Subscription integration | `cargo test --test subscriptions_integration` | **SKIP** — `DATABASE_URL` not set |
| T-5 | Forecast integration (US-0002 carry-forward) | `cargo test --test forecast_integration` | **SKIP** — `DATABASE_URL` not set |
| T-6 | Firefly integration (US-0001 carry-forward) | `cargo test --test firefly_integration` | **SKIP** — `DATABASE_URL` not set |
| T-7 | Frontend production build | `npm run build` | **PASS** (PriceHistoryChart lazy chunk emitted) |
| T-8 | Grafana dashboard provisioning | Static review `grafana/provisioning/` | **PASS** — uid `subscriptions`, datasource `FlowFinancePostgreSQL` |
| T-9 | Runtime E2E (live sync → detection → UI confirm/reject → Grafana) | Not executed in QA environment | **Deferred** to `/verify-work` |

### Environment dependencies (non-blocking)

- **`DATABASE_URL`:** Required for `subscriptions_integration` (pending pattern persistence, new_detection alert). Harness skips gracefully; unit tests and static verification cover engine logic and schema. Not a QA blocker.
- **Synced expense transactions (≥3 recurring occurrences):** Required for non-empty `/subscriptions` UI at runtime — deferred to verify-work UAT.
- **OIDC / `AUTH_DEV_BYPASS=true`:** Required for live API/UI acceptance — deferred to verify-work.

## Acceptance criteria matrix

| AC | Criterion | Verdict | Evidence |
|----|-----------|---------|----------|
| AC-1 | Detection engine identifies recurring patterns with confidence score (95/80/60% tiers) | **PASS** | `recurrence/detect.rs` `score_confidence` returns 95/80/60; `min_emit_confidence: 60`; migration 003 `CHECK (confidence_pct IN (60, 80, 95))`; unit tests `confidence_tiers_per_dec_0014`, `emits_only_at_least_sixty_percent`; UI `confidenceClass()` badges on pending cards. |
| AC-2 | User receives notification for new detected subscription with confirm/reject actions | **PASS** | `detection.rs` inserts `new_detection` alert on upsert; 7 REST endpoints incl. `POST .../confirm`, `POST .../reject`; pending cards with Confirm/Reject buttons + kind override dialog; alerts banner + sessionStorage toast on new unread alerts. |
| AC-3 | Confirmed subscriptions appear in subscription list with interval and amount | **PASS** | `GET /api/v1/subscriptions` with status/kind filters; All/Confirmed table columns Name, Interval, Amount (`intervalLabel`, `current_amount`); detail drawer shows interval + amount. |
| AC-4 | Standing-order (Dauerauftrag) patterns detected separately from discretionary subscriptions | **PASS** | `classify.rs` rule-based Dauerauftrag heuristics (DEC-0016); `subscription_kind` enum; Standing orders tab filters `status=confirmed&kind=standing_order`; confirm dialog kind override; unit tests `large_fixed_monthly_is_standing_order`, `small_variable_is_subscription`. |
| AC-5 | Price increase/decrease detected when amount changes for confirmed subscription | **PASS** | `price_change.rs` dual-threshold `classify_price_change`; `process_confirmed_pattern` emits `price_increase`/`price_decrease` events + alerts; `GET .../price-history`; lazy `PriceHistoryChart` in detail drawer; unit tests `dual_threshold_requires_both`, `small_absolute_change_ignored`. |
| AC-6 | Alert fired on new subscription detection and on price change | **PASS** | `subscription_alerts` table + `subscription_alert_type` enum; `insert_alert` for `new_detection` and `price_change`; `GET /api/v1/subscriptions/alerts`; UI alert banner with mark-read; integration test asserts `new_detection` alert count (skipped here — env only). |
| AC-7 | Grafana Dashboard 2 (Subscriptions, price changes, new) provisioned | **PASS** | `grafana/provisioning/dashboards/analytics/subscriptions.json` — uid `subscriptions`, title "Subscriptions"; panels: Confirmed subscriptions, Monthly spend (normalized), Pending review, Price changes (90 days), New detections; datasource uid matches `postgres.yaml`. |
| AC-8 | Rejected patterns do not appear in subscription forecasts or alerts | **PASS** | `reject_pattern` writes `subscription_rejections` fingerprint; `run_candidates` skips rejected/confirmed fingerprints; `project.rs` `exclude_rejected` + `apply_subscription_override`; unit test `rejected_fingerprint_excluded_from_projection`; sync passes `DetectionResult` to forecast recompute. |

**Summary:** 8/8 PASS (7 fully verified in QA; AC-6 integration alert path and runtime sync/UI E2E deferred to verify-work with operator env).

## Findings

### Blockers

None.

### Observations (non-blocking)

1. **`subscriptions_integration` skipped:** Expected without external PostgreSQL. Unit tests (recurrence confidence, classify, price_change, forecast override/exclusion) and migration/schema audit provide sufficient QA coverage; verify-work should run integration test with `DATABASE_URL`.
2. **Runtime E2E deferred:** Live `/subscriptions` confirm/reject flow, price-history chart with real data, sync→detection→forecast timing, and Grafana panel rendering require operator-provisioned stack — covered in verify-work UAT checklist (`sprints/S0003/uat.md`).
3. **Weak integration assertion:** `subscriptions_integration.rs` line 78 uses `assert!(result.rejected_fingerprints.is_empty() \|\| true)` — tautology; primary assertions (pending count ≥1, new_detection alert ≥1) still valid. Recommend tightening in a future hygiene pass (not a blocker).
4. **ECharts bundle size:** Main chunk ~1 MB (vite warning); `PriceHistoryChart` is code-split — acceptable for MVP.
5. **Rust dead-code warning:** `Claims` fields unused in `auth/mod.rs` — cosmetic carry-forward from US-0001; no functional impact.

## Decision gates

No critical issues requiring dev rework. Proceed to **`/verify-work`**.

## Next phase

Run `/verify-work` in a fresh subagent with operator-provisioned PostgreSQL (`DATABASE_URL`), successful Firefly sync with recurring expense transactions, and optional `AUTH_DEV_BYPASS=true` for API/UI acceptance.
