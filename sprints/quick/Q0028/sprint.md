# Q0028 — BUG-0020 Subscriptions list quality (duplicates, uncategorized)

| Field | Value |
|-------|-------|
| **ID** | Q0028 |
| **Type** | `/quick` |
| **Status** | PLANNED |
| **Bug** | BUG-0020 |
| **Created** | 2026-06-11 |
| **Architecture** | `docs/engineering/architecture.md` § BUG-0020 (DEC-0109) |
| **Handoff** | `handoffs/tl_to_dev.md` (`sprint-plan-20260611-q0028-bug0020`) |
| **Acceptance** | `docs/product/acceptance.md` — BUG-0020 rows **BI**, **BJ** |
| **Task count** | 7 (all P0 mandatory) |
| **Next phase** | `/plan-verify` |

## Goal

Close BUG-0020 via the **DEC-0109** data-quality contract: migration **016**
one-time reconcile (YouTube confirmed merge, Strom pending collapse) plus
confirmed-only `display_category_id` backfill (**DA**/**DB**); **All** tab
excludes `rejected`/`inactive` (**DA2**, gate **BI**); forward pending guard in
`detection.rs` (**DA3**); integration tests + operator verify-work close **BI**/**BJ**
gates. Unfiltered `GET /api/v1/subscriptions` unchanged.

## Scope

| Slice | Tasks | Layer |
|-------|-------|-------|
| DA1 — reconcile migration | DA1 | `backend/migrations/016_bug0020_subscription_list_quality.sql` |
| DB1 — display_category backfill | DB1 | same migration file |
| DA2 — All-tab list contract | DA2 | `frontend/src/pages/SubscriptionsPage.tsx` |
| DA3 — forward pending guard | DA3 | `backend/src/subscriptions/detection.rs` |
| T1 — integration tests | T1 | `backend/tests/bug0020_subscription_list_quality.rs` |
| Automated gate | G1 | `cargo test` + migration idempotency smoke |
| Verify | V1 | `uat.md` + operator localhost/omniflow smoke |

**Ops-only (not execute tasks):** Operator **BACKEND_FRONTEND_DEPLOY** +
migration 016 apply on target DB; optional pre-migration SQL probe per
architecture § BUG-0020.

**Out of scope (DEC-0109 forbidden):** API list dedup collapse; list-time
`display_category_id` recompute; pending/rejected backfill; hardcoded merchant
ids; Firefly write-back; `list_patterns` SQL shape change.

## Task summary

| ID | Title | Est. | Depends | Acceptance | Priority |
|----|-------|------|---------|------------|----------|
| DA1 | Migration 016 YouTube merge + Strom pending collapse | 3h | — | **BI** | P0 |
| DB1 | Migration 016 confirmed `display_category_id` backfill | 1.5h | DA1 | **BJ** | P0 |
| DA2 | SubscriptionsPage All-tab exclude rejected/inactive | 1h | — | **BI** | P0 |
| DA3 | detection.rs forward pending guard (confirmed/rejected) | 2h | — | **BI** (regression) | P0 |
| T1 | Integration tests BI/BJ + discover/tags regression | 3h | DA1–DA3 | **BI**, **BJ** | P0 |
| G1 | Automated gate (`cargo test`, migration smoke) | 0.5h | T1 | **BI**, **BJ** static | P0 |
| V1 | verify-work operator smoke | 2h | G1 + deploy | **BI**, **BJ** | P0 |

**Total estimate:** ~13h (11h dev + ~2h operator V1).

## Deploy order

```text
DA1 → DB1 (single migration 016 file; reconcile before backfill)
  ∥ DA2 (frontend) ∥ DA3 (backend detection)
  → T1 integration tests
  → G1 automated gate
  → operator: BACKEND_FRONTEND_DEPLOY + migration 016 apply
  → operator: optional pre/post SQL probes (architecture § BUG-0020)
  → operator: FULL_FIREFLY_SYNC (detection regression — no new YouTube dup)
  → V1 verify-work
```

## Acceptance mapping

| Row | Primary tasks | Verify |
|-----|---------------|--------|
| **BI** | DA1, DA2, DA3, T1, G1, V1 | `GET /api/v1/subscriptions?status=confirmed` ≤1 row per `payee_key`; no duplicate YouTube `display_name`; `/subscriptions` **All** — no triplicate Strom / duplicate YouTube visible rows |
| **BJ** | DB1, T1, G1, V1 | Confirmed samples netflix/kindle → `display_category_id = '18'`; youtube survivor → `'66'`; hgp → `'56'`; florian gabriel → `'3'` (R-0090 oracle) |

## Architecture → sprint mapping

| Architecture ID | Disposition |
|-----------------|-------------|
| DA1 | Task **DA1** |
| DA2 | Task **DA2** |
| DA3 | Task **DA3** |
| DB1 | Task **DB1** |
| T1 | Task **T1** |
| BI/BJ runtime gates | Task **V1** |

## Frozen boundaries

See `task.json` `frozen_boundaries`.
