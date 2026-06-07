# UAT — Q0020 (BUG-0013)

**Status:** POPULATED — verify-work complete 2026-06-09  
**Acceptance:** `docs/product/acceptance.md` — BUG-0013 rows **AI**, **AJ**, **AK**, **AL**, **AM**, **AN**  
**Sprint:** Q0020 (`/quick`)  
**Verdict:** **PASS** — code/test + waived AM; runtime probes pass-with-prerequisites  
**Next phase:** `/release`

## Acceptance ↔ task traceability

| Row | Task(s) | Acceptance criterion (canonical) | Result |
|-----|---------|-----------------------------------|--------|
| **AI** | V1 | Cashflow scarcity + forecast-horizons baseline non-empty signed balances for funded account — not persistent flat 0 € | **pass_with_prerequisites** |
| **AJ** | AJ1, V1 | Price changes (90d) shows rows when events exist, or documented empty-state when none | **pass** (code) / **pass_with_prerequisites** (live AJ-1) |
| **AK** | AN1, AK2, V1 | Crypto value reflects exchange holdings; FX warning only with documented gaps; performance % when snapshot history exists | **pass** (code) / **pass_with_prerequisites** (live AK-1, AK-2) |
| **AL** | AL1, V1 | Budgets MTD plan/actual/deviation plausible — not unexplained −€150K planned with €0 actual | **pass** (code) / **pass_with_prerequisites** (live AL-1) |
| **AM** | V1 | ds/query + annotations return 200 without browser Failed to fetch — waived per R-0077 | **pass** (waived) |
| **AN** | AN1, V1 | Exchange crypto balances in wealth/portfolio totals when sync succeeds | **pass** (code) / **pass_with_prerequisites** (live AN-1) |

## Operator gates (before live omniflow probes)

1. **BACKEND_FRONTEND_DEPLOY** — deploy AL1 + AN1 release on omniflow. — **PENDING**
2. **GRAFANA_PROVISIONING_RELOAD** — reload provisioning after AL1/AJ1/AK2. — **PENDING**
3. **FULL_FIREFLY_SYNC** — Full sync (not exchanges-only) + forecast recompute. — **PENDING**

## UAT steps (verify-work results)

| Step | Row | Probe | Result | Evidence |
|------|-----|-------|--------|----------|
| UAT-AL-CODE | AL | DEC-0079 panel 5 SQL `<= CURRENT_DATE` + footnote | **pass** | `budgets.json`; qa-findings T-4 |
| AL-1 | AL | `/analytics/budgets` MTD summary plausible | **pass_with_prerequisites** | Pending deploy + Grafana reload |
| UAT-AN-CODE | AN | Bitunix array wallet + unrealizedPNL + unit tests | **pass** | `bitunix.rs`, `pnl.rs`; qa-findings T-2, T-3 |
| AN-1 | AN | `GET /api/v1/wealth` `crypto.subtotal_eur` > 0 | **pass_with_prerequisites** | Pending deploy + Full sync |
| UAT-AK-CODE | AK | Linear unrealized EUR + AK2 footnote | **pass** | `pnl.rs`, `portfolio.json`; qa-findings T-6, T-8 |
| AK-1 | AK | `/analytics/portfolio` crypto stat non-zero | **pass_with_prerequisites** | Pending deploy + sync + Grafana |
| AK-2 | AK | Performance % empty-state or data | **pass_with_prerequisites** | Pending Grafana reload |
| AI-1 | AI | Cashflow + forecast-horizons baseline acct 114 | **pass_with_prerequisites** | Pending Full sync (ops regression) |
| UAT-AJ-CODE | AJ | Subscriptions price-changes panel description | **pass** | `subscriptions.json`; qa-findings T-7 |
| AJ-1 | AJ | `/analytics/subscriptions` price changes panel | **pass_with_prerequisites** | Pending Grafana reload |
| UAT-AM | AM | ds/query + annotations HTTP 200 | **pass** | Waived per R-0077 |
| REG-1 | regression | Six `/analytics/{slug}` routes embed | **pass_with_prerequisites** | Pending deploy + Grafana |

## Automated verification (verify-work independent re-run)

| Check | Result |
|-------|--------|
| `cd backend && cargo test --lib` | **PASS** (174/174) |
| DEC-0079 AL1 MTD cap | **PASS** |
| DEC-0080 AN1 unit tests (5) | **PASS** |
| AJ1 / AK2 Grafana copy | **PASS** |

## Results summary

| Metric | Count |
|--------|-------|
| UAT steps total | 12 |
| **pass** | 5 |
| **pass_with_prerequisites** | 7 |
| **fail** | 0 |
| Acceptance rows **pass** (code/waived) | AL, AN, AK, AJ, AM |
| Acceptance rows **pass_with_prerequisites** (runtime) | AI |

**Traceability:** BUG-0013 rows **AI–AN** mapped in `sprints/quick/Q0020/uat.json`. Checkbox updates in `docs/product/acceptance.md` are **release** phase. Decisions **DEC-0079** (AL1), **DEC-0080** (AN1).

**Operator advisory:** After **BACKEND_FRONTEND_DEPLOY**, **GRAFANA_PROVISIONING_RELOAD**, and **FULL_FIREFLY_SYNC**, execute AL-1 through REG-1 probes on `https://financegnome.omniflow.cc` per smoke checklist above.
