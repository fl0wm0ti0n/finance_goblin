# sprint-plan-20260614-bug0025-q0034 — BUG-0025 Firefly Stromkosten mirror lag

**From:** Tech Lead  
**To:** QA (`/plan-verify`) → Dev (`/execute`)  
**Date:** 2026-06-14  
**Work item:** BUG-0025 (bug)  
**Sprint:** `/quick` **Q0034**  
**Orchestrator run:** `auto-20260613-bug0025`  
**Next phase:** `/plan-verify` (role: qa)

### Summary

Sprint **Q0034** materialized from architecture § BUG-0025: **DEC-0002** backdated skip root cause (category **146** mirror **2026-05** only) plus misleading Sync Status hero when exchange-only runs succeed. **B1** adds manual Full **365d** lookback on `trigger=manual` (**GATE-OVERLAP-1**). **B2**/**F1** split **`last_firefly_run`** from exchange-only `last_run` (**GATE-SYNC-UX-1**). **D1** documents cursor-reset remediation (**GATE-REMED-1**). **T1** integration repro (**GATE-TEST-1**). Closes **BW**/**BX**/**BY**. Extends **DEC-0002** — **GATE-DEC-1 closed**, no new DEC.

**Decisions:** extends **DEC-0002** — **GATE-DEC-1 closed**  
**Research:** **R-0097** §1–9  
**Architecture ref:** `docs/engineering/architecture.md` § **BUG-0025**  
**Spec-pack:** `docs/engineering/spec-pack/BUG-0025-{design-concept,crs,technical-specification}.md`  
**Sprint artifacts:** `sprints/quick/Q0034/*`

### Task tree

| ID | Title | Row | Files | Gate | Priority |
|----|-------|-----|-------|------|----------|
| B1 | Manual 365d lookback wiring | **BW**, **BX** | `backend/src/firefly/mod.rs`, `backend/src/sync/mod.rs` | GATE-OVERLAP-1 | P0 |
| B2 | `last_firefly_run` API split | **BY** | `backend/src/sync/mod.rs`, `frontend/src/lib/api.ts` | GATE-SYNC-UX-1 | P0 |
| F1 | Sync Status hero + DEC-0002 callout | **BX**, **BY** | `frontend/src/pages/SyncStatusPage.tsx` | GATE-SYNC-UX-1 | P0 |
| D1 | Runbook backdated-import remediation | **BX** | `docs/engineering/runbook.md` | GATE-REMED-1 | P0 |
| T1 | Integration backdated-window repro | **BW** | `backend/tests/` | GATE-TEST-1 | P0 |
| G1 | Automated gate | all | cargo test, npm test, build | — | P0 |
| V1 | verify-work BW/BX/BY + OIDC smoke | all | `sprints/quick/Q0034/uat.md` | — | P0 |

**Mandatory count:** 7 (B1, B2, F1, D1, T1, G1, V1) — 7/12 under `SPRINT_MAX_TASKS`.

### Acceptance traceability

| Row | Tasks | Verify |
|-----|-------|--------|
| **BW** | B1, T1, G1, V1 | Manual **Sync now** → multi-month Stromkosten in mirror; expense-series category **146** bars per month |
| **BX** | B1, D1, F1, G1, V1 | Manual Full ingest **or** callout + runbook explain DEC-0002 + cursor reset |
| **BY** | B2, F1, G1, V1 | Hero **Last Firefly sync**; exchange secondary; **Sync now** → `manual` in history |

### Execute order

```text
B1 ∥ B2 (parallel backend)
  → F1 (needs B2 API field)
  → D1 (after B1 contract frozen)
  → T1 (needs B1)
  → G1
  → operator: BACKEND_REBUILD + FRONTEND_DEPLOY
  → V1
```

### Must-not-break

- **DEC-0002** scheduled path — `watermark − overlap_days` unchanged for `trigger=scheduled`
- Upsert + watermark contract — no duplicate rows on cursor reset
- Exchange-only runs — must not update **Last Firefly sync** hero
- CategoryTrendChart / expense-series SQL — mirror ingest fix only (**H3 ruled out**)
- **BUG-0006** category_id ingest — not reopened

### Operator gates (V1)

1. **BACKEND_REBUILD** — B1 + B2 live.
2. **FRONTEND_DEPLOY** — F1 Sync Status UX live.
3. **BW:** Manual **Sync now** → `/forecast` Category spending trend **Wohnen - Stromkosten** multi-month bars.
4. **BX:** `/sync` DEC-0002 callout + runbook anchor.
5. **BY:** Hero uses `last_firefly_run`; history `trigger` column distinguishes run kinds.
6. OIDC smoke per acceptance **BW**/**BX**/**BY**.

### Blockers

None — sprint-plan complete; ready for `/plan-verify`.

---

# sprint-plan-20260611-q0029-bug0021 — BUG-0021 Frontend UX polish

**From:** Tech Lead  
**To:** QA (`/plan-verify`) → Dev (`/execute`)  
**Date:** 2026-06-11  
**Work item:** BUG-0021 (bug)  
**Sprint:** `/quick` **Q0029**  
**Orchestrator run:** `auto-20260611-bug0021`  
**Next phase:** `/plan-verify` (role: qa)

### Summary

Sprint **Q0029** materializes **DEC-0110** + **DEC-0111** into seven mandatory P0
tasks + one optional P2 (8/12 under `SPRINT_MAX_TASKS`): static CategoryFilter
import on Forecast Monthly (**EA1**) and Wealth Overview (**EA2**); SQL
`COALESCE(attributes, root)` `account_role` path (**EB1**); frontend
`formatAccountRole` label map (**EB2**); optional PlanningPage parity (**EA3**);
integration tests (**T1**); automated gate (**G1**); operator verify-work (**V1**).
Closes acceptance **BK** (category filter snappy) and **BL** (wealth Role column).

**Decisions:** **DEC-0110**, **DEC-0111**  
**Research:** **R-0091**  
**Architecture ref:** `docs/engineering/architecture.md` § **BUG-0021**  
**Sprint artifacts:** `sprints/quick/Q0029/{sprint.md,sprint.json,tasks.md,task.json}`

### Execute order

```text
EA1 ∥ EA2 ∥ EB1
  → EB2 (WealthPage — coordinate with EA2)
  → EA3 (optional)
  → T1 → G1
  → operator: BACKEND_FRONTEND_DEPLOY
  → operator: SNAPSHOT_UPSERT_OR_SYNC (optional)
  → V1
```

### Task index

| ID | Title | Row | Files |
|----|-------|-----|-------|
| EA1 | ForecastPage static CategoryFilter import | **BK** | `frontend/src/pages/ForecastPage.tsx` |
| EA2 | WealthPage static CategoryFilter import | **BK** | `frontend/src/pages/WealthPage.tsx` |
| EB1 | repository COALESCE account_role SQL | **BL** | `backend/src/wealth/repository.rs` |
| EB2 | formatAccountRole label map | **BL** | `frontend/src/pages/WealthPage.tsx` (+ optional `accountRole.ts`) |
| EA3 | PlanningPage parity (optional P2) | — | `frontend/src/pages/PlanningPage.tsx` |
| T1 | Integration tests BK/BL + regression | **BK**, **BL** | `backend/tests/bug0021_wealth_account_role.rs` |
| G1 | Automated gate | **BK**, **BL** | `cargo test` + `npm run build` + progress.md |
| V1 | verify-work operator smoke | **BK**, **BL** | `sprints/quick/Q0029/uat.md` |

### BK/BL verification (frozen)

- **BK:** Forecast → Monthly and Wealth → Overview: no multi-second **Loading category filter…**; combobox ≤1 s
- **BL:** `GET /api/v1/wealth` asset accounts `account_role` non-null; Role column human labels; snapshot/Grafana post-upsert; OIDC smoke
- Regression: categories endpoint; wealth list shape; CategoryTrendChart lazy unchanged

### Operator gates (V1)

1. **BACKEND_FRONTEND_DEPLOY** — rebuild; fix ForecastPage TS6133 if still blocking (BUG-0020 follow-up)
2. **SNAPSHOT_UPSERT_OR_SYNC** (optional) — daily snapshot or manual sync for BL snapshot/Grafana gate

### Out of scope

CategoryFilter component logic; CategoryTrendChart lazy; Firefly sync rewrite; Grafana JSON edit; migration; root `active`/`include_net_worth` path fix.

---

# sprint-plan-20260611-q0028-bug0020 — BUG-0020 Subscriptions list quality

**From:** Tech Lead  
**To:** QA (`/plan-verify`) → Dev (`/execute`)  
**Date:** 2026-06-11  
**Work item:** BUG-0020 (bug)  
**Sprint:** `/quick` **Q0028**  
**Orchestrator run:** `auto-20260610-bug0019`  
**Next phase:** `/plan-verify` (role: qa)

### Summary

Sprint **Q0028** materializes **DEC-0109** into seven P0 tasks (7/12 under
`SPRINT_MAX_TASKS`): migration **016** reconcile + backfill (**DA1**, **DB1**),
All-tab filter (**DA2**), forward pending guard (**DA3**), integration tests
(**T1**), automated gate (**G1**), operator verify-work (**V1**). Closes acceptance
**BI** (duplicates) and **BJ** (display category).

**Decisions:** **DEC-0109** (extends DEC-0085, DEC-0086, DEC-0100)  
**Research:** **R-0090**  
**Architecture ref:** `docs/engineering/architecture.md` § **BUG-0020**  
**Sprint artifacts:** `sprints/quick/Q0028/{sprint.md,sprint.json,tasks.md,task.json}`

### Execute order

```text
DA1 → DB1 (migration 016)
  ∥ DA2 ∥ DA3
  → T1 → G1
  → operator: BACKEND_FRONTEND_DEPLOY + migration 016
  → operator: FULL_FIREFLY_SYNC
  → V1
```

### Task index

| ID | Title | Row | Files |
|----|-------|-----|-------|
| DA1 | Migration 016 YouTube merge + Strom collapse | **BI** | `backend/migrations/016_bug0020_subscription_list_quality.sql` |
| DB1 | Migration 016 confirmed `display_category_id` backfill | **BJ** | same migration |
| DA2 | All-tab exclude rejected/inactive | **BI** | `frontend/src/pages/SubscriptionsPage.tsx` |
| DA3 | Forward pending guard | **BI** | `backend/src/subscriptions/detection.rs` |
| T1 | Integration tests BI/BJ + regression | **BI**, **BJ** | `backend/tests/bug0020_subscription_list_quality.rs` |
| G1 | Automated gate | **BI**, **BJ** | `cargo test` + progress.md |
| V1 | verify-work operator smoke | **BI**, **BJ** | `sprints/quick/Q0028/uat.md` |

### BI/BJ verification (frozen)

- **BI:** `GET /api/v1/subscriptions?status=confirmed` ≤1 per `payee_key`; All tab no Strom/YouTube dupes; post-sync no new YouTube confirmed dup
- **BJ:** netflix/kindle→18; youtube→66; hgp→56; florian gabriel→3 (R-0090 oracle)
- Regression: discover, tags, detection, OIDC list smoke

### Operator gates (V1)

1. **BACKEND_FRONTEND_DEPLOY** — rebuild + migration 016 apply
2. **FULL_FIREFLY_SYNC** — detection regression after DA3

### Out of scope

API list dedup; list-time category recompute; pending/rejected backfill; Firefly writes.

---

# architecture-20260611-bug0020 — BUG-0020 Subscriptions list quality

**From:** Tech Lead  
**To:** Tech Lead (`/sprint-plan`) → Dev (`/execute`)  
**Date:** 2026-06-11  
**Work item:** BUG-0020 (bug)  
**Orchestrator run:** `auto-20260610-bug0019`  
**Next phase:** `/sprint-plan` (role: tech-lead)

### Summary

Architecture complete per **DEC-0109**. Two-layer fix: (1) **DA** — migration
`016_bug0020_subscription_list_quality.sql` reconciles YouTube confirmed dup +
Strom pending collapse; All tab excludes `rejected`/`inactive`; forward pending
guard in detection; (2) **DB** — same migration backfills `display_category_id`
for all confirmed patterns via DEC-0100 RANK SQL. Unfiltered API unchanged.

**Decisions:** **DEC-0109** (extends DEC-0085, DEC-0086, DEC-0100)  
**Research:** **R-0090**  
**Architecture ref:** `docs/engineering/architecture.md` § **BUG-0020**  
**Acceptance:** **BI** (DA), **BJ** (DB)

### Execute slices (for sprint-plan)

| ID | Scope | Primary files |
|----|-------|---------------|
| DA1 | Reconcile migration | `backend/migrations/016_bug0020_subscription_list_quality.sql` |
| DA2 | All-tab filter | `frontend/src/pages/SubscriptionsPage.tsx` |
| DA3 | Forward pending guard | `backend/src/subscriptions/detection.rs` |
| DB1 | Confirmed backfill (in DA1 migration) | same migration file |
| T1 | Integration tests | `backend/tests/bug0020_subscription_list_quality.rs` |
| V1 | Operator smoke | `sprints/quick/Q0028/uat.md` (sprint-plan to create) |

### BI/BJ verification (frozen)

- **BI:** `GET /api/v1/subscriptions?status=confirmed` ≤1 row per `payee_key`; All tab no Strom/YouTube dupes
- **BJ:** netflix/kindle → cat 18; youtube → 66; hgp → 56 (R-0090 oracle)
- Regression: discover, tags, detection, OIDC list smoke

### Out of scope

API list dedup collapse; list-time category recompute; pending/rejected backfill; Firefly writes.
