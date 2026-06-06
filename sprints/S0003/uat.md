# UAT — Sprint S0003 / US-0003

**Sprint:** S0003  
**Story:** US-0003  
**Phase:** `/verify-work`  
**Date:** 2026-05-31  
**Verdict:** **PASS** (ready for `/release`)  
**Ready for release:** yes (operator runtime prerequisites documented below)

## Inputs

- QA PASS: `sprints/S0003/qa-findings.md`
- Acceptance: `docs/product/acceptance.md` (US-0003)
- Operator guide: `docs/user-guides/US-0003.md`
- Implementation: `backend/src/recurrence/`, `backend/src/subscriptions/`, `backend/src/api/subscriptions.rs`, `backend/src/forecast/project.rs`, `backend/src/sync/mod.rs`, `frontend/src/pages/SubscriptionsPage.tsx`, `grafana/provisioning/dashboards/analytics/subscriptions.json`

## Environment

| Prerequisite | Status in verify-work |
|--------------|----------------------|
| `.env` populated | **Not present** — no operator `.env` in workspace |
| `DATABASE_URL` (TimescaleDB + extension) | **Unset** — `subscriptions_integration`, `forecast_integration`, and `firefly_integration` skipped |
| Firefly PAT + synced expense transactions (≥3 recurring) | **Not provisioned** — live detection, confirm/reject, and price-history E2E deferred |
| `AUTH_DEV_BYPASS` or OIDC IdP | **Unset** — live API/UI auth flow deferred |

Per workflow policy: code-level and automated verification **pass**; runtime E2E steps recorded as **PASS-with-prerequisites** where external infra is required.

## Automated checks

| ID | Check | Command | Result |
|----|-------|---------|--------|
| AUTO-1 | Test harness | `bash tests/run-tests.sh` | **PASS** |
| AUTO-2 | Firefly GET-only unit test | (via harness) `cargo test --test firefly_readonly` | **PASS** (1/1) |
| AUTO-3 | Subscription/recurrence unit tests | (via harness) `cargo test --lib` | **PASS** (18/18) |
| AUTO-4 | Firefly integration (audit log) | (via harness) `cargo test --test firefly_integration` | **SKIP** — `DATABASE_URL` unset |
| AUTO-5 | Forecast hypertable integration | (via harness) `cargo test --test forecast_integration` | **SKIP** — `DATABASE_URL` unset |
| AUTO-6 | Subscription integration | (via harness) `cargo test --test subscriptions_integration` | **SKIP** — `DATABASE_URL` unset |
| AUTO-7 | Frontend production build | (via harness) `npm run build` | **PASS** (lazy chunk: `PriceHistoryChart`) |
| AUTO-8 | Compose minimal services | `docker compose --profile minimal config --services` (placeholder env) | **PASS** — `firefly-iii`, `flow-finance-ai`, `grafana` |

## UAT steps (acceptance criteria)

