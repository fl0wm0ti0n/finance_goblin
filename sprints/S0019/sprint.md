# Sprint S0019

**ID:** S0019  
**Story:** US-0020 — Subscription manual discovery, majority category & operator tags  
**Status:** PLANNED  
**Created:** 2026-06-10  
**Orchestrator:** `auto-20260608-us0020-001`

## Goal

Deliver **DEC-0098** discover explorer API + Discover tab; **DEC-0099** manual confirm-from-discover with DEC-0085 merge; **DEC-0100** majority `display_category_id` + badge; **DEC-0101**/**DEC-0102** operator tag schema, CRUD, assign, and list filter; publish `docs/user-guides/US-0020.md`; US-0003/US-0008 regression tests; OIDC smoke template in UAT. Optional **DEC-0103** Grafana `$tag` (P2).

## Scope

| Slice | Tasks | Primary files |
|-------|-------|---------------|
| **US-0020-S1** — Discover explorer | T-0198 … T-0200 | `migrations/`, `subscriptions/discovery.rs`, `api/subscriptions.rs`, `SubscriptionsPage.tsx` |
| **US-0020-S2** — Manual confirm | T-0201 | `repository.rs`, `service.rs`, `api/subscriptions.rs` |
| **US-0020-S3** — Majority category | T-0202 … T-0203 | `repository.rs`, `SubscriptionsPage.tsx` |
| **US-0020-S4** — Tags backend | T-0204 … T-0205 | `api/subscription_tags.rs`, `api/subscriptions.rs` |
| **US-0020-S5** — Tags UI | T-0206 | `SubscriptionsPage.tsx` |
| **US-0020-S6** — Docs + regression | T-0207 … T-0208 | `docs/user-guides/US-0020.md`, `subscriptions/` tests |
| **US-0020-S7** — Optional Grafana | T-0210 | `subscriptions.json` |
| **V1** — UAT smoke | T-0209 | `uat.md`, `uat.json` |

**Out of scope:** Firefly tag/category write-back; operator override of display category; per-account tags; changes to auto-detection thresholds; paginated discover beyond 50; amount band filter unless P2 capacity (DEC-0098 stretch).

## Task table

| ID | Title | Slice | Est. | Acceptance |
|----|-------|-------|------|------------|
| T-0198 | Migration `display_category_id` + tag tables | S1/S3/S4 | 3h | AC-5 |
| T-0199 | Discover service + GET `/discover` route | S1 | 4h | AC-1 |
| T-0200 | Discover tab UI (account + payee + interval) | S1 | 4h | AC-1 |
| T-0201 | POST `discover/confirm` + merge (DEC-0085) | S2 | 4h | AC-2 |
| T-0202 | Majority category compute + merge refresh | S3 | 3h | AC-3, AC-5 |
| T-0203 | Majority category badge + tooltip UI | S3 | 3h | AC-3 |
| T-0204 | Operator tag CRUD API | S4 | 3h | AC-4, AC-5 |
| T-0205 | PUT tag assign + list `?tag=` filter | S4 | 3h | AC-4 |
| T-0206 | Tag manager modal + filter chips UI | S5 | 4h | AC-4 |
| T-0207 | User guide US-0020 | S6 | 2h | — |
| T-0208 | US-0003/US-0008 detection + dedup regression | S6 | 2h | AC-6 |
| T-0209 | UAT OIDC smoke + AC-1..AC-6 template | V1 | 2h | AC-6 |
| T-0210 | Optional Grafana `$tag` variable | S7 | 2h | — |

**Total estimate:** ~37h across 12 mandatory tasks (T-0210 P2 optional).

## Risks

| Risk | Mitigation | Ref |
|------|------------|-----|
| Explorer perf on 365d window | Account filter required in UI; cap 50 | DEC-0098, T-0200 |
| Manual confirm bypasses rejection maps | 409 on rejected payee-interval | DEC-0099, T-0201 |
| DEC-0085 merge + category drift | Recompute majority on merge | DEC-0100, T-0202 |
| `mode()` tie ambiguity | Explicit RANK policy in tooltip | DEC-0100, T-0203 |
| Tag delete surprise | Confirm dialog on hard delete | DEC-0101, T-0206 |
| Detection regression | No `run_candidates` changes; dedicated tests | AC-6, T-0208 |
| Grafana stretch slips | DEC-0103 P2; SPA `?tag=` sufficient | T-0210 |
| AC-6 operator gate | OIDC smoke pass-with-prerequisites **BACKEND_FRONTEND_DEPLOY** | T-0209 |

## Definition of Done

- All 11 mandatory sprint tasks complete (`T-0198` … `T-0208`, `T-0209`; T-0210 conditional P2)
- `/plan-verify` coverage: no gaps against `docs/product/acceptance.md` § US-0020 (AC-1..AC-6)
- Discover tab filters by account, payee, interval; results capped at 50 with meta (AC-1)
- Manual confirm creates/merges confirmed subscription without pending-only path (AC-2)
- `display_category_id` set via RANK majority; badge + tie-break tooltip (AC-3)
- Tag CRUD, multi-assign per subscription, list filter by tag slug (AC-4)
- Tags + majority metadata in product DB only — no Firefly write-back (AC-5)
- US-0003 detection + US-0008 alert dedup unchanged; OIDC smoke template (AC-6)
- `docs/user-guides/US-0020.md` published (`USER_GUIDE_MODE=1`)

## Architecture references

- `docs/engineering/architecture.md` § US-0020
- `decisions/DEC-0098.md` through `DEC-0103.md`
- Research: R-0085, R-0080; frozen DEC-0084, DEC-0085, DEC-0086, DEC-0087
- Spec-pack: `docs/engineering/spec-pack/US-0020-{design-concept,crs,technical-specification}.md`
- User guide: `docs/user-guides/US-0020.md`
- Discovery: `handoffs/po_to_tl.md#discovery-20260609-us0020`
- Acceptance: `docs/product/acceptance.md` § US-0020
- Handoff: `handoffs/tl_to_dev.md` (`sprint-plan-20260610-s0019-us0020`)

## Sequencing (frozen)

```text
S1: T-0198 → T-0199 → T-0200
S2: T-0201 (after T-0199)
S3: T-0202 → T-0203 (after T-0198 + T-0201)
S4: T-0204 → T-0205 (after T-0198; ∥ S2/S3 after migration)
S5: T-0206 (after T-0205)
S6: T-0207 → T-0208 (after S1–S5)
S7: T-0210 optional after T-0205
V1: T-0209 after T-0208
Operator: BACKEND_FRONTEND_DEPLOY → verify-work omniflow smoke (UAT)
```

## Acceptance coverage map

| Row | Tasks | Notes |
|-----|-------|-------|
| AC-1 | T-0199, T-0200, T-0209 | Discover search: account + payee + interval; cap 50 |
| AC-2 | T-0201, T-0209 | Manual confirm without auto-detection-only path |
| AC-3 | T-0202, T-0203, T-0209 | Majority display category; RANK tie-break in tooltip |
| AC-4 | T-0204, T-0205, T-0206, T-0209 | Tag CRUD; multi-assign; list filter |
| AC-5 | T-0198, T-0202, T-0204, T-0209 | Product DB overlay; no Firefly write-back |
| AC-6 | T-0207, T-0208, T-0209 | US-0003/US-0008 unchanged; OIDC smoke |

## Split decision

- **Why 12 tasks:** Architecture M1 + D1–D2 + C1–C3 + T1–T3 + R1–R2 + V1 = 12 = `SPRINT_MAX_TASKS` 12.
- **Why not split S0019a/b:** Single subscription-ops vertical slice; confirm gates majority; tags parallel after migration.
- **P2 optional:** T-0210 (Grafana `$tag`) does not block MVP — SPA `?tag=` filter sufficient per DEC-0103.
- **User guide in R1:** `USER_GUIDE_MODE=1` — separate task avoids bundling with V1.

## Next phase

`/plan-verify` in fresh subagent/chat (role: qa)
