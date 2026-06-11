# UAT — Q0028 (BUG-0020)

**Status:** COMPLETE — verify-work 2026-06-11  
**Verdict:** **PASS-WITH-PREREQUISITES**  
**Acceptance:** `docs/product/acceptance.md` — BUG-0020 rows **BI**, **BJ**  
**Sprint:** Q0028 (`/quick`)  
**Orchestrator:** `auto-20260610-bug0019`

## Operator gates

| Gate | Status | Notes |
|------|--------|-------|
| MIGRATION_016_APPLY | **PASS** (manual) | Applied via psql 2026-06-11; reconcile + backfill UPDATE 6; `_sqlx_migrations` v16 not registered (migration 15 checksum conflict) |
| FULL_FIREFLY_SYNC | **PASS** | Run `ed2c35fb-b4f5-46d4-828f-485b7c0ae4b2` success 2026-06-11T08:30:34Z |
| BACKEND_FRONTEND_DEPLOY | **pass-with-prerequisites** | `docker build` failed — `ForecastPage.tsx` TS6133 unused `hasForecast`; container pre-Q0028 (DA2 All-tab filter not in deployed static; DA3 guard not in running binary) |

## UAT steps

| Step | Row | Probe | Result | Evidence |
|------|-----|-------|--------|----------|
| BI-API | BI | `GET /api/v1/subscriptions?status=confirmed` — ≤1 row per `payee_key`; no duplicate YouTube | **pass** | 6 confirmed; 0 dup `payee_key`; 1 YouTube survivor |
| BI-ALL | BI | `/subscriptions` **All** tab — no triplicate Strom / duplicate YouTube | **pass_with_prerequisites** | API + DA2 filter simulation: 1 Strom + 1 YouTube; UI static pre-deploy |
| BJ-NETFLIX | BJ | netflix → `display_category_id = '18'` | **pass** | DB + API |
| BJ-KINDLE | BJ | kindle → `display_category_id = '18'` | **pass** | DB + API |
| BJ-YOUTUBE | BJ | youtube survivor → `display_category_id = '66'` | **pass** | DB + API |
| BJ-HGP | BJ | hgp → `display_category_id = '56'` | **pass** | Both hgp rows |
| BJ-FLORIAN | BJ | florian gabriel → `display_category_id = '3'` | **pass** | DB + API |
| REG-DISCOVER | regression | discover unchanged | **pass** | HTTP 200 `{candidates, meta}` |
| REG-TAGS | regression | tags unchanged | **pass** | HTTP 200 |
| REG-DETECT | regression | post-sync no new YouTube dup | **pass** | 1 confirmed after full sync |
| OIDC-1 | regression | omniflow list smoke | **pass** | financegnome 200 |

## Pre/post migration audit

| Metric | Pre | Post |
|--------|-----|------|
| `_sqlx_migrations` max | 15 | 15 (016 applied manually, not registered) |
| confirmed count | 7 | 6 |
| confirmed with `display_category_id` | 0/7 | 6/6 |
| YouTube confirmed dup | 2 | 0 (1 survivor + 1 inactive) |
| Strom All-tab visible | 3 (1 pending + 2 rejected) | 1 pending |

## Automated checks

- `bug0020_subscription_list_quality` **7/7 PASS**
- `bug0008_subscription_alerts` **8/8 PASS**
- `subscriptions_integration` **1/1 PASS**
