# PO to TL archive pack (2026-06-06)

- Rollover trigger: `PO_TO_TL_HOT_MAX_LINES=500, PO_TO_TL_HOT_MAX_SECTIONS=40`
- Source: `handoffs/po_to_tl.md`
- Archived units (oldest first, contiguous prefix): 1
- Retained units in hot file: 9
- First archived heading: `## intake-20260608-us0013 — US-0013 production ML hardening (re-intake)`
- Last archived heading: `## intake-20260608-us0013 — US-0013 production ML hardening (re-intake)`
- Verification tuple (mandatory):
  - archived_body_lines=86
  - retained_body_lines=483

---

## intake-20260608-us0013 — US-0013 production ML hardening (re-intake)

**From:** PO  
**To:** Tech Lead  
**Date:** 2026-06-08  
**Story:** US-0013  
**Orchestrator run:** auto-20260608-us0013-001  
**Next phase:** `/discovery`

### Summary

Formal **re-intake** on P0 epic **US-0013** after **BUG-0010** Q0013 release. Baseline forecast/wealth defects (AA/AB/AC) are **DONE**; remaining gap is **AC3 ML production path** on US-0010 external omniflow profile: `stats-forecast` sidecar today is **`full` profile only**, `[forecast_ml] enabled=false` by default (DEC-0049), zero `ml_enhanced` computations on production. Intake refines scope, expands acceptance **6→10 rows** (9 open + 1 prerequisite checked), and recommends **4 vertical slices** at sprint-plan without splitting backlog IDs.

### Scope (bounded)

| In | Out |
|----|-----|
| External overlay sidecar + `FORECAST_ML_ENABLED` on omniflow | New model research (US-0009 / R-0043 ladder) |
| Sync `forecast_ml` phase + `ml_enhanced` persistence (DEC-0050/0052) | Raw transaction ML training |
| React Compare + wealth ML overlay (US-0009 parity) | Monthly bucket AI (**US-0015**) |
| Grafana `$forecast_variant=ml_enhanced` panels with data | BUG-0009 banner-only empty-state (already DONE) |
| Runbook + CI sidecar fixture | Host secret reads |

### Decomposition recommendation (for `/sprint-plan`)

**Evaluator:** high breadth — 5+ workflow steps, 4+ component surfaces, 10 acceptance rows → exceeds `SPRINT_MAX_TASKS=12` as undifferentiated epic.

**Decision:** retain **single epic US-0013**; slice at sprint-plan:

| Slice | Title | Boundary |
|-------|-------|----------|
| **US-0013-S1** | External compose + ML config | `docker-compose.external.yml`, `.env.example`, `[forecast_ml]` merge |
| **US-0013-S2** | Sync ML pipeline + API | `forecast_ml` phase, sidecar health, `variant=ml_enhanced` |
| **US-0013-S3** | UI + Grafana parity | `/forecast` Compare, `/wealth` overlay, forecast-horizons ML panels |
| **US-0013-S4** | Runbook + CI fixture | Operator omniflow ML ops doc; mock sidecar test |

**Alternatives rejected:** new US-0017..0020 now (epic continuity); mega-sprint without slices (SPRINT_AUTO_SPLIT=1).

### Intake evidence (US-0078 / DEC-0060)

| Field | Value |
|-------|-------|
| `intake_run_id` | `intake-20260608-us0013` |
| `selected_pack` | `small-intake-pack` |
| `asked_topics` | outcome_success_criteria, impacted_components, constraints_compatibility_risks, required_tests_acceptance_checks, done_definition |
| `missing_topics` | (none) |
| `assumptions_confirmed` | (none) |
| Bundle | `handoffs/intake_evidence/intake-20260608-us0013.json` |
| Validator | `python scripts/intake_evidence_validate.py --file …` → **PASS** |
| Prior intake | `intake-20260605-forecast-wealth-ml` (BUG-0010 deferral origin) |

### Research pointer

[R-0071](docs/engineering/research.md#r-0071--us-0013-production-ml-enablement-on-omniflow-external-profile) — enablement path, slice boundaries, risks (R-0034 FX, min history gate).

### Risks (carry to discovery)

1. **Profile union** — external + full sidecar duplication if misconfigured; document compose invocation
2. **Min history gate** — <12 monthly points → skip or SeasonalNaive (DEC-0051); operator expectation management
3. **Host resource** — StatsForecast container on shared omniflow; footprint per R-0044
4. **FX incomplete crypto** — R-0034 low-confidence banner on wealth, not hard block
5. **Deploy prerequisite** — operator **BACKEND_FRONTEND_DEPLOY** before runtime smoke (state checkpoint 2026-06-08)

### Acceptance delta

- **Before:** 6 rows (generic sidecar/compare/wealth/runbook + BUG-0010 parent)
- **After:** 10 rows — compose config, sync phase, API variant, UI Compare, wealth overlay, Grafana panels, runbook, CI test, prerequisite checked
- **Net added:** 4 open rows + prerequisite row checked (BUG-0010 DONE)

### Recommended next steps

1. **`/discovery`** — UX references for Compare/bands on production; confirm Grafana panel expectations post-enablement
2. **`/research`** — extend R-0071 if discovery surfaces open questions (external overlay compose pattern, sidecar health SLO)
3. **`/architecture`** — DEC for external-profile ML enablement contract
4. **`/sprint-plan`** — materialize US-0013-S1..S4 tasks

### Evidence

- Vision: `docs/product/vision.md` (Intake notes US-0013 2026-06-08)
- Backlog: `docs/product/backlog.md#US-0013`
- Acceptance: `docs/product/acceptance.md` § US-0013 (10 rows)
- Architecture: `docs/engineering/architecture.md` § US-0009, US-0010
- Prior research: R-0043, R-0044, R-0062, DEC-0049

---