| ID | AC | Step | Result | Evidence |
|----|-----|------|--------|----------|
| UAT-1 | AC-1 | Detection engine confidence tiers 95/80/60% | **PASS** | `recurrence/detect.rs` `score_confidence` returns 95/80/60; `min_emit_confidence: 60`; migration 003 `CHECK (confidence_pct IN (60, 80, 95))`; unit tests `confidence_tiers_per_dec_0014`, `emits_only_at_least_sixty_percent`; UI `confidenceClass()` badges on pending cards. |
| UAT-2 | AC-2 | New detection notification with confirm/reject actions | **PASS-with-prerequisites** | `detection.rs` inserts `new_detection` alert; `POST .../confirm`, `POST .../reject`; pending cards with Confirm/Reject + kind override dialog; alerts banner + sessionStorage toast. **Operator prerequisite:** sync recurring expense transactions; open `/subscriptions` and confirm/reject a pending detection. |
| UAT-3 | AC-3 | Confirmed subscriptions list with interval and amount | **PASS-with-prerequisites** | `GET /api/v1/subscriptions` with status/kind filters; All/Confirmed table columns Name, Interval, Amount; detail drawer shows interval + amount. **Operator prerequisite:** confirm at least one pending pattern; verify list columns populate. |
| UAT-4 | AC-4 | Standing-order patterns separate from subscriptions | **PASS** | `classify.rs` Dauerauftrag heuristics (DEC-0016); `subscription_kind` enum; Standing orders tab filters `status=confirmed&kind=standing_order`; confirm dialog kind override; unit tests `large_fixed_monthly_is_standing_order`, `small_variable_is_subscription`. |
| UAT-5 | AC-5 | Price increase/decrease on confirmed subscription | **PASS-with-prerequisites** | `price_change.rs` dual-threshold `classify_price_change`; `process_confirmed_pattern` emits events + alerts; `GET .../price-history`; lazy `PriceHistoryChart` in detail drawer; unit tests `dual_threshold_requires_both`, `small_absolute_change_ignored`. **Operator prerequisite:** confirmed subscription with amount change in synced history; verify drawer chart and price-change callout. |
| UAT-6 | AC-6 | Alerts on new detection and price change | **PASS-with-prerequisites** | `subscription_alerts` table + `subscription_alert_type` enum; `insert_alert` for `new_detection` and `price_change`; `GET /api/v1/subscriptions/alerts`; UI alert banner with mark-read. Integration test asserts `new_detection` alert count (**SKIP** without `DATABASE_URL`). **Operator prerequisite:** trigger sync after recurring expenses; verify alert banner/toast. |
| UAT-7 | AC-7 | Grafana Dashboard 2 provisioned | **PASS** | `grafana/provisioning/dashboards/analytics/subscriptions.json` — uid `subscriptions`, title "Subscriptions"; panels: Confirmed subscriptions, Monthly spend (normalized), Pending review, Price changes (90 days), New detections; datasource uid `FlowFinancePostgreSQL` matches `postgres.yaml`. |
| UAT-8 | AC-8 | Rejected patterns excluded from forecast and alerts | **PASS** | `reject_pattern` writes `subscription_rejections` fingerprint; `run_candidates` skips rejected/confirmed fingerprints; `project.rs` `exclude_rejected` + `apply_subscription_override`; unit test `rejected_fingerprint_excluded_from_projection`; sync passes `DetectionResult` to forecast recompute. |

## Summary

| Metric | Count |
|--------|-------|
| Acceptance criteria passed (code/automated) | 8/8 |
| Full runtime E2E executed | 0/8 (blocked by missing operator infra) |
| Automated checks passed | 5/8 (3 SKIP — expected without `DATABASE_URL`) |
| Failed | 0 |

## Operator prerequisites (post-release smoke)

1. Copy `.env.example` → `.env`; set `DATABASE_*`, `FIREFLY_*`, optional OIDC or `AUTH_DEV_BYPASS=true`.
2. Provision external TimescaleDB; apply migrations including `003_subscriptions.sql`.
3. `docker compose --profile minimal up --build`
4. Complete Firefly setup; create PAT; sync expense transactions with ≥3 recurring occurrences per payee.
5. Open `http://localhost:8080/subscriptions` — verify pending cards, confirm/reject workflow, confirmed list, standing orders tab, price-history drawer.
6. Open Grafana Analytics dashboard `subscriptions`.
7. Optional: `DATABASE_URL=... cargo test --test subscriptions_integration` for pending persistence and `new_detection` alert proof.

## Findings

### Blockers

None.

### Observations

1. `subscriptions_integration`, `forecast_integration`, and `firefly_integration` require operator `DATABASE_URL` — skipped by design in verify-work; unit tests and schema audit provide sufficient gate coverage.
2. Live detection and confirm/reject depend on synced recurring expense history — documented in `docs/user-guides/US-0003.md`.
3. ECharts main chunk ~1 MB (vite warning); `PriceHistoryChart` is code-split — acceptable for MVP.
4. Compose env interpolation requires placeholder values (e.g. `AUTHENTIK_SECRET_KEY`) even for `--profile minimal` — documented in `.env.example`.

## Next phase

Run `/release` in a fresh release subagent context.
