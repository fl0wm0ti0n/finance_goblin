# PO to TL archive pack (2026-06-09)

- Rollover trigger: `PO_TO_TL_HOT_MAX_LINES=500, PO_TO_TL_HOT_MAX_SECTIONS=40`
- Source: `handoffs/po_to_tl.md`
- Archived units (oldest first, contiguous prefix): 2
- Retained units in hot file: 12
- First archived heading: `## architecture-20260609-bug0017 — BUG-0017 forecast recompute architecture (hot pointer)`
- Last archived heading: `## research-20260609-bug0017 — BUG-0017 forecast recompute research (hot pointer)`
- Verification tuple (mandatory):
  - archived_body_lines=89
  - retained_body_lines=500

---

## architecture-20260609-bug0017 — BUG-0017 forecast recompute architecture (hot pointer)
**From:** Tech Lead **To:** Sprint Plan **Bug:** BUG-0017 **Run:** `intake-20260609-ui-audit` **Next:** `/sprint-plan` **Full:** below · [R-0087](docs/engineering/research.md#r-0087--bug-0017-post-sync-forecast-recompute-cluster-audit-check-fk-retention-ml-gate-forecastpage-loading) · **DEC-0105**, **DEC-0106** · AY–BD

---

**From:** Tech Lead  
**To:** Sprint Plan  
**Date:** 2026-06-09  
**Bug:** BUG-0017  
**Orchestrator run:** `intake-20260609-ui-audit`  
**Next phase:** `/sprint-plan`

### Summary

Architecture complete for BUG-0017 post-sync forecast recompute cluster. Accepted **DEC-0105** (audit CHECK migration) and **DEC-0106** (`paired_baseline_id` ON DELETE CASCADE + ml_enhanced-before-baseline retention order). **BD** `isFetched` empty guard frozen in architecture. **BB** month-bucket SQL probe and **BC** Planning Compare re-smoke scoped to verify-work only. Sync fail-on-recompute **deferred** (R-0050). Sprint shape: `/quick` ≤6 tasks.

### Sprint task sketch (for `/sprint-plan`)

| Task | AC | Surface |
|------|-----|---------|
| AY1 | AY, AZ | Migration `0XX_bug0017_ai_audit_forecast.sql` per DEC-0105 |
| BA1 | BA | FK CASCADE migration per DEC-0106 |
| BA2 | BA | `enforce_retention` kind order `ml_enhanced` → `baseline` |
| BD1 | BD | `ForecastPage.tsx` `isFetched` empty guard |
| T1 | BA | Retention integration test (paired rows) |
| V1 | AY–BD | Verify-work: sync smoke, month-bucket probe, planning, nav |

### Architecture gates resolved

| # | Gate | Resolution |
|---|------|------------|
| 1 | Audit CHECK | **DEC-0105** |
| 2 | FK retention | **DEC-0106** |
| 3 | Retention loop order | ml_enhanced before baseline (DEC-0106) |
| 4 | BB verification | Month-bucket SQL — verify-work |
| 5 | BC plan stale | Downstream — verify-work only |
| 6 | BD loading UX | `isFetched` guard in architecture |
| 7 | Sync fail-on-recompute | **Deferred** |
| 8 | Sprint shape | `/quick` ≤6 tasks |

**Evidence:** `docs/engineering/architecture.md` § BUG-0017, `decisions/DEC-0105.md`, `decisions/DEC-0106.md`, spec-pack BUG-0017, [R-0087](docs/engineering/research.md#r-0087--bug-0017-post-sync-forecast-recompute-cluster-audit-check-fk-retention-ml-gate-forecastpage-loading)

`triad_hot_surface`: architecture BUG-0017 prepended; DEC-0105/0106 created; next **sprint-plan** (2026-06-10T00:00:00Z)

## research-20260609-bug0017 — BUG-0017 forecast recompute research (hot pointer)
**From:** Tech Lead **To:** Architecture **Bug:** BUG-0017 **Run:** `intake-20260609-ui-audit` **Next:** `/architecture` **Full:** below · [R-0087](docs/engineering/research.md#r-0087--bug-0017-post-sync-forecast-recompute-cluster-audit-check-fk-retention-ml-gate-forecastpage-loading) · discovery `po-to-tl-pack-20260609-h.md` · AY–BD

**From:** Tech Lead  
**To:** PO / Architecture  
**Date:** 2026-06-09  
**Bug:** BUG-0017  
**Orchestrator run:** `intake-20260609-ui-audit`  
**Next phase:** `/architecture`

### Summary

Web + code research completed for BUG-0017 post-sync forecast recompute cluster. Added **[R-0087](docs/engineering/research.md#r-0087--bug-0017-post-sync-forecast-recompute-cluster-audit-check-fk-retention-ml-gate-forecastpage-loading)** — two confirmed backend defects (audit CHECK + FK retention), one verify-after-fix ML gate, two downstream/UX symptoms. No host `.env` or secrets read.

### Key findings

| Topic | Research | Recommendation for architecture |
|-------|----------|--------------------------------|
| **AY/AZ audit CHECK** | [R-0087 §2](docs/engineering/research.md#r-0087--bug-0017-post-sync-forecast-recompute-cluster-audit-check-fk-retention-ml-gate-forecastpage-loading) | New migration DROP+ADD `tool_name` + `result_status` CHECK; add `forecast_bucket_assignment` + `low_confidence`/`provider_unavailable`/`parse_error` |
| **BA FK retention** | [R-0087 §3](docs/engineering/research.md#r-0087--bug-0017-post-sync-forecast-recompute-cluster-audit-check-fk-retention-ml-gate-forecastpage-loading) | `ON DELETE CASCADE` on `paired_baseline_id` + ml-before-baseline prune order |
| **BB ML gate** | [R-0087 §4](docs/engineering/research.md#r-0087--bug-0017-post-sync-forecast-recompute-cluster-audit-check-fk-retention-ml-gate-forecastpage-loading) | Verify after AY–BA via month-bucket SQL probe; do not lower `min_monthly_points` |
| **BC plan stale** | [R-0087 §5](docs/engineering/research.md#r-0087--bug-0017-post-sync-forecast-recompute-cluster-audit-check-fk-retention-ml-gate-forecastpage-loading) | Downstream of BA; re-smoke Planning Compare post-fix |
| **BD empty flash** | [R-0087 §6](docs/engineering/research.md#r-0087--bug-0017-post-sync-forecast-recompute-cluster-audit-check-fk-retention-ml-gate-forecastpage-loading) | `ForecastPage`: empty only when `metaQuery.isFetched && !computation_id` |

### Architecture decision gates

1. **DEC candidate** — audit CHECK migration contract (AY/AZ)
2. **DEC candidate** — `paired_baseline_id ON DELETE CASCADE` (BA)
3. **Retention loop** — ml_enhanced before baseline (defense in depth)
4. **BB probe** — month-bucket SQL on operator mirror before declaring ML defect
5. **BD contract** — `isFetched` empty guard; loading skeleton while `isPending`
6. **Sync semantics** — defer fail-on-recompute (keep R-0050 warn-and-serve)
7. **Sprint shape** — recommend `/quick` (migration + repository + ForecastPage; ≤6 tasks)

### Risks surfaced (carry to architecture)

1. Constraint name drift on operator DB — verify before migration
2. CASCADE deletes paired ML on baseline prune — acceptable but document
3. Masking true `insufficient_history` if threshold lowered — reject
4. BD error state must not show false empty — guard `!isError`

**Evidence:** [R-0087](docs/engineering/research.md#r-0087--bug-0017-post-sync-forecast-recompute-cluster-audit-check-fk-retention-ml-gate-forecastpage-loading), `handoffs/archive/po-to-tl-pack-20260609-h.md`, `backend/migrations/006_ai_audit.sql`, `backend/src/forecast/repository.rs`, `frontend/src/pages/ForecastPage.tsx`

`triad_hot_surface`: research BUG-0017 prepended; R-0087 appended; next **architecture** (2026-06-09T23:30:00Z)

