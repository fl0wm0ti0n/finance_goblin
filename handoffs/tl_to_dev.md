# TL -> Dev Handoff

## sprint-plan-20260607-q0023-bug0015 — BUG-0015 confirm persistence after rebuild

**From:** Tech Lead  
**To:** Dev (`/execute`) / QA (`/plan-verify`) / Operator (V1)  
**Date:** 2026-06-07  
**Work item:** BUG-0015 (defect)  
**Sprint:** **Q0023** (`/quick`)  
**Orchestrator run:** `auto-20260607-bug0015-001`  
**Next phase:** `/plan-verify`

### Summary

Quick sprint **Q0023** formalizes architecture **DEC-0084** (AU1 card `payee_key` normalization), **DEC-0085**/**DEC-0086** (AU2–AU4 payee+interval skip+merge, ±3d tolerance, in-place fingerprint rotation), and **V1** verify-work rebuild smoke — five tasks (all P0 mandatory); no split (5 < `SPRINT_MAX_TASKS` 12). AU1 → AU2 → (AU3 ∥ AU4); operator gates before V1.

**Decisions:** **DEC-0084**, **DEC-0085**, **DEC-0086** (+ **DEC-0071**, **DEC-0072**, **DEC-0013**, **DEC-0015**)  
**Research:** **R-0081**, **R-0082**  
**Architecture ref:** `docs/engineering/architecture.md` § **BUG-0015**  
**Sprint ref:** `sprints/quick/Q0023/sprint.md`, `sprints/quick/Q0023/task.json`

### Execute order (frozen)

| Order | Task | Primary files | Acceptance |
|-------|------|---------------|------------|
| 1 | **AU1** | `recurrence/normalize.rs` | **AU**, **AV** |
| 2 | **AU2** | `subscriptions/repository.rs`, migration | **AU**, **AV** |
| 3 | **AU3** | `subscriptions/detection.rs`, `service.rs` | **AU**, **AV**, **AW** |
| 4 | **AU4** | `detection.rs`, `service.rs` (stale map) | **AV** |
| 5 | **V1** | `uat.md` + operator rebuild smoke | **AU**–**AW** |

### Acceptance mapping

| Row | Tasks | Verify |
|-----|-------|--------|
| **AU** | AU1, AU2, AU3, V1 | Confirmed Cursor/Apple after rebuild + Full sync — not pending |
| **AV** | AU1–AU4, V1 | No duplicate pending; merge/skip on payee+interval |
| **AW** | AU3, V1 | Unread alerts reconcile — no spurious new_detection |

### Operator gates (before V1)

1. **BACKEND_FRONTEND_DEPLOY** — Q0023 backend on omniflow
2. **POSTGRES_PERSISTENCE_PROBE** — H2 SQL after rebuild, **before** Full sync
3. **FULL_FIREFLY_SYNC** — Full sync + detection phase

### Frozen boundaries

- **Single release** AU1–AU4 — no partial deploy
- **H2 probe** ops-only — not sprint code task
- **No** reopen BUG-0008; **no** alert-only dedup; **no** UI changes (H3 refuted)
- **No** global amount drop from fingerprint (option B rejected)
- Pre-fix orphan pending cleanup deferred

### Test contract

- AU1: `cargo test` recurrence normalize — Cursor/Apple descriptor fixtures
- AU2: merge upsert + `interval_matches` ±3d; index migration
- AU3/AU4: detection skip+merge; stale by payee+interval; no duplicate pending INSERT
- V1: `sprints/quick/Q0023/uat.md` rebuild smoke AU–AW on omniflow

### Artifacts created

- `sprints/quick/Q0023/{sprint.md,sprint.json,tasks.md,task.json,progress.md,uat.md,uat.json}`
- `handoffs/tl_to_dev.md` (this section)
- `docs/product/backlog.md#BUG-0015` (sprint_id Q0023)
- `docs/engineering/state.md` (traceability + governance)

### Prior handoff

`architecture-20260607-bug0015` in `handoffs/po_to_tl.md` — superseded for execute by this sprint-plan handoff.

---

## sprint-plan-20260610-q0022-bug0014 — BUG-0014 post-rebuild omniflow cluster

**From:** Tech Lead  
**To:** Dev (`/execute`) / QA (`/plan-verify`) / Operator (V1)  
**Date:** 2026-06-10  
**Work item:** BUG-0014 (defect)  
**Sprint:** **Q0022** (`/quick`)  
**Orchestrator run:** `auto-20260607-bug0014-001`  
**Next phase:** `/plan-verify`

### Summary

Quick sprint **Q0022** formalizes architecture **DEC-0081** (AQ1/AQ2 holdings/FX), **DEC-0082** (AS1 delete guard), **DEC-0083** (AS2 target_type UI), **AO1** Grafana dual-scenario ML banner, conditional **AP2** (DEC-0080 subtotal extension), conditional **AR1**, and **V1** verify-work — eight tasks (5 P0 mandatory + AS2 P1 optional + AP2/AR1 conditional); no split (8 < `SPRINT_MAX_TASKS` 12). AO1/AQ1/AS1 parallelizable; AQ2 after AQ1; operator gates before V1 and AP2.

**Decisions:** **DEC-0081**, **DEC-0082**, **DEC-0083** (+ **DEC-0080**, **DEC-0064**, **DEC-0066**, **DEC-0076**, **DEC-0077**)  
**Research:** **R-0079**  
**Architecture ref:** `docs/engineering/architecture.md` § **BUG-0014**  
**Sprint ref:** `sprints/quick/Q0022/sprint.md`, `sprints/quick/Q0022/task.json`

### Execute order (frozen)

| Order | Task | Primary files | Acceptance |
|-------|------|---------------|------------|
| 1 | **AO1** | `forecast-horizons.json` panel 13 | **AO** |
| 2 | **AQ1** | `wealth/service.rs`, `types.rs` | **AQ** |
| 3 | **AQ2** | `WealthPage.tsx`, `api.ts` | **AQ** |
| 4 | **AS1** | `plans.rs`, `PlanningPage.tsx` | **AS** |
| 5 | **AS2** | `PlanningPage.tsx` (optional P1) | **AS** |
| 6 | **AP2** | `wealth/service.rs` (conditional) | **AP** |
| 7 | **AR1** | `cashflow.json` (conditional) | **AR** |
| 8 | **V1** | `uat.md` + operator smoke | **AO**–**AT** |

### Acceptance mapping

| Row | Tasks | Verify |
|----|-------|--------|
| **AO** | AO1, V1 | ML available or sidecar-unreachable copy — not misleading US-0013 disabled |
| **AP** | AP2, V1 | crypto.subtotal_eur > 0 when wallet priced; AP2 after AP1 gate |
| **AQ** | AQ1, AQ2, V1 | holdings_all native+EUR; unified fx_incomplete |
| **AR** | AR1, V1 | Cashflow acct 114 non-zero; AR1 only API≠Grafana |
| **AS** | AS1, AS2, V1 | Plan delete + 409 active; target_type enum + help |
| **AT** | V1 | Three-service compose — stats-forecast when ML enabled |

### Operator gates (before V1 / AP2)

1. **BACKEND_FRONTEND_DEPLOY** — Q0020 image on omniflow
2. **THREE_SERVICE_COMPOSE** — stats-forecast per DEC-0076
3. **FULL_FIREFLY_SYNC** — Full sync + recompute acct **114**
4. **GRAFANA_PROVISIONING_RELOAD** — after AO1 (and AR1 if run)
5. **AP1_SQL_PROBE** — futures row priced before **AP2**

### Frozen boundaries

- **AP2** only after AP1 SQL gate — do not mask deploy gap
- **AR1** only when V1 proves API≠Grafana — default verify-only
- **AO/AT runtime** ops-only — not code tasks
- **No** ExchangePriceBook tier-2; **no** Grafana dynamic ML Postgres variable; **no** `target_type` enum expansion
- **DEC-0064** — linear notional excluded from `crypto.subtotal_eur`

### Test contract

- AQ1/AP2: `cargo test` wealth paths PASS
- AS1: DELETE active → 409; non-active → success
- V1: `sprints/quick/Q0022/uat.md` smoke AO–AT on omniflow

### Artifacts created

- `sprints/quick/Q0022/{sprint.md,sprint.json,tasks.md,task.json,progress.md,uat.md,uat.json}`
- `handoffs/tl_to_dev.md` (this section)
- `docs/product/backlog.md#BUG-0014` (sprint_id Q0022)
- `docs/engineering/state.md` (traceability + governance)

### Prior handoff

`architecture-20260609-bug0014` in `handoffs/po_to_tl.md` — superseded for execute by this sprint-plan handoff.

---

## sprint-plan-20260609-q0021-us0017 — US-0017 README living-doc expansion (doc-only)

**From:** Tech Lead  
**To:** Dev (`/execute`) / QA (`/plan-verify`)  
**Date:** 2026-06-09  
**Work item:** US-0017 (story)  
**Sprint:** **Q0021** (`/quick`)  
**Orchestrator run:** `auto-20260609-us0017-001`  
**Next phase:** `/plan-verify`

### Summary

Quick sprint **Q0021** formalizes **DEC-0070** US-0017 extension — seven doc-only tasks (E1–E6 architecture slices + UG1 user guide); no split (7 < `SPRINT_MAX_TASKS` 12). E1/E2/E4/E5 parallelizable on disjoint files; E6 validator gate fail-closed after all edits. **No application code.**

**Decision:** **DEC-0070** (US-0017 extension)  
**Research:** **R-0078** (extends R-0066, R-0067)  
**Architecture ref:** `docs/engineering/architecture.md` § **US-0017**  
**Sprint ref:** `sprints/quick/Q0021/sprint.md`, `sprints/quick/Q0021/task.json`

### Execute order (frozen)

| Order | Task | Primary files | Acceptance |
|-------|------|---------------|------------|
| 1 | **E1** | `README.md` § Examples H3 | **AC-1** |
| 2 | **E2** | `README.md` § Limitations H3 | **AC-2** |
| 3 | **E4** | `docs/developer/README.md` | **AC-4** |
| 4 | **E5** | `runbook.md` § README maintenance | **AC-4** |
| 5 | **E3** | `README.md` Product status verify | **AC-3** |
| 6 | **UG1** | `docs/user-guides/US-0017.md` | **AC-1**, **AC-2** |
| 7 | **E6** | validator run | **AC-5** |

### Acceptance mapping

| AC | Tasks | Verify |
|----|-------|--------|
| **AC-1** | E1, UG1 | Omniflow external-profile smoke H3 — not localhost-only |
| **AC-2** | E2, UG1 | Troubleshooting H3 — gates, symptom table, ML-off vs data-missing |
| **AC-3** | E3 | Product status verify — US-0015 + post-US-0016 closures |
| **AC-4** | E4, E5 | Per-segment maintenance hooks in developer README + runbook |
| **AC-5** | E6 | `validate_doc_profile --no-template-parity` exit 0 |

### Frozen boundaries

- **H3 only** under existing `## Examples` and `## Limitations` — no new root H2
- **E3 verify-only** — append Product status bullets only if segment closes additional US/BUG
- **Content sources:** R-0078 §2 curls; R-0078 §3 / Q0020 uat symptom table — **no credential literals**
- **Out of scope:** analytics code; dedicated `## Troubleshooting` H2; full runbook §23 inline; DEC-0081

### Test contract

- E6: `python3 scripts/validate_doc_profile.py --repo . --no-template-parity` → exit **0**
- Manual review: H3 content, per-segment hooks, user guide cross-links

### Artifacts created

- `sprints/quick/Q0021/{sprint.md,sprint.json,tasks.md,task.json,progress.md,uat.md,uat.json}`
- `handoffs/tl_to_dev.md` (this section)
- `docs/product/backlog.md#US-0017` (sprint_id Q0021)
- `docs/engineering/state.md` (traceability + governance)

### Prior handoff

`architecture-20260609-us0017` in `handoffs/po_to_tl.md` — superseded for execute by this sprint-plan handoff.

---

## sprint-plan-20260608-q0020-bug0013 — BUG-0013 omniflow analytics regression (AL/AN)

**From:** Tech Lead  
**To:** Dev (`/execute`) / QA (`/plan-verify`) / Operator (V1)  
**Date:** 2026-06-08  
**Work item:** BUG-0013 (defect)  
**Sprint:** **Q0020** (`/quick`)  
**Next phase:** `/plan-verify`

### Summary

Quick sprint **Q0020** formalizes architecture **DEC-0079** (AL1 budgets MTD upper date bound) + **DEC-0080** (AN1 Bitunix wallet parse + linear unrealized EUR) — five tasks (3 P0 mandatory + 2 P2 optional); no split (5 < `SPRINT_MAX_TASKS` 12). AL1 (Grafana) and AN1 (backend) parallelizable; operator gates before V1.

**Decisions:** **DEC-0079**, **DEC-0080**  
**Research:** **R-0076**, **R-0077**  
**Architecture ref:** `docs/engineering/architecture.md` § **BUG-0013**  
**Sprint ref:** `sprints/quick/Q0020/sprint.md`, `sprints/quick/Q0020/task.json`

### Execute order (frozen)

| Order | Task | Primary files | Acceptance |
|-------|------|---------------|------------|
| 1 | **AL1** | `grafana/.../budgets.json` | **AL** |
| 2 | **AN1** | `bitunix.rs`, `pnl.rs` + tests | **AN**, **AK** |
| 3 | **AJ1** | `subscriptions.json` (optional) | **AJ** |
| 4 | **AK2** | `portfolio.json` (optional) | **AK** |
| 5 | **V1** | verify-work omniflow smoke | **AI**–**AN** |

### Acceptance mapping

| Row | Tasks | Verify |
|-----|-------|--------|
| **AI** | V1 | Re-smoke baseline panels acct 114 — discovery refuted code regression |
| **AJ** | AJ1, V1 | Documented empty-state when 0 price-change events |
| **AK** | AN1, AK2, V1 | Crypto subtotal > 0; performance % footnote if <2 snapshots |
| **AL** | AL1, V1 | MTD capped — not 730-day sum (−€150K artifact) |
| **AM** | V1 | ds/query 200 — waived per R-0077 |
| **AN** | AN1, V1 | Wealth/portfolio crypto totals after sync |

### Frozen boundaries

- **AL1 only** for confirmed SQL bug — `pdc.ts::date <= CURRENT_DATE` on planned CTE
- **AN1** — wallet array parse + linear unrealized EUR; **no** linear notional in `market_value_eur` (DEC-0064)
- **AI/AJ/AM** — no code fixes; V1 re-smoke or waived (R-0077 for AM)
- **AJ1/AK2** — optional P2 UX; skip without blocking P0 closure
- **US-0013** ML overlay out of scope
- **AM1** deferred unless operator HAR shows non-200

### Test contract (AL/AN)

- AL1: broken MTD −150337.6 → capped value (0 when plan starts after today)
- AN1: array wallet mock → futures row; linear unrealized USDT→EUR
- AN1: `crypto.subtotal_eur` > 0 post sync when wallet equity > 0
- V1: budgets + wealth + portfolio + cashflow/forecast probes on omniflow

### Operator gates (V1)

| Gate | Before | Action |
|------|--------|--------|
| **BACKEND_FRONTEND_DEPLOY** | V1 | Deploy AL1 + AN1 release on omniflow |
| **GRAFANA_PROVISIONING_RELOAD** | V1 | Reload Grafana after AL1 |
| **FULL_FIREFLY_SYNC** | V1 | Full sync + forecast recompute (not exchanges-only) |

### Artifacts created

- `sprints/quick/Q0020/{sprint.md,sprint.json,tasks.md,task.json,progress.md,uat.md,uat.json}`
- `handoffs/tl_to_dev.md` (this section)
- `docs/product/backlog.md#BUG-0013` (quick_task_id Q0020)
- `docs/engineering/state.md` (traceability + governance)

### Prior handoff

`architecture-20260608-bug0013` in `handoffs/po_to_tl.md` — superseded for execute by this sprint-plan handoff.

---

## sprint-plan-20260606-s0016-us0015 — US-0015 AI forecast bucket mapping

**From:** Tech Lead  
**To:** Dev (`/execute`) / QA (`/plan-verify`) / Operator (BACKEND_FRONTEND_DEPLOY)  
**Date:** 2026-06-06  
**Story:** US-0015  
**Sprint:** **S0016**  
**Orchestrator:** `auto-20260606-us0015-001`  
**Next phase:** `/plan-verify`

### Summary

Standard sprint **S0016** formalizes architecture **DEC-0078** — AI-assisted forecast bucket mapping. Twelve tasks across slices US-0015-S1..S3: inference service + privacy allowlist (S1), projection merge with config precedence (S2), API provenance + UI badge + audit + user guide (S3). No split (12 = `SPRINT_MAX_TASKS` 12). S1+S2 before S3 API/UI.

**Decision:** **DEC-0078**  
**Research:** **R-0074**, **R-0075**  
**Architecture ref:** `docs/engineering/architecture.md` § **US-0015**  
**Sprint ref:** `sprints/S0016/sprint.md`, `sprints/S0016/tasks.md`, `sprints/S0016/sprint.json`

### Execute order (frozen)

| Order | Task | Primary files | Acceptance |
|-------|------|---------------|------------|
| 1 | **T-0163** | `forecast/bucket_inference.rs` | AC-2 |
| 2 | **T-0164** | `ai/privacy.rs` | AC-3 |
| 3 | **T-0165** | `bucket_inference.rs`, `config/default.toml` | AC-2 |
| 4 | **T-0166** | `bucket_inference.rs`, `privacy.rs` tests | AC-2, AC-3 |
| 5 | **T-0167** | `forecast/categories.rs`, `project.rs` | AC-1 |
| 6 | **T-0168** | `forecast/project.rs` | AC-1, AC-2 |
| 7 | **T-0169** | `forecast/project.rs`, `types.rs` | AC-4 |
| 8 | **T-0170** | `project.rs`, `forecast_integration` | AC-1 |
| 9 | **T-0171** | `api/forecast.rs` | AC-4 |
| 10 | **T-0172** | `ForecastPage.tsx` | AC-5 |
| 11 | **T-0173** | `ai_tool_audit` persistence | AC-6 |
| 12 | **T-0174** | `docs/user-guides/US-0015.md`, `sprints/S0016/uat.md` | AC-7 |

### Acceptance mapping

| Row | Tasks | Verify |
|-----|-------|--------|
| Prerequisite | — | BUG-0012 AG/AH DONE (pre-checked) |
| AC-1 | T-0167, T-0168, T-0170 | Config map never overridden by AI |
| AC-2 | T-0163, T-0165, T-0166, T-0168 | LLM proposal + threshold fallback |
| AC-3 | T-0164, T-0166 | Privacy allowlist under default TOML |
| AC-4 | T-0169, T-0171 | `bucket_sources` on monthly API |
| AC-5 | T-0172 | AI-mapped badge on Monthly tab |
| AC-6 | T-0173 | `forecast_bucket_assignment` audit rows |
| AC-7 | T-0174 | OIDC Monthly smoke UAT; **BACKEND_FRONTEND_DEPLOY** gate |

### Sequencing diagram

```text
T-0163 → T-0164 → T-0165 → T-0166
  ↓
T-0167 → T-0168 → T-0169 → T-0170
  ↓
T-0171 → T-0172 → T-0173 → T-0174
  ↓
Operator: BACKEND_FRONTEND_DEPLOY → UAT /forecast Monthly smoke
```

### Frozen contracts (DEC-0078)

| Topic | Contract |
|-------|----------|
| Cascade | config → rule → LLM → Variable |
| Threshold | `ai_bucket_min_confidence = 0.75` default |
| Privacy | R-0075 allowlist; same for local/cloud |
| Provider | US-0008 `build_provider()` reuse — no forecast_ai_* split |
| Rolling residual | Variable only in MVP — document in user guide |
| Chat boundary | No chat tool registry changes (DEC-0069) |
| Badge | `ai_mapped` authoritative over mixed `bucket_sources` |

### Operator gate

| Gate | Before | Action |
|------|--------|--------|
| **BACKEND_FRONTEND_DEPLOY** | AC-7 OIDC smoke | Deploy S1–S3 backend + frontend on omniflow |

### Next phase

**`/plan-verify`** — validate task coverage against acceptance; do not begin execute in sprint-plan subagent.

---

## sprint-plan-20260608-s0015-us0014 — US-0014 planning UX completion

**From:** Tech Lead  
**To:** Dev (`/execute`) / QA (`/plan-verify`) / Operator (BACKEND_FRONTEND_DEPLOY)  
**Date:** 2026-06-08  
**Story:** US-0014  
**Sprint:** **S0015**  
**Orchestrator:** `auto-20260608-us0014-001`  
**Next phase:** `/plan-verify`

### Summary

Standard sprint **S0015** formalizes architecture **DEC-0077** — page-local planning mutation feedback on `PlanningPage.tsx`. Eight tasks across slices US-0014-S1..S3: verify-first onboarding (S1), mutation helper + onError + PVA invalidation (S2 primary), Compare/PVA verify + user guide + OIDC UAT template (S3). No split (8 < `SPRINT_MAX_TASKS` 12). S2 helper before S1 toasts and S3 error-path smoke.

**Decision:** **DEC-0077**  
**Research:** **R-0072**, **R-0073**  
**Architecture ref:** `docs/engineering/architecture.md` § **US-0014**  
**Sprint ref:** `sprints/S0015/sprint.md`, `sprints/S0015/tasks.md`, `sprints/S0015/sprint.json`

### Execute order (frozen)

| Order | Task | Primary files | Acceptance |
|-------|------|---------------|------------|
| 1 | **T-0158** | `frontend/src/pages/planningFeedback.ts`, `PlanningPage.tsx` | AC-7 |
| 2 | **T-0159** | `PlanningPage.tsx` (7 mutations) | AC-7 |
| 3 | **T-0160** | `PlanningPage.tsx` (invalidation) | AC-2 |
| 4 | **T-0155** | `PlanningPage.tsx` (verify-only) | AC-1 |
| 5 | **T-0156** | `PlanningPage.tsx` (banner) | AC-6 |
| 6 | **T-0157** | `PlanningPage.tsx` (success toasts) | AC-5 |
| 7 | **T-0161** | `PlanningPage.tsx` (verify-only) | AC-3, AC-4 |
| 8 | **T-0162** | `docs/user-guides/US-0014.md`, `sprints/S0015/uat.md` | AC-8 |

### Acceptance mapping

| Row | Tasks | Verify |
|-----|-------|--------|
| Prerequisite | — | BUG-0011 AD/AE/AF DONE (pre-checked) |
| AC-1 | T-0155 | Empty-state template grid + Create empty plan regression |
| AC-2 | T-0160, T-0159 | Add-line success + immediate PVA invalidation |
| AC-3 | T-0161 | Compare 0.00 delta + overlay footnote |
| AC-4 | T-0161 | PVA guided card when no active plan |
| AC-5 | T-0157 | Create/template/apply success confirmations |
| AC-6 | T-0156 | Set-active banner + Grafana Dashboard 3 copy |
| AC-7 | T-0158, T-0159 | Red error card on all mutation failures |
| AC-8 | T-0162 | OIDC three-tab smoke UAT; **BACKEND_FRONTEND_DEPLOY** gate |

### Sequencing diagram

```text
T-0158 → T-0159 → T-0160
  ↓
(T-0155 + T-0156) ∥ T-0157
  ↓
T-0161 → T-0162
  ↓
Operator: BACKEND_FRONTEND_DEPLOY → UAT /planning smoke
```

### Frozen contracts (DEC-0077)

| Topic | Contract |
|-------|----------|
| Helper scope | Page-local only — no global MutationCache |
| Error coverage | All 7 mutations require `onError` |
| Success toasts | createPlan, applyTemplate (all), addAdjustment, activate |
| Invalidation | `plan-vs-actual` on adjustment CRUD + activate + createPlan |
| Banner | Plan vs Actual + Grafana Dashboard 3 (`budgets`) |
| Compare/PVA API | Frozen DEC-0073 / DEC-0074 |

### Operator gate

| Gate | Before | Action |
|------|--------|--------|
| **BACKEND_FRONTEND_DEPLOY** | AC-8 OIDC smoke | Deploy S1–S2 frontend on omniflow |

### Next phase

**`/plan-verify`** — validate task coverage against acceptance; do not begin execute in sprint-plan subagent.

---

## architecture-20260608-us0014 — US-0014 planning UX completion

**From:** Tech Lead  
**To:** Sprint planner (`/sprint-plan`)  
**Date:** 2026-06-08  
**Story:** US-0014  
**Orchestrator:** `auto-20260608-us0014-001`  
**Next phase:** `/sprint-plan` → **S0015**

### Summary

Architecture freezes **DEC-0077** — page-local `planningFeedback` helper with success/error card variants; mandatory `onError` on all seven planning mutations; success confirmations on create/template/add/activate; immediate `plan-vs-actual` invalidation on adjustment CRUD; set-active banner extended for Grafana Dashboard 3. Q0019 shipped AC-1/AC-3/AC-4 — execute is **polish + error surfaces**, not greenfield. DEC-0073/0074 frozen — frontend-only.

**Decision:** **DEC-0077**  
**Research:** **R-0072**, **R-0073**  
**Architecture ref:** `docs/engineering/architecture.md` § **US-0014**  
**Spec-pack:** `docs/engineering/spec-pack/US-0014-{design-concept,crs,technical-specification}.md`  
**User guide:** `docs/user-guides/US-0014.md`  
**Acceptance:** `docs/product/acceptance.md` § US-0014 (9 rows)

### Recommended sprint

| Field | Value |
|-------|-------|
| **Type** | Standard sprint **S0015** |
| **Slices** | US-0014-S1..S3 |
| **Tasks (est.)** | ~8 |
| **Split** | No — under `SPRINT_MAX_TASKS=12` |

### Execute slices (frozen — sprint-plan materializes)

| Slice | Primary files | Acceptance |
|-------|---------------|------------|
| **S1** | `PlanningPage.tsx` banner + create toasts | AC-1 verify, AC-5, AC-6 |
| **S2** | `planningFeedback.ts`, `PlanningPage.tsx` mutations | AC-2, AC-7 |
| **S3** | `docs/user-guides/US-0014.md`, verify Compare/PVA | AC-3, AC-4, AC-8 |

**Sequencing:** S2 helper before onError wiring; S3 smoke after S2; S1 verify may parallel S2.

### Frozen contracts (DEC-0077)

| Topic | Contract |
|-------|----------|
| Helper scope | Page-local only — no global MutationCache |
| Error coverage | All 7 mutations require `onError` |
| Success toasts | createPlan, applyTemplate (all), addAdjustment, activate |
| Invalidation | `plan-vs-actual` on adjustment CRUD + activate + createPlan |
| Banner | Plan vs Actual + Grafana Dashboard 3 (`budgets`) |
| Compare/PVA API | Frozen DEC-0073 / DEC-0074 |

### Operator gate (V1 — sprint-plan adds)

| Gate | Before | Action |
|------|--------|--------|
| **BACKEND_FRONTEND_DEPLOY** | AC-8 OIDC smoke | Deploy S1–S2 frontend on omniflow |

### Artifacts created

- `docs/engineering/architecture.md` § US-0014
- `decisions/DEC-0077.md`, `docs/engineering/decisions.md`
- `docs/engineering/research.md` R-0073
- `docs/engineering/spec-pack/US-0014-*.md`
- `docs/user-guides/US-0014.md`
- `handoffs/tl_to_dev.md` (this section)
- `docs/engineering/state.md` (traceability + governance)

### Next phase

**`/sprint-plan` S0015** — materialize task.json from slice table; do not begin execute in architecture subagent.

---

## sprint-plan-20260608-s0014-us0013 — US-0013 production ML hardening

**From:** Tech Lead  
**To:** Dev (`/execute`) / QA (`/plan-verify`) / Operator (BACKEND_COMPOSE_DEPLOY)  
**Date:** 2026-06-08  
**Story:** US-0013  
**Sprint:** **S0014**  
**Orchestrator:** `auto-20260608-us0013-001`  
**Next phase:** `/plan-verify`

### Summary

Standard sprint **S0014** formalizes architecture **DEC-0076** — external-profile ML enablement on omniflow. Eleven tasks across slices US-0013-S1..S4: compose overlay + env + CI assert (S1), verify-first sync/API paths (S2), React/Grafana ML parity (S3), runbook + dual CI guard (S4). No split (11 < `SPRINT_MAX_TASKS` 12). S1-before-S2 sequencing frozen.

**Decision:** **DEC-0076**  
**Research:** **R-0071**; addenda R-0043, R-0044, R-0045, R-0053, R-0062  
**Architecture ref:** `docs/engineering/architecture.md` § **US-0013**  
**Sprint ref:** `sprints/S0014/sprint.md`, `sprints/S0014/tasks.md`, `sprints/S0014/sprint.json`

### Execute order (frozen)

| Order | Task | Primary files | Acceptance |
|-------|------|---------------|------------|
| 1 | **T-0144** | `docker-compose.external.yml` | AC-1 |
| 2 | **T-0145** | `docker-compose.external.yml` | AC-1 |
| 3 | **T-0146** | `.env.example` | AC-1 |
| 4 | **T-0147** | `scripts/compose-config-check.sh` | AC-1 |
| 5 | **T-0148** | `backend/src/sync/mod.rs`, `backend/src/forecast_ml/` | AC-2, AC-3 |
| 6 | **T-0149** | `backend/src/api/forecast.rs`, `backend/src/forecast_ml/service.rs` | AC-3, AC-4 |
| 7 | **T-0150** | `frontend/src/pages/ForecastPage.tsx` | AC-5 |
| 8 | **T-0151** | `frontend/src/pages/WealthPage.tsx` | AC-6 |
| 9 | **T-0152** | `grafana/.../forecast-horizons.json` | AC-7 |
| 10 | **T-0153** | `docs/engineering/runbook.md` | AC-8 |
| 11 | **T-0154** | `backend/tests/forecast_ml_integration.rs`, `tests/run-tests.sh` | AC-9 |

### Acceptance mapping

| Row | Tasks | Verify |
|-----|-------|--------|
| AC-1 | T-0144 … T-0147 | External overlay starts sidecar; env documented; CI 3-service assert |
| AC-2 | T-0148 | Sidecar health gate before ML phase |
| AC-3 | T-0148, T-0149 | Sync ML phase + skip metadata; UI phase label |
| AC-4 | T-0149 | `ml_enhanced` persisted; API `variant=ml_enhanced` returns series |
| AC-5 | T-0150 | Forecast Compare overlay; `sidecar_disabled` copy (DEC-0066) |
| AC-6 | T-0151 | Wealth ML portfolio overlay; signed totals (DEC-0065); FX banner (R-0034) |
| AC-7 | T-0152 | Grafana ML panels with `$forecast_variant=ml_enhanced` |
| AC-8 | T-0153 | Runbook § Omniflow ML enablement |
| AC-9 | T-0154 | Wiremock integration + compose assert in CI |
| Prerequisite | — | BUG-0010 AA/AB/AC DONE (Q0013) |

### Frozen boundaries

- **Profile union** — overlay additive `external` on single `stats-forecast`; no duplicate service block
- **Network** — traefik-only co-attachment; `STATS_FORECAST_URL=http://stats-forecast:8090`
- **Opt-in** — `FORECAST_ML_ENABLED=true` required; DEC-0049 default-off elsewhere
- **Verify-first S2/S3** — no algorithm changes unless gap found during audit
- **Dual CI guard** — compose-config-check + `forecast_ml_integration` both required
- **DEC-0056 guard unchanged** — `minimal+external` still excludes `firefly-iii`

### Operator gate

| Gate | Before | Action |
|------|--------|--------|
| **BACKEND_COMPOSE_DEPLOY** | UAT omniflow ML smoke | Deploy S1 overlay + env; restart `flow-finance-ai` + `stats-forecast`; set `FORECAST_ML_ENABLED=true`; Full Firefly sync after `/health` OK |

### Deploy order

```text
(T-0144 → T-0145 → T-0146 → T-0147) compose PR — atomic overlay + compose-check
(T-0148 → T-0149) verify sync/API — may include minimal fixes if gap found
(T-0150 + T-0151 + T-0152) verify UI/Grafana — parallel after S2
(T-0153 + T-0154) runbook + CI — T-0154 after T-0147
Deploy → Full sync → UAT omniflow smoke
```

### Artifacts created

- `sprints/S0014/sprint.md`, `sprints/S0014/sprint.json`, `sprints/S0014/tasks.md`, `sprints/S0014/progress.md`
- `sprints/S0014/uat.md`, `sprints/S0014/uat.json` (placeholders)
- `docs/engineering/state.md` — sprint-plan checkpoint + traceability
- `docs/product/backlog.md#US-0013` — sprint plan section
- `handoffs/tl_to_dev.md` (this section)

### Prior handoffs

- `architecture-20260608-us0013` — superseded by this sprint-plan handoff for execute.

---

## architecture-20260608-us0013 — US-0013 production ML hardening

**From:** Tech Lead  
**To:** Sprint planner (`/sprint-plan`)  
**Date:** 2026-06-08  
**Story:** US-0013  
**Orchestrator:** `auto-20260608-us0013-001`  
**Next phase:** `/sprint-plan` → **S0014**

### Summary

Architecture freezes **DEC-0076** — external overlay adds `stats-forecast` via additive `profiles: [external]` on the existing service (traefik network, host port 8091); passthrough `FORECAST_ML_ENABLED` / `STATS_FORECAST_URL` on `flow-finance-ai`; DEC-0049 default-off preserved; failure semantics unchanged (DEC-0052/0066); dual CI guard (compose-config-check + `forecast_ml_integration`). US-0009 backend/frontend/Grafana paths are **verify-first** — gap is compose wiring + operator opt-in + runbook.

**Decision:** **DEC-0076**  
**Research:** **R-0071**; addenda R-0043, R-0044, R-0045, R-0053, R-0062  
**Architecture ref:** `docs/engineering/architecture.md` § **US-0013**  
**Spec-pack:** `docs/engineering/spec-pack/US-0013-{design-concept,crs,technical-specification}.md`  
**User guide:** `docs/user-guides/US-0013.md`  
**Acceptance:** `docs/product/acceptance.md` § US-0013 (10 rows)

### Recommended sprint

| Field | Value |
|-------|-------|
| **Type** | Standard sprint **S0014** |
| **Slices** | US-0013-S1..S4 |
| **Tasks (est.)** | ~11 |
| **Split** | No — under `SPRINT_MAX_TASKS=12` |

### Execute slices (frozen — sprint-plan materializes)

| Slice | Primary files | Acceptance |
|-------|---------------|------------|
| **S1** | `docker-compose.external.yml`, `.env.example`, `scripts/compose-config-check.sh` | AC-1 |
| **S2** | `backend/src/sync/mod.rs`, `backend/src/forecast_ml/`, `backend/src/api/forecast.rs` | AC-2, AC-3, AC-4 |
| **S3** | `ForecastPage.tsx`, `WealthPage.tsx`, `forecast-horizons.json` | AC-5, AC-6, AC-7 |
| **S4** | `docs/engineering/runbook.md`, `backend/tests/forecast_ml_integration.rs` | AC-8, AC-9 |

**Sequencing:** S1 → S2 → S3; S4 after S1 (runbook/CI may parallel S3).

### Frozen contracts (DEC-0076)

| Topic | Contract |
|-------|----------|
| Profile union | Overlay additive `external` on single `stats-forecast` — no duplicate service block |
| Network | Traefik-only co-attachment; `STATS_FORECAST_URL=http://stats-forecast:8090` |
| Opt-in | `FORECAST_ML_ENABLED=true` required; default-off elsewhere |
| Health SLO | Runtime `health_ok()` gate; compose healthcheck advisory |
| Min history | `min_monthly_points=12` unchanged |
| CI | 3-service external set + traefik assert + wiremock integration |

### Operator gate (V1 — sprint-plan adds)

| Gate | Before | Action |
|------|--------|--------|
| **BACKEND_COMPOSE_DEPLOY** | V1 omniflow smoke | Deploy S1 overlay + env; restart `flow-finance-ai` + `stats-forecast` |

### Artifacts created

- `docs/engineering/architecture.md` § US-0013
- `decisions/DEC-0076.md`, `docs/engineering/decisions.md`
- `docs/engineering/spec-pack/US-0013-*.md`
- `docs/user-guides/US-0013.md`
- `handoffs/tl_to_dev.md` (this section)
- `docs/engineering/state.md` (traceability + governance)

### Next phase

**`/sprint-plan` S0014** — materialize task.json from slice table; do not begin execute in architecture subagent.

---

## sprint-plan-20260608-q0019-bug0011 — BUG-0011 planning mode broken (AD/AE/AF)

**From:** Tech Lead  
**To:** Dev (`/execute`) / QA (`/plan-verify`) / Operator (V1)  
**Date:** 2026-06-08  
**Work item:** BUG-0011 (defect)  
**Sprint:** **Q0019** (`/quick`)  
**Orchestrator:** `auto-20260608-bug0011-001`  
**Next phase:** `/plan-verify`

### Summary

Quick sprint **Q0019** formalizes architecture **DEC-0073** (AE overlay-only compare) + **DEC-0074** (AF PVA 200 `no_active_plan`) + **AD** first-run/add-line UX — eleven tasks in AE-before-AF order; no split (11 < `SPRINT_MAX_TASKS` 12). Single backend PR for AE1–AF1+T1 acceptable if AE lands before AF in commit order; frontend AF2/AD1–AD4; operator gate before V1.

**Decisions:** **DEC-0073**, **DEC-0074**  
**Research:** **R-0070**; addenda R-0015, R-0016, R-0017, R-0020  
**Architecture ref:** `docs/engineering/architecture.md` § **BUG-0011**  
**Sprint ref:** `sprints/quick/Q0019/sprint.md`, `sprints/quick/Q0019/task.json`

### Execute order (frozen)

| Order | Task | Primary files | Acceptance |
|-------|------|---------------|------------|
| 1 | **AE1** | `plan/overlay.rs`, `plan/project.rs` | **AE** |
| 2 | **AE2** | `plan/repository.rs`, `plan/service.rs` | **AE** |
| 3 | **AE3** | backend compare tests | **AE** |
| 4 | **AF1** | `api/plans.rs`, `plan/types.rs` | **AF** |
| 5 | **AF2** | `PlanningPage.tsx` PVA tab | **AF** |
| 6 | **AD1** | `PlanningPage.tsx` empty state | **AD** |
| 7 | **AD2** | `PlanningPage.tsx` add form | **AD** |
| 8 | **AD3** | `PlanningPage.tsx` Custom toast | **AD** |
| 9 | **AD4** | compare footnote + Set active banner | **AD**, **AE** |
| 10 | **T1** | integration tests | **AD/AE/AF** |
| 11 | **V1** | verify-work OIDC `/planning` | footer |

### Acceptance mapping

| Row | Tasks | Verify |
|-----|-------|--------|
| **AD** | AD1–AD3, T1, V1 | Create empty plan + add-line UX; not silent no-op |
| **AE** | AE1–AE3, AD4, T1, V1 | Zero/neutral compare deltas on empty plan |
| **AF** | AF1–AF2, T1, V1 | PVA 200 JSON; guided tab when no active plan |
| Footer | T1, V1 | OIDC `/planning` three-tab regression |

### Frozen boundaries

- **AE-before-AF** — AE1 helper before AF1 API freeze; non-negotiable
- **Overlay-only delta** — DEC-0073 contract frozen; projected balance unchanged
- **PVA 200 no_active_plan** — DEC-0074 contract frozen; no auto-activate
- **Grafana Dashboard 3** — unchanged (R-0020)
- **US-0014** — holistic UX epic deferred
- **Release note** — compare numbers shift for non-empty plans
- **PVA breaking change** — 404→200 documented in changelog + user guide

### Test contract (AD/AE/AF)

- AE3: zero adjustments → `monthly_delta_sum` = 0.00; Leasing ~overlay
- AF1: `no_active_plan` serializes 200 tagged JSON
- T1: compare endpoint + PVA route with/without active plan
- AD2: POST adjustment creates row; table editable
- Grafana: Dashboard 3 panels unchanged (no SQL edit)
- Operator smoke: `/planning` Scenarios + Compare + Plan vs Actual on OIDC deploy

### Operator gates (V1)

| Gate | Before | Action |
|------|--------|--------|
| **BACKEND_FRONTEND_DEPLOY** | V1 | Deploy AE1–T1 backend + AF2/AD1–AD4 frontend on omniflow |

### Artifacts created

- `sprints/quick/Q0019/{sprint.md,sprint.json,tasks.md,task.json,progress.md,uat.md,uat.json}`
- `handoffs/tl_to_dev.md` (this section)
- `docs/product/backlog.md#BUG-0011` (quick_task_id Q0019)
- `docs/engineering/state.md` (traceability + governance)

### Prior handoff

`architecture-20260608-bug0011` below — superseded for execute by this sprint-plan handoff.

---

## architecture-20260608-bug0011 — BUG-0011 planning mode broken (AD/AE/AF)

**From:** Tech Lead  
**To:** Sprint planner (`/sprint-plan`)  
**Date:** 2026-06-08  
**Bug:** BUG-0011  
**Orchestrator:** `auto-20260608-bug0011-001`  
**Next phase:** `/sprint-plan` → **`/quick` Q0019**

### Summary

Architecture freezes **DEC-0073** (AE overlay-only `monthly_delta_sum` via `build_overlay_deltas`; projected balance unchanged; zero-overlay → 0.00) and **DEC-0074** (AF HTTP 200 tagged `no_active_plan` for plan-vs-actual; guided PVA tab; no auto-activate). **AD** bundle (empty create, inline add form, Custom toast) is execute scope without a third DEC. **AE-before-AF** sequencing mandatory — AE1 helper before AF1 API freeze.

**Decisions:** **DEC-0073**, **DEC-0074**  
**Research:** **R-0070**; addenda R-0015, R-0016, R-0017, R-0020  
**Architecture ref:** `docs/engineering/architecture.md` § **BUG-0011**  
**Spec-pack:** `docs/engineering/spec-pack/BUG-0011-{design-concept,crs,technical-specification}.md`  
**User guide:** `docs/user-guides/BUG-0011.md`  
**Acceptance:** `docs/product/acceptance.md` rows **AD**, **AE**, **AF**

**ID coordination:** US-0090 caveman forward-refs renumbered **DEC-0073 → DEC-0075** in runbook + scripts.

### Recommended sprint

| Field | Value |
|-------|-------|
| **Type** | `/quick` **Q0019** |
| **Tasks** | 11 (AE1–AE3, AF1–AF2, AD1–AD4, T1, V1) |
| **Split** | No — under `SPRINT_MAX_TASKS=12` |
| **Order** | AE1 → AE2 → AE3 → AF1 → AF2 → AD1 → AD2 → AD3 → AD4 → T1 → V1 |

### Execute slices (frozen — sprint-plan materializes)

| Order | Task | Primary files | Acceptance |
|-------|------|---------------|------------|
| 1 | **AE1** | `plan/overlay.rs` or `project.rs` | **AE** |
| 2 | **AE2** | `plan/repository.rs`, `plan/service.rs` | **AE** |
| 3 | **AE3** | backend compare tests | **AE** |
| 4 | **AF1** | `api/plans.rs`, `plan/types.rs` | **AF** |
| 5 | **AF2** | `PlanningPage.tsx` PVA tab | **AF** |
| 6 | **AD1** | `PlanningPage.tsx` empty state | **AD** |
| 7 | **AD2** | `PlanningPage.tsx` add form | **AD** |
| 8 | **AD3** | `PlanningPage.tsx` Custom toast | **AD** |
| 9 | **AD4** | compare footnote + Set active banner | **AD**, **AE** |
| 10 | **T1** | integration tests | **AD/AE/AF** |
| 11 | **V1** | verify-work OIDC `/planning` | footer |

### Frozen contracts

| Topic | Contract |
|-------|----------|
| Compare delta | Overlay-only sum; empty adjustments → 0.00; projected balance unchanged |
| PVA empty | 200 `{ status: "no_active_plan", reason: "no_active_plan" }` — not 404 |
| Set active | Explicit operator action — no auto-activate on create |
| Grafana | Dashboard 3 (`budgets`) unchanged — R-0020 |
| AD form | Inline above table; POST/PATCH existing routes |
| Release note | Compare numbers shift for non-empty plans |

### Deploy order

```text
(AE1 → AE2) overlay helper + compare paths
→ AE3 tests
→ AF1 tagged PVA API
→ (AF2 + AD1–AD4) frontend
→ T1 integration
→ deploy backend + frontend
→ V1 operator /planning smoke
```

### Out of scope

- Grafana `budgets` panel SQL
- Auto-activate first plan
- US-0014 holistic UX epic
- AI simulate_plan changes

### Artifacts created

- `docs/engineering/architecture.md` § BUG-0011
- `decisions/DEC-0073.md`, `decisions/DEC-0074.md`
- `docs/engineering/decisions.md` (DEC-0073, DEC-0074)
- `docs/engineering/spec-pack/BUG-0011-*.md`
- `docs/user-guides/BUG-0011.md`
- US-0090 forward-ref renumber DEC-0075 (runbook, scripts)
- `handoffs/tl_to_dev.md` (this section)

### Prior handoff

`research-20260608-bug0011` in `handoffs/po_to_tl.md` — superseded by this architecture handoff for sprint-plan.

---

## sprint-plan-20260608-q0018-bug0008 — BUG-0008 subscription alerts & detection recall

**From:** Tech Lead  
**To:** Dev (`/execute`) / QA (`/plan-verify`) / Operator (V1)  
**Date:** 2026-06-08  
**Work item:** BUG-0008 (defect)  
**Sprint:** **Q0018** (`/quick`)  
**Orchestrator:** `auto-20260608-bug0008-001`  
**Next phase:** `/plan-verify`

### Summary

Quick sprint **Q0018** formalizes architecture **DEC-0071** (W bundle) + **DEC-0072** (X Phase 1) — twelve tasks in W-before-X order; no split (12 = `SPRINT_MAX_TASKS`). Single backend PR acceptable if W slices land before X in commit order; W6 frontend; operator gate before V1.

**Decisions:** **DEC-0071**, **DEC-0072**  
**Research:** **R-0068**, **R-0069**; addenda R-0009–R-0013  
**Architecture ref:** `docs/engineering/architecture.md` § **BUG-0008**  
**Sprint ref:** `sprints/quick/Q0018/sprint.md`, `sprints/quick/Q0018/task.json`

### Execute order (frozen)

| Order | Task | Primary files | Acceptance |
|-------|------|---------------|------------|
| 1 | **W1** | migration, backfill | **W** |
| 2 | **W2** | `subscriptions/repository.rs` | **W** |
| 3 | **W3** | `subscriptions/detection.rs` | **W** |
| 4 | **W4** | `subscriptions/routes.rs`, service | **W** |
| 5 | **W5** | confirm/reject/inactive handlers | **W** |
| 6 | **W6** | `SubscriptionsPage.tsx` | **W** |
| 7 | **W7** | backend tests | **W** regression |
| 8 | **X1** | `recurrence/normalize.rs` | **X** |
| 9 | **X2** | `recurrence/group.rs`, detection | **X** |
| 10 | **X3** | `config/default.toml` | **X** |
| 11 | **X4** | integration tests | **X** regression |
| 12 | **V1** | verify-work omniflow | **W**, **X** |

### Acceptance mapping

| Row | Tasks | Verify |
|-----|-------|--------|
| **W** | W1–W7, V1 | Reconciled unread-count vs pending; banner uses API not list length |
| **X** | X1–X4, V1 | Patterns > 12 baseline; no alert spam invariant |
| Footer | W7, X4, V1 | OIDC + bundled-firefly deploy regression |

### Frozen boundaries

- **W-before-X** — W1–W3 before X1; non-negotiable
- **Fingerprint + unread-count** — DEC-0071 contract frozen
- **Phase 2 X5** — category grouping deferred if over capacity
- **Header bell** — US-0005-only; no combined subscription badge
- **R-0065 coordinate** — additive unread-count route only; no `list_patterns` filter changes
- **min_emit 60** — hardcoded until W closed + FP review
- **AI detection** — deferred; document in release notes
- **BUG-0007** — AI tool changes out of scope

### Test contract (W/X)

- W1: migration backfill dedupes; partial unique index
- W2: ON CONFLICT upsert; no duplicate unread fingerprints
- W3: no alert on unchanged pending resync
- W4: unread-count `reconciled: true` when aligned
- W5: confirm/reject mark-read orphans
- W6: banner from unread-count API
- X1–X2: SEPA fixture merges under single payee key
- X3: 730-day window from TOML
- X4: forecast recurring unaffected or improved
- Operator smoke: reconciled banner; patterns > 12; deploy regression

### Operator gates (V1)

| Gate | Before | Action |
|------|--------|--------|
| **BACKEND_FRONTEND_DEPLOY** | V1 | Deploy W1–X4+W7 backend + W6 frontend on omniflow |

### Artifacts created

- `sprints/quick/Q0018/{sprint.md,sprint.json,tasks.md,task.json,progress.md,uat.md,uat.json}`
- `handoffs/tl_to_dev.md` (this section)
- `docs/product/backlog.md#BUG-0008` (quick_task_id Q0018)
- `docs/engineering/state.md` (traceability + governance)

### Prior handoff

`architecture-20260608-bug0008` below — superseded for execute by this sprint-plan handoff.

---

## architecture-20260608-bug0008 — BUG-0008 subscription alerts & detection recall

**From:** Tech Lead  
**To:** Sprint planner (`/sprint-plan`)  
**Date:** 2026-06-08  
**Bug:** BUG-0008  
**Orchestrator:** `auto-20260608-bug0008-001`  
**Next phase:** `/sprint-plan` → **`/quick` Q0018**

### Summary

Architecture freezes **DEC-0071** (W bundle: fingerprint dedup + unread-count API + orphan lifecycle + US-0005-only bell) and **DEC-0072** (X Phase 1: payee normalization + transfer counterparty priority + 730-day window; Phase 2 category grouping gated; AI deferred). **W-before-X sequencing mandatory** — W1–W3 before X1.

**Decisions:** **DEC-0071**, **DEC-0072**  
**Research:** **R-0068**, **R-0069**; addenda R-0009–R-0013  
**Architecture ref:** `docs/engineering/architecture.md` § **BUG-0008**  
**Spec-pack:** `docs/engineering/spec-pack/BUG-0008-{design-concept,crs,technical-specification}.md`  
**User guide:** `docs/user-guides/BUG-0008.md`  
**Acceptance:** `docs/product/acceptance.md` rows **W**, **X**

### Recommended sprint

| Field | Value |
|-------|-------|
| **Type** | `/quick` **Q0018** |
| **Tasks** | 12 (W1–W7, X1–X4, V1) |
| **Split** | No — at `SPRINT_MAX_TASKS=12`; Phase 2 X5 deferred if over capacity |
| **Order** | W1 → W2 → W3 → W4 → W5 → W6 → W7 → X1 → X2 → X3 → X4 → V1 |

### Execute slices (frozen — sprint-plan materializes)

| Order | Task | Primary files | Acceptance |
|-------|------|---------------|------------|
| 1 | **W1** | migration, backfill | **W** |
| 2 | **W2** | `subscriptions/repository.rs` | **W** |
| 3 | **W3** | `subscriptions/detection.rs` | **W** |
| 4 | **W4** | `subscriptions/routes.rs`, service | **W** |
| 5 | **W5** | confirm/reject/inactive handlers | **W** |
| 6 | **W6** | `SubscriptionsPage.tsx` | **W** |
| 7 | **W7** | backend tests | **W** regression |
| 8 | **X1** | `recurrence/normalize.rs` | **X** |
| 9 | **X2** | `recurrence/group.rs`, detection | **X** |
| 10 | **X3** | `config/default.toml` | **X** |
| 11 | **X4** | integration tests | **X** regression |
| 12 | **V1** | verify-work omniflow | **W**, **X** |

### Frozen contracts

| Topic | Contract |
|-------|----------|
| Dedup | Fingerprint partial unique; upsert_alert; emit on new pending or tier increase only |
| Unread-count | `GET /api/v1/subscriptions/alerts/unread-count`; banner/toast consume API — not list length |
| Bell | US-0005-only header badge unchanged |
| Lifecycle | Mark-read orphans on confirm/reject/inactive |
| Recall Phase 1 | SEPA normalization + transfer counterparty priority + 730-day window |
| Recall Phase 2 | Category grouping ≥70% — gated; optional if sprint capacity |
| AI | Deferred — document in release notes |
| Coordinate | No `list_patterns` filter changes (R-0065) |
| Sequencing | W dedup before X recall — non-negotiable |

### Deploy order

```text
(W1 → W2 → W3) migration + repository + detection gate
→ (W4 → W5) API + lifecycle
→ W6 frontend
→ (X1 → X2 → X3) recurrence core
→ X4 tests
→ deploy backend + frontend
→ V1 operator verify
```

### Out of scope

- Combined header badge
- AI in-pipeline detection
- min_emit 60 → 55 until W closed
- BUG-0007 AI tool changes

### Artifacts created

- `docs/engineering/architecture.md` § BUG-0008
- `decisions/DEC-0071.md`, `decisions/DEC-0072.md`
- `docs/engineering/decisions.md` (DEC-0071, DEC-0072)
- `docs/engineering/spec-pack/BUG-0008-*.md`
- `docs/user-guides/BUG-0008.md`
- `handoffs/tl_to_dev.md` (this section)

### Prior handoff

`research-20260608-bug0008` in `handoffs/po_to_tl.md` — superseded by this architecture handoff for sprint-plan.

---

## sprint-plan-20260608-s0013-us0016 — US-0016 root README living documentation

**From:** Tech Lead  
**To:** Dev (`/execute`) / QA (`/plan-verify`)  
**Date:** 2026-06-08  
**Story:** US-0016  
**Sprint:** **S0013**  
**Orchestrator:** `auto-20260606-us0016-001`  
**Next phase:** `/plan-verify`

### Summary

Standard sprint **S0013** formalizes architecture **DEC-0070** — documentation-only root README living doc. Seven tasks in dependency order: split-layout skeleton, Purpose + Product status seed, content sections, related-doc cross-links, `--no-template-parity` CI gate, runbook § README maintenance hooks, developer shard pointer. AC-6 (template parity) deferred vacuously until `template/` tree lands.

**Decision:** **DEC-0070**  
**Research:** **R-0066**, **R-0067**  
**Architecture ref:** `docs/engineering/architecture.md` § **US-0016**  
**Sprint ref:** `sprints/S0013/sprint.md`, `sprints/S0013/tasks.md`

### Execute order (frozen)

| Order | Task | Primary files | Acceptance |
|-------|------|---------------|------------|
| 1 | **T-0137** | `README.md` | AC-1, AC-2 |
| 2 | **T-0138** | `README.md` § Purpose | AC-1, AC-5 |
| 3 | **T-0139** | `README.md` (Quickstart, Examples, Limitations) | AC-1 |
| 4 | **T-0140** | `README.md` § Related documentation | AC-3 |
| 5 | **T-0141** | `tests/run-tests.sh`, runbook `TEST_COMMAND` | AC-4 |
| 6 | **T-0142** | `docs/engineering/runbook.md` | AC-5 |
| 7 | **T-0143** | `docs/developer/README.md` | AC-5 |

### Acceptance mapping

| Row | Tasks | Verify |
|-----|-------|--------|
| AC-1 | T-0137, T-0138, T-0139, T-0140 | Split layout H2s + Flow Finance AI content; validator non-stub |
| AC-2 | T-0137 | `## Contributing` → dev shard; zero DEV_* H2 in root |
| AC-3 | T-0140 | user-guides, runbook, compose commands (minimal/bundled-firefly/external) |
| AC-4 | T-0141 | `validate_doc_profile.py --repo . --no-template-parity` exit 0 CI + local |
| AC-5 | T-0142, T-0143 | Runbook § README maintenance + dev shard pointer |
| AC-6 | _(deferred)_ | Vacuous until `template/` exists; T1 flip gate out of scope |

### Frozen boundaries

- **`--no-template-parity`** until full `template/` mirror — no partial stub
- **Product status** — `###` under Purpose; 8 bullets max; `{id} — outcome` format
- **Phase-boundary hooks only** — release + refresh-context; not per-commit
- **No application code** — documentation surfaces only
- **No per-story user guide** — root links `docs/user-guides/` when `USER_GUIDE_MODE=1`

### Deploy order

```text
(T-0137 → T-0138 + T-0139 + T-0140 → T-0141) docs PR
(T-0142 → T-0143) maintenance docs (may parallelize with content)
```

No backend/frontend deploy required.

### Artifacts created

- `sprints/S0013/sprint.md`, `sprints/S0013/sprint.json`, `sprints/S0013/tasks.md`, `sprints/S0013/progress.md`
- `sprints/S0013/uat.md`, `sprints/S0013/uat.json` (placeholders)
- `docs/engineering/state.md` — sprint-plan checkpoint
- `handoffs/tl_to_dev.md` (this section)

### Prior architecture handoff

`architecture-20260608-us0016` — superseded by this sprint-plan handoff for execute.

---

## architecture-20260608-us0016 — US-0016 root README living documentation

**From:** Tech Lead  
**To:** Sprint planner (`/sprint-plan`)  
**Date:** 2026-06-08  
**Story:** US-0016  
**Orchestrator:** `auto-20260606-us0016-001`  
**Next phase:** `/sprint-plan`

### Summary

Architecture freezes **DEC-0070** — root README split layout (US-0077 doc profile), **`### Product status`** under **`## Purpose`** (8 bullets max), **`--no-template-parity`** validator posture until full `template/` tree ships, and **release + refresh-context** maintenance hooks documented in runbook § README maintenance (execute). Documentation-only; no application code.

**Decision:** **DEC-0070**  
**Research:** **R-0066**, **R-0067**  
**Architecture ref:** `docs/engineering/architecture.md` § **US-0016**  
**Spec-pack:** `docs/engineering/spec-pack/US-0016-{design-concept,crs,technical-specification}.md`  
**Acceptance:** `docs/product/acceptance.md` § US-0016 (6 rows)

### Recommended sprint

| Field | Value |
|-------|-------|
| **Type** | Standard sprint (not `/quick`) |
| **Tasks** | ~6–8 (R1–R6 + CI validator) |
| **Split** | No — under `SPRINT_MAX_TASKS=12` |

### Execute slices (frozen — sprint-plan decomposes)

| Slice | Primary files | Acceptance |
|-------|---------------|------------|
| **R1** | `README.md` | AC-1, AC-2, AC-3 |
| **R2** | `README.md` § Purpose | AC-5 (Product status contract) |
| **R4** | CI / `tests/run-tests.sh` | AC-4 |
| **R5** | `docs/engineering/runbook.md` | AC-5 |
| **R6** | `docs/developer/README.md` | AC-5 (pointer) |
| **T1** | (deferred) | AC-6 when `template/` lands |

### Frozen contracts (DEC-0070)

| Topic | Contract |
|-------|----------|
| Split layout | 5 user H2s + `## Contributing`; no `DEV_*` in root |
| Product status | `###` under Purpose; `{US\|BUG-id} — outcome`; cap 8; link backlog |
| Validator | `--no-template-parity` until full `template/` mirror |
| Release hook | Append bullets for closed US/BUG; validator before readiness |
| Refresh hook | Verify/update status; validator if README touched |

### Out of scope

- Full `template/` tree; per-commit README automation; application code

### Artifacts written

- `docs/engineering/architecture.md` § US-0016
- `decisions/DEC-0070.md`, `docs/engineering/decisions.md`
- `docs/engineering/spec-pack/US-0016-*.md`
- `handoffs/tl_to_dev.md` (this section)

---

## sprint-plan-20260607-q0017-bug0007 — BUG-0007 AI merchant/category discovery

**From:** Tech Lead  
**To:** Dev (`/execute`) / QA (`/plan-verify`) / Operator (V1)  
**Date:** 2026-06-07  
**Work item:** BUG-0007 (defect)  
**Sprint:** **Q0017** (`/quick`)  
**Orchestrator:** `auto-20260607-bug0007-001`  
**Next phase:** `/plan-verify`

### Summary

Quick sprint **Q0017** formalizes architecture **DEC-0069** — A′+E+F intelligence-layer fix for AI merchant/category discovery **(S)**, **(T)**, **(U)**. Seven tasks in dependency order; single backend PR for A1–E2+T1; operator gate before V1.

**Decision:** **DEC-0069**  
**Research:** **R-0065**  
**Architecture ref:** `docs/engineering/architecture.md` § **BUG-0007**  
**Sprint ref:** `sprints/quick/Q0017/sprint.md`, `sprints/quick/Q0017/task.json`

### Execute order (frozen)

| Order | Task | Primary files | Acceptance |
|-------|------|---------------|------------|
| 1 | **A1** | `transactions/repository.rs`, `types.rs` | **(T)**, **(U)** |
| 2 | **A2** | `transactions/service.rs`, `ai/tools/transactions.rs` | **(T)**, **(U)** |
| 3 | **F1** | `ai/tools/subscriptions.rs` | **(S)** |
| 4 | **E1** | `ai/orchestrator.rs` | **(S)**, **(T)**, **(U)** |
| 5 | **E2** | `ai/tools/{transactions,subscriptions}.rs` (schema descriptions) | **(S)**, **(T)** |
| 6 | **T1** | `backend/tests/` or module `#[cfg(test)]` | regression |
| 7 | **V1** | verify-work omniflow AI Chat smoke | **(S)**, **(T)**, **(U)** |

### Acceptance mapping

| Row | Tasks | Verify |
|-----|-------|--------|
| **(S)** | F1, E1, E2, V1 | Chat lists named subscription merchants after cancelable-total question |
| **(T)** | A1, A2, E1, E2, V1 | Strom/Amazon amounts via category_search; 2023 Amazon cites mirror bounds |
| **(U)** | A1, A2, F1, E1, V1 | Multi-tool fusion without user-supplied merchant names |
| Footer | T1, V1 | Six tools; allow_raw_transactions=false default |

### Frozen boundaries

- **Six-tool registry** — no seventh tool; no DEC waiver
- **allow_raw_transactions=false** — default unchanged
- **BUG-0008** — additive AI JSON only; no alert/list/detection changes
- **RAG (V)** — out of scope for MVP
- **Payee aggregates (B)** — deferred
- **BUG-0006** — do not revert ingest fixes
- **No frontend changes** unless sprint-plan adds optional audit UI hint (not required for AC)

### Test contract (S/T/U)

- A1 fixture: ILIKE resolves Strom → 146, Amazon → 47; bounds min/max
- A2 integration: `category_matches`, `mirror_date_bounds`, `search_attempted` in tool JSON
- F1 unit: Counterparty-* rejected; `merchant_names` deduped; kind enum in schema
- E1 unit: audit `result_rows` non-NULL for both tools
- Privacy: six-tool registry; `allow_raw_transactions=false` → no `raw_rows`
- Operator smoke: subscription names; Strom/Amazon amounts; 2023 bounds empty-state

### Operator gates (V1)

1. **BACKEND_DEPLOY** — after deploy, backend image on omniflow before V1 AI Chat probes

### Deploy order

```text
(A1 → A2 → F1 → E1 → E2 → T1) single backend PR → deploy → V1 AI Chat smoke
```

No Firefly re-sync required.

### Artifacts created

- `sprints/quick/Q0017/sprint.json`, `sprint.md`, `tasks.md`, `task.json`, `progress.md`
- `docs/product/backlog.md` — BUG-0007 sprint_id Q0017
- `docs/engineering/state.md` — sprint-plan checkpoint
- `handoffs/resume_brief.md` — next phase plan-verify

### Prior architecture handoff

`architecture-20260607-bug0007` — superseded by this sprint-plan handoff for execute.

---

## architecture-20260607-bug0007 — BUG-0007 AI merchant/category discovery

**From:** Tech Lead  
**To:** Sprint planner / Dev (`/sprint-plan` → `/execute`)  
**Date:** 2026-06-07  
**Work item:** BUG-0007 (defect)  
**Orchestrator:** `auto-20260607-bug0007-001`  
**Next phase:** `/sprint-plan`

### Summary

Architecture freezes **DEC-0069** — **A′ + E + F** fix bundle within the six-tool registry. Server-side **`category_search`** on `get_transactions` resolves Strom→146 and Amazon→47; **`mirror_date_bounds`** closes T-a empty-period evidence; **`get_subscriptions`** schema enrichment + Counterparty guard fixes S; orchestrator prompt + **`audit.result_rows`** fixes enumeration and operator debug. No seventh tool; no RAG; BUG-0008 coordinate note only (additive AI JSON).

**Decision:** **DEC-0069**  
**Research:** **R-0065**  
**Architecture ref:** `docs/engineering/architecture.md` § **BUG-0007**  
**Acceptance:** `docs/product/acceptance.md` — BUG-0007 rows **(S)**, **(T)**, **(U)**

### Recommended sprint

| Field | Value |
|-------|-------|
| **Type** | `/quick` |
| **ID** | **Q0017** (recommended) |
| **Tasks** | 7 (A1, A2, F1, E1, E2, T1, V1) |
| **Estimate** | ~13.5h total (~12.5h dev + ~1h operator V1) |
| **Split** | No — under `SPRINT_MAX_TASKS=12` |

### Execute order (frozen)

| Order | Task | Primary files | Acceptance |
|-------|------|---------------|------------|
| 1 | **A1** | `transactions/repository.rs`, `types.rs` | **(T)**, **(U)** |
| 2 | **A2** | `transactions/service.rs`, `ai/tools/transactions.rs` | **(T)**, **(U)** |
| 3 | **F1** | `ai/tools/subscriptions.rs` | **(S)** |
| 4 | **E1** | `ai/orchestrator.rs` | **(S)**, **(T)**, **(U)** |
| 5 | **E2** | `ai/tools/{transactions,subscriptions}.rs` (schema descriptions) | **(S)**, **(T)** |
| 6 | **T1** | `backend/tests/` or module `#[cfg(test)]` | regression |
| 7 | **V1** | verify-work omniflow AI Chat smoke | **(S)**, **(T)**, **(U)** |

### Frozen contracts (DEC-0069)

#### A′ — `get_transactions.category_search`

| Slice | Contract |
|-------|----------|
| Param | `category_search` optional string; min 2 chars after trim; ILIKE on `categories.name` |
| Precedence | `category_search` wins when both `category_id` + search supplied |
| Cap | 10 category matches; `category_matches_truncated` when exceeded |
| Response | Always include `mirror_date_bounds { min, max }`; `category_matches[]`, `search_attempted` |
| Privacy | DEC-0032 unchanged; no raw_rows; no description/payee search |

#### F — `get_subscriptions`

| Slice | Contract |
|-------|----------|
| Schema | `kind` enum: `subscription` \| `standing_order` |
| Guard | Reject `Counterparty-*` in status/kind → InvalidArgs |
| Response | Add `patterns_count`, `merchant_names[]` (deduped display_name) |
| REST | **No** `list_patterns` behavior change |

#### E — Orchestrator + audit

| Slice | Contract |
|-------|----------|
| Prompt | Enumerate all display_name; category_search for keywords; cite bounds on empty period |
| Audit | Populate `result_rows` for get_transactions (bucket count) and get_subscriptions (patterns_count) |
| Local | No `tool_choice: required` |

### Acceptance mapping

| Row | Tasks | Verify |
|-----|-------|--------|
| **(S)** | F1, E1, E2, V1 | Chat lists named subscription merchants after cancelable-total question |
| **(T)** | A1, A2, E1, V1 | Strom/Amazon amounts via category_search; 2023 Amazon cites mirror bounds |
| **(U)** | A1, A2, F1, E1, V1 | Multi-tool fusion without user-supplied merchant names |
| Footer | T1, V1 | Six tools; allow_raw_transactions=false default |

### Frozen boundaries

- **Six-tool registry** — no seventh tool; no DEC waiver
- **allow_raw_transactions=false** — default unchanged
- **BUG-0008** — additive AI JSON only; no alert/list/detection changes
- **RAG (V)** — out of scope for MVP
- **Payee aggregates (B)** — deferred
- **BUG-0006** — do not revert ingest fixes
- **No frontend changes** unless sprint-plan adds optional audit UI hint (not required for AC)

### Deploy order

```text
(A1 → A2 → F1 → E1 → E2 → T1) single backend PR → deploy → V1 AI Chat smoke
```

No Firefly re-sync required.

### Operator gates (V1)

1. Probe **S:** ask cancelable streaming total, then "liste mir die dienste auf" — expect named merchants from tool data
2. Probe **T-b:** Strom and Amazon spend in mirror window — expect category_search-backed amounts
3. Probe **T-a:** Amazon Jan–Oct 2023 — expect explicit empty-state with mirror_date_bounds (2025-06-05…)
4. Regression: six-tool count; settings `allow_raw_transactions=false`

### Artifacts created

- `decisions/DEC-0069.md`
- `docs/engineering/architecture.md` § **BUG-0007**
- `docs/engineering/decisions.md` — DEC-0069 index + summary
- `docs/engineering/state.md` — architecture checkpoint
- `handoffs/resume_brief.md` — sprint-plan readiness

**Status:** superseded by `sprint-plan-20260607-q0017-bug0007` above; Q0017 sprint-plan complete 2026-06-07.

### Prior handoffs (reference)

- `research-20260607-bug0007` in `handoffs/po_to_tl.md` — research complete
- `discovery-20260607-bug0007` in `handoffs/po_to_tl.md` — discovery verdicts S/T/U/V

---

## sprint-plan-20260606-q0016-bug0009 — BUG-0009 Grafana empty panels & account overview

**From:** Tech Lead  
**To:** Dev (`/execute`) / QA (`/plan-verify`) / Operator (V1)  
**Date:** 2026-06-06  
**Work item:** BUG-0009 (defect)  
**Sprint:** **Q0016** (`/quick`)  
**Orchestrator:** `auto-20260606-bug0009-001`  
**Next phase:** `/plan-verify`

### Summary

Quick sprint **Q0016** formalizes architecture **DEC-0068** — provisioning-only fix for Grafana perceived emptiness **(Y)** and missing cross-account overview **(Z)**. Six tasks in dependency order; single PR for provisioning JSON + tests (Z1–Y2, T1); operator gate before V1.

**Decision:** **DEC-0068**  
**Research:** **R-0064**  
**Architecture ref:** `docs/engineering/architecture.md` § **BUG-0009**  
**Sprint ref:** `sprints/quick/Q0016/sprint.md`, `sprints/quick/Q0016/task.json`

### Execute order (frozen)

| Order | Task | Primary files | Acceptance |
|-------|------|---------------|------------|
| 1 | **Z1** | `grafana/provisioning/dashboards/analytics/portfolio.json` | **(Z)** |
| 2 | **Z2** | `portfolio.json` (grid layout + overview table title) | **(Z)** |
| 3 | **Y1** | `cashflow.json`, `forecast-horizons.json` (`$account_id` query) | **(Y)** |
| 4 | **Y2** | `forecast-horizons.json` (ML banner + noValue) | **(Y)** |
| 5 | **T1** | SQL fixtures + optional provisioning snapshot test | **(Y)(Z)** |
| 6 | **V1** | verify-work omniflow smoke | **(Y)(Z)** |

### Acceptance mapping

| Row | Tasks | Verify |
|-----|-------|--------|
| **(Y)** | Y1, Y2, T1, V1 | Default-load cashflow/forecast non-empty; ML banner honest; ds/query 200 |
| **(Z)** | Z1, Z2, T1, V1 | Portfolio overview shows all synced asset accounts; six `/analytics/{slug}` routes smoke |

### Frozen boundaries

- **No backend changes** — mirror `accounts.balance` already correct (DEC-0060/0065)
- **No React changes** — optional Z3 docs only
- **US-0013** — ML enablement out of scope
- **Seventh overview dashboard** — rejected
- **Grafana dynamic hide rules** — rejected
- **React `/forecast` API reorder** — optional follow-up; not required for AC Y

### Test contract (Y/Z)

- SQL fixture: 3-account snapshot → breakdown query returns **3 rows**
- SQL fixture: variable query ABS sort picks funded account over zero wallet
- Operator smoke: `/analytics/cashflow` default load → non-flat series (acct 114 pattern on omniflow)
- Operator smoke: `/analytics/portfolio` → overview table 3 rows; `total_eur` stat visible
- Regression: six analytics routes + ds/query 200 (BUG-0003 H, BUG-0004 K)

### Operator gates (V1)

1. **GRAFANA_PROVISIONING_RELOAD** — after deploy, reload Grafana provisioning (container restart or poll) before V1 omniflow probes

### Deploy order

```text
(Z1 + Z2 + Y1 + Y2 + T1) single PR → deploy → Grafana provisioning reload → V1
```

### Artifacts created

- `sprints/quick/Q0016/sprint.json`, `sprint.md`, `tasks.md`, `task.json`, `progress.md`
- `docs/product/backlog.md` — BUG-0009 sprint_id Q0016
- `docs/engineering/state.md` — sprint-plan checkpoint
- `handoffs/resume_brief.md` — next phase plan-verify

### Prior architecture handoff

`architecture-20260606-bug0009` — superseded by this sprint-plan handoff for execute.

---

## architecture-20260606-bug0009 — BUG-0009 Grafana empty panels & account overview

**From:** Tech Lead  
**To:** Sprint planner / Dev (`/sprint-plan` → `/execute`)  
**Date:** 2026-06-06  
**Work item:** BUG-0009 (defect)  
**Orchestrator:** `auto-20260606-bug0009-001`  
**Next phase:** `/sprint-plan`

### Summary

Architecture freezes **DEC-0068** — provisioning-only fix for Grafana perceived emptiness (Y) and missing cross-account overview (Z). Primary Y cause is **`$account_id` alphabetical default → acct 116 (zero forecast)**; Z requires **portfolio SQL subquery fix (Z1)** + **overview table on portfolio dashboard (Z2)**. ML panels empty on omniflow is expected (DEC-0049) — close with **banner + noValue (Y2)**, not US-0013. No backend or React code in scope.

**Decision:** **DEC-0068**  
**Research:** **R-0064**  
**Architecture ref:** `docs/engineering/architecture.md` § **BUG-0009**  
**Acceptance:** `docs/product/acceptance.md` — BUG-0009 rows **(Y)**, **(Z)**

### Execute order (frozen)

| Order | Task | Primary files | Acceptance |
|-------|------|---------------|------------|
| 1 | **Z1** | `grafana/provisioning/dashboards/analytics/portfolio.json` | **(Z)** |
| 2 | **Z2** | `portfolio.json` (grid layout + overview table title) | **(Z)** |
| 3 | **Y1** | `cashflow.json`, `forecast-horizons.json` (`$account_id` query) | **(Y)** |
| 4 | **Y2** | `forecast-horizons.json` (ML banner + noValue) | **(Y)** |
| 5 | **T1** | SQL fixtures + optional provisioning snapshot test | **(Y)(Z)** |
| 6 | **V1** | verify-work omniflow smoke | **(Y)(Z)** |

### Frozen contracts (DEC-0068)

| Slice | Contract |
|-------|----------|
| **Y1** | `ORDER BY ABS(COALESCE(balance,0)) DESC, name` on `$account_id`; **omit** `current` in JSON |
| **Z1** | Latest-snapshot subquery + `LATERAL jsonb_array_elements`; remove global `LIMIT 1` on cross-join |
| **Z2** | Overview on **portfolio dashboard only** — stat row + all-accounts table; `/wealth` supplementary (Z3 docs) |
| **Y2** | Text banner + `noValue: "ML unavailable"`; **no** dynamic hide; **no** US-0013 scope |

### Acceptance mapping

| Row | Tasks | Verify |
|-----|-------|--------|
| **(Y)** | Y1, Y2, T1, V1 | Default-load cashflow/forecast non-empty; ML banner honest; ds/query 200 |
| **(Z)** | Z1, Z2, T1, V1 | Portfolio overview shows all synced asset accounts; six `/analytics/{slug}` routes smoke |

**Status:** superseded by `sprint-plan-20260606-q0016-bug0009` above; Q0016 released PASS 2026-06-06.

---
|-----|-------|--------|
| **(Y)** | Y1, Y2, T1, V1 | Default-load cashflow/forecast non-empty; ML banner honest; ds/query 200 |
| **(Z)** | Z1, Z2, T1, V1 | Portfolio overview shows all synced asset accounts; six `/analytics/{slug}` routes smoke |

**Status:** superseded by `sprint-plan-20260606-q0016-bug0009` above; Q0016 released PASS 2026-06-06.

---
