# Sprint S0020

**ID:** S0020  
**Story:** US-0021 — Subscription transaction explorer with rich filters  
**Status:** RELEASED  
**Created:** 2026-06-13  
**Orchestrator:** `auto-20260613-us0021`

## Goal

Deliver **DEC-0112** transaction search API with SQL push-down filters and 100/page pagination; **DEC-0113** dual-mode Discover UX (Transactions default | Suggested patterns); **DEC-0114** hint pass on filtered subsets; multi-select confirm via preview-group → **DEC-0099** confirm; preserve **DEC-0098** patterns tab; publish `docs/user-guides/US-0021.md`; US-0003/US-0008 regression tests; OIDC smoke template in UAT.

## Scope

| Slice | Tasks | Primary files |
|-------|-------|---------------|
| **US-0021-S1** — Transaction search API | TX1 … TX3 | `repository.rs`, `discovery.rs`, `api/subscriptions.rs` |
| **US-0021-S2** — Transactions mode UI | UI1 … UI4 | `SubscriptionsPage.tsx`, `api.ts` |
| **US-0021-S3** — Patterns tab preservation | PT1 | `SubscriptionsPage.tsx` |
| **US-0021-S4** — Tests + docs | T1, T2, R1 | `subscriptions/` tests, `docs/user-guides/US-0021.md` |
| **V1** — UAT smoke | V1 | `uat.md`, `uat.json` |

**Out of scope:** Firefly write-back; global auto-detection threshold changes; amount band filter (P2); composite index `idx_transactions_account_date` (P2); 2-tx weak hints (GATE-HINT-2 P2); keyset pagination MVP; all-accounts search without cap.

## Task table

| ID | Title | Slice | Est. | Acceptance |
|----|-------|-------|------|------------|
| TX1 | Repository SQL search + COUNT + role JOIN | S1 | 4h | AC-1, AC-2 |
| TX2 | Search service + hint pass | S1 | 4h | AC-3 |
| TX3 | GET search + POST preview-group routes | S1 | 3h | AC-1, AC-4 |
| UI1 | Dual-mode tab shell (DEC-0113) | S2 | 3h | AC-1 |
| UI2 | Rich filter bar | S2 | 4h | AC-2 |
| UI3 | Tx table + pagination + hints | S2 | 4h | AC-1, AC-3 |
| UI4 | Multi-select confirm flow | S2 | 4h | AC-4 |
| PT1 | Patterns sub-tab extraction | S3 | 3h | AC-5 |
| T1 | Search + hint integration tests | S4 | 3h | AC-1, AC-3 |
| T2 | AC-5 regression tests | S4 | 2h | AC-5 |
| R1 | User guide US-0021 | S4 | 2h | — |
| V1 | UAT OIDC smoke + AC-1..AC-6 template | V1 | 2h | AC-6 |

**Total estimate:** ~34h across 12 mandatory tasks (9 P0 + 2 P1 + V1 P0).

## Risks

| Risk | Mitigation | Ref |
|------|------------|-----|
| Dual-mode UI complexity | Transactions default; shared account/payee state | DEC-0113, UI1 |
| Hint pass perf on wide filters | 500 tx hint budget; account required | DEC-0114, TX2 |
| AC-3 sub-threshold expectation | Document MVP boundary; GATE-HINT-2 P2 defer | TX2, UI3 |
| Regression on detection | No `detection.rs` edits; dedicated AC-5 tests | AC-5, T2 |
| Geldbereich JOIN on JSON | DEC-0111 proven COALESCE path | TX1 |
| Sprint over 12 tasks | P2 (amount band, index, weak hints) excluded | architecture § P2 |
| AC-6 operator gate | OIDC smoke pass-with-prerequisites **BACKEND_FRONTEND_DEPLOY** | V1 |

## Definition of Done

- All 12 sprint tasks complete (`TX1` … `V1`; T2/R1 P1 mandatory for release)
- `/plan-verify` coverage: no gaps against `docs/product/acceptance.md` § US-0021 (AC-1..AC-6)
- Transactions mode lists individual expense txs paginated at 100/page with `total_count` + `has_more` (AC-1)
- Rich filters: account, payee, category, Geldbereich, date range (AC-2)
- Hint badges on filtered subset without auto-emit or pending creation (AC-3)
- Multi-select → preview-group → confirm modal → DEC-0099 confirm (AC-4)
- Suggested patterns tab unchanged — DEC-0098 contract frozen (AC-5)
- US-0020 tags/majority + US-0003/US-0008 unchanged; OIDC smoke template (AC-5, AC-6)
- `docs/user-guides/US-0021.md` published (`USER_GUIDE_MODE=1`)

## Architecture references

- `docs/engineering/architecture.md` § US-0021
- `decisions/DEC-0112.md`, `DEC-0113.md`, `DEC-0114.md`
- Extends: DEC-0098, DEC-0099, DEC-0111
- Research: R-0092 §5–8
- Spec-pack: `docs/engineering/spec-pack/US-0021-{design-concept,crs,technical-specification}.md`
- User guide: `docs/user-guides/US-0021.md`
- Discovery: `handoffs/po_to_tl.md#discovery-20260613-us0021`
- Acceptance: `docs/product/acceptance.md` § US-0021
- Handoff: `handoffs/tl_to_dev.md` (`sprint-plan-20260613-us0021-s0020`)

## Sequencing (frozen)

```text
S1: TX1 → TX2 → TX3
S2: UI1 (shell) ∥ TX1 start; UI2–UI4 after TX3
S3: PT1 after UI1
S4: T1 after TX2+TX3; T2 after PT1+UI4; R1 after UI4
V1: after T1+T2+R1
Operator: BACKEND_FRONTEND_DEPLOY → verify-work omniflow smoke (UAT)
```

## Acceptance coverage map

| Row | Tasks | Notes |
|-----|-------|-------|
| AC-1 | TX1, TX2, TX3, UI1, UI3, V1 | Individual txs paginated; not candidates-only |
| AC-2 | TX1, UI2, V1 | category, Geldbereich, date filters |
| AC-3 | TX2, UI3, V1 | hint badge; account 114 SEPA-Lastschrift fixture |
| AC-4 | TX3, UI4, V1 | multi-select → preview-group → confirm |
| AC-5 | PT1, T2, V1 | patterns tab + detection unchanged |
| AC-6 | V1 | OIDC external profile smoke |

## Split decision

- **Why 12 tasks:** Architecture TX1–TX3 + UI1–UI4 + PT1 + T1/T2 + R1 + V1 = 12 = `SPRINT_MAX_TASKS` 12.
- **Why not split S0020a/b:** Single subscription-ops vertical slice extending US-0020; backend routes gate UI filters and confirm.
- **P2 optional:** amount band, composite index, 2-tx weak hints — excluded from mandatory count per GATE-HINT-2/GATE-IDX-1 defer.
- **User guide in R1:** `USER_GUIDE_MODE=1` — separate task avoids bundling with V1.

## Next phase

**Released** — `0.21.0-us0021` (2026-06-13). Operator: **BACKEND_FRONTEND_DEPLOY** → verify-work omniflow smoke (UAT).
