# Plan-verify findings — Q0016 / BUG-0009

**Date:** 2026-06-06  
**Role:** QA (`/plan-verify`)  
**Orchestrator:** `auto-20260606-bug0009-001`  
**Verdict:** **PASS**

## Test plan (plan-verify scope)

| Check | Source | Pass criteria |
|-------|--------|---------------|
| Acceptance triad | `acceptance.md` BUG-0009 rows Y/Z | Each row maps to ≥1 task with verify steps |
| Task completeness | `tasks.md`, `task.json` | 6 tasks; no orphans; dependencies acyclic |
| Architecture alignment | `architecture.md` § BUG-0009 | Z1, Z2, Y1, Y2, T1, V1 match DEC-0068 frozen contracts |
| Decision alignment | `decisions.md` DEC-0068 | Y1 ABS(balance); Z1 subquery+LATERAL; Z2 portfolio-only; Y2 banner+noValue |
| Discovery coverage | `backlog.md` BUG-0009 | Sub-defects Y (panel emptiness), Z (overview gap) addressed |
| Frozen boundaries | `task.json` | Provisioning-only; US-0013 out; no seventh dashboard; no dynamic hide |
| UAT readiness | V1 task spec | `uat.md` + operator smoke rows Y/Z + six routes + ds/query regression planned |

## Findings

### Coverage matrix

| Row | Criterion (summary) | Tasks | Covered |
|-----|---------------------|-------|---------|
| **(Y)** | Representative Grafana dashboards return 200 with non-empty panel values when account scope matches synced asset accounts; not persistent empty panels post-BUG-0004 | Y1, Y2, T1, V1 | Yes |
| **(Z)** | Cross-account value overview in analytics (summary panel/table or documented equivalent); six `/analytics/{slug}` routes smoke | Z1, Z2, T1, V1 | Yes |
| Regression | OIDC-enabled and bundled-firefly deploy regression checks pass | post-V1 smoke | Yes (advisory) |

### Task → acceptance map

| Task | Acceptance hooks | DEC-0068 slice |
|------|------------------|----------------|
| Z1 | **(Z)** | Latest-snapshot subquery + `LATERAL jsonb_array_elements`; remove global `LIMIT 1` |
| Z2 | **(Z)** | Stat row + all-accounts table on portfolio dashboard only |
| Y1 | **(Y)** | `$account_id` `ORDER BY ABS(COALESCE(balance,0)) DESC, name`; omit `current` |
| Y2 | **(Y)** | ML text banner + `noValue: "ML unavailable"`; US-0013 boundary preserved |
| T1 | **(Y)**, **(Z)** | SQL fixtures (3-row breakdown; ABS sort order); optional provisioning snapshot |
| V1 | **(Y)**, **(Z)** | verify-work omniflow smoke after **GRAFANA_PROVISIONING_RELOAD** |

### Dependency review

- **Order:** Z1 → Z2 → Y1 → Y2 → T1 (may start after Z1+Y1 contracts frozen) → single PR deploy → Grafana provisioning reload → V1
- **Circular deps:** none
- **Operator gates:** **GRAFANA_PROVISIONING_RELOAD** before V1 runtime probes on `financegnome.omniflow.cc`

### DEC-0068 contract checklist

| Contract element | Sprint task | Status |
|------------------|-------------|--------|
| Y1 ABS(balance) variable on cashflow + forecast-horizons | Y1 | Mapped |
| Omit `current` from provisioning JSON | Y1, T1 | Mapped |
| Z1 subquery + LATERAL breakdown SQL | Z1 | Mapped |
| Z2 portfolio-only overview (stat + table) | Z2 | Mapped |
| Y2 ML banner + noValue (not hide, not US-0013) | Y2 | Mapped |
| Provisioning-only — no backend/React | frozen_boundaries | Respected |
| `/wealth` supplementary (Z3 docs) | Z2, V1 | Mapped |

### Gaps

**0 gaps** — all acceptance rows Y/Z have primary task coverage with executable verify steps.

### Orphans

**0 orphans** — all six tasks reference Y and/or Z acceptance hooks.

### Advisories (non-blocking)

1. **ADV-1:** Acceptance regression footer (OIDC + bundled-firefly) has no dedicated dev task; operator verify-work post-V1 — not a plan-verify blocker.
2. **ADV-2:** Row **(Y)** runtime proof for cashflow/forecast default-load requires deploy + **GRAFANA_PROVISIONING_RELOAD** before V1 — operator gate documented.
3. **ADV-3:** Acceptance Y lists subscriptions/budgets in representative ds/query set; those dashboards have no `$account_id` variable (R-0064) — V1 should probe ds/query **200** regression (architecture Y3); non-empty at default load not blocked on Y1 fix.
4. **ADV-4:** All-zero balance deploy falls back to alphabetical `$account_id` default — documented edge case in frozen boundaries.
5. **ADV-5:** Overview is portfolio-dashboard-only; cashflow-first operators navigate via sidebar — Z3 docs mitigate; not AC Z blocker.
6. **ADV-6:** ML charts remain visually empty below banner until US-0013 — Y2 honest empty-state closes BUG-0009 Y2 sub-defect only.
7. **ADV-7:** Manual Grafana UI save may bake `current` — V1 runbook warning required at execute.

## Recommendation

Approve sprint for **`/execute`**. No `handoffs/qa_to_dev.md` fix list required.
