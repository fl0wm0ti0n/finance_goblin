# Q0018 — BUG-0008 subscription alerts & detection recall

| Field | Value |
|-------|-------|
| **ID** | Q0018 |
| **Type** | `/quick` |
| **Status** | PLANNED |
| **Bug** | BUG-0008 |
| **Created** | 2026-06-08 |
| **Architecture** | `architecture-20260608-bug0008` (`docs/engineering/architecture.md` § BUG-0008) |
| **Handoff** | `handoffs/tl_to_dev.md` (`sprint-plan-20260608-q0018-bug0008`) |
| **Acceptance** | `docs/product/acceptance.md` — BUG-0008 rows **W**, **X** |
| **Task count** | 12 |
| **Next phase** | `/plan-verify` |

## Goal

Close BUG-0008 on US-0010 external omniflow: **DEC-0071** W bundle (fingerprint dedup migration, `upsert_alert`, unread-count API, orphan lifecycle, `/subscriptions` banner/toast) then **DEC-0072** X Phase 1 (payee normalization, transfer counterparty priority, 730-day window); backend + frontend tests (**W7**, **X4**); operator verify (**V1**) on `financegnome.omniflow.cc` after deploy.

## Scope

| Slice | Tasks | Layer |
|-------|-------|-------|
| W — Alert dedup & unread count (DEC-0071) | W1–W7 | backend migration, subscriptions, frontend |
| X — Detection recall Phase 1 (DEC-0072) | X1–X4 | backend recurrence, config, tests |
| Verify | V1 | uat + operator omniflow smoke |

**Out of scope:** Phase 2 category grouping (**X5** — follow-up if capacity); combined header bell; AI in-pipeline detection; `min_emit_confidence` TOML wiring; BUG-0007 AI tool changes; `list_patterns` filter changes (R-0065 coordinate).

## Task summary

| ID | Title | Est. | Depends | Acceptance hook |
|----|-------|------|---------|-----------------|
| W1 | Fingerprint migration + backfill dedupe | 3h | — | **W** |
| W2 | `upsert_alert` repository | 2h | W1 | **W** |
| W3 | Detection emit gate | 2h | W2 | **W** |
| W4 | Unread-count API route | 2h | W3 | **W** |
| W5 | Orphan lifecycle hooks | 1.5h | W3 | **W** |
| W6 | Frontend banner + toast | 2h | W4 | **W** |
| W7 | Backend dedup + lifecycle tests | 3h | W1–W5 | **W** regression |
| X1 | Payee normalization | 3h | W3 | **X** |
| X2 | Transfer counterparty priority | 2h | X1 | **X** |
| X3 | `detection_window_days` 730 | 0.5h | — | **X** |
| X4 | Forecast + subscription integration tests | 2h | X1–X3 | **X** regression |
| V1 | verify-work omniflow smoke | 1h | W6, X4 deploy | **W**, **X** |

**Total estimate:** ~24h (~23h dev + ~1h operator V1).

## Deploy order

```text
(W1 → W2 → W3) migration + repository + detection gate
→ (W4 → W5) API + lifecycle
→ W6 frontend
→ (X1 → X2 → X3) recurrence core
→ (W7 → X4) tests
→ deploy backend + frontend on omniflow
→ V1 verify-work
```

**Sequencing rule:** W1–W3 before X1 (W-before-X mandatory). X3 may parallelize with W4–W6 once W3 lands.

## Acceptance mapping

| Row | Primary tasks | Verify |
|-----|---------------|--------|
| **W** | W1–W7, V1 | Reconciled `unread_new_detection` vs pending; no 33-vs-11 class mismatch; banner uses unread-count API |
| **X** | X1–X4, V1 | Patterns > 12 baseline; `unread_new_detection <= pending_patterns`; improved recall without alert spam |
| Regression | W7, X4, V1 | OIDC-enabled + bundled-firefly deploy checks pass |

## Architecture → sprint mapping

| Architecture ID | Disposition |
|-----------------|-------------|
| W1 | Task **W1** |
| W2 | Task **W2** |
| W3 | Task **W3** |
| W4 | Task **W4** |
| W5 | Task **W5** |
| W6 | Task **W6** |
| W7 | Task **W7** |
| X1 | Task **X1** |
| X2 | Task **X2** |
| X3 | Task **X3** |
| X4 | Task **X4** |
| V1 | Task **V1** |

## Frozen boundaries

See `task.json` `frozen_boundaries`.
