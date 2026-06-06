# Plan-verify — Sprint S0011 / US-0011

**Sprint:** S0011  
**Story:** US-0011 — Unified analytics UI in financegnome (Grafana in-app)  
**Verified at:** 2026-06-02T20:00:00Z  
**Role:** QA  
**Verdict:** **PASS**

## Inputs

| Source | Path |
|--------|------|
| Acceptance | `docs/product/acceptance.md#US-0011` |
| Tasks | `sprints/S0011/tasks.md` |
| Architecture / decision | `decisions/DEC-0057.md`, `docs/engineering/architecture.md#US-0011` |
| Sprint plan handoff | `handoffs/tl_to_dev.md#sprint-plan-20260602-s0011` |
| Research | R-0054, R-0056 |

## Test plan (coverage review)

For each acceptance criterion, confirm at least one task with explicit done-when checks and execution-order feasibility.

| AC | Criterion (abbrev.) | Tasks | Covered |
|----|---------------------|-------|---------|
| AC-1 | Analytics sidebar lists all six dashboards with in-app routes | T-0123, T-0124 | yes |
| AC-2 | In-app embed; no default new tab | T-0120, T-0123, T-0126, T-0127 | yes |
| AC-3 | Traefik auth / dev bypass embed works | T-0120, T-0121, T-0126, T-0127, T-0129 | yes |
| AC-4 | ECharts product pages regression | T-0128 | yes |
| AC-5 | Wealth in-app portfolio analytics (no external tab primary) | T-0125, T-0122 | yes |
| AC-6 | Future-chart guideline documented | T-0129 | yes |
| AC-7 | Single-URL operator guide; no public Grafana host required | T-0122, T-0129 | yes |

### DEC-0057 alignment

| Contract element | Task(s) | Aligned |
|------------------|---------|---------|
| `/analytics/grafana/` reverse proxy | T-0120 | yes |
| `GRAFANA_UPSTREAM` + SSRF allowlist | T-0119 | yes |
| `VITE_GRAFANA_EMBED_BASE`; deprecate `VITE_GRAFANA_URL` | T-0122, T-0125 | yes |
| Six `/analytics/{slug}` kiosk routes | T-0123 | yes |
| Analytics sidebar nav group | T-0124 | yes |
| Grafana anonymous Viewer compose env | T-0121 | yes |
| Proxy outside `/api/v1` JWT; WebSocket + framing rewrite | T-0120 | yes |
| SPA CSP `frame-src 'self'` | T-0126 | yes |
| Future-chart guideline + canonical UX table | T-0129 | yes |

Frozen boundaries (no public Grafana router, no subpath serve, no uid changes, ECharts authoritative) are reflected in task descriptions and sprint scope.

### Task traceability

| Task | Acceptance refs (tasks.md) | Orphan |
|------|---------------------------|--------|
| T-0119 | AC-2, AC-3 | no |
| T-0120 | AC-2, AC-3 | no |
| T-0121 | AC-2, AC-3 | no |
| T-0122 | AC-7 | no |
| T-0123 | AC-1, AC-2 | no |
| T-0124 | AC-1 | no |
| T-0125 | AC-5 | no |
| T-0126 | AC-2, AC-3 | no |
| T-0127 | AC-2 | no |
| T-0128 | AC-4 | no |
| T-0129 | AC-6, AC-7 | no |

### Dependency review

| Check | Result |
|-------|--------|
| Circular dependencies | none |
| Execution order feasible | yes — T-0119 ∥ T-0121 → T-0120 → T-0122 → T-0123 → T-0124/T-0125 → T-0126 → T-0127 → T-0128 → T-0129 |
| Parallel paths valid | T-0124/T-0125 after T-0123; T-0126/T-0127 after T-0120 |

### Test coverage review

| Layer | Task | Scope |
|-------|------|-------|
| Proxy integration | T-0127 | Reachability, prefix strip, framing headers, WebSocket forward |
| ECharts regression | T-0128 | `/forecast`, `/wealth`, `/planning`, `/subscriptions`, `/alerts` |
| UAT (post-execute) | `sprints/S0011/uat.md` | UAT-1..UAT-7 mapped to AC-1..AC-7 |

## Findings

### Gaps

None. All seven acceptance criteria have task coverage with done-when criteria and test/UAT validation paths.

### Advisories (non-blocking)

1. **ADV-1:** AC-3 Traefik auth runtime proof depends on omniflow host at QA/UAT; no dedicated smoke task like S0010 T-0118 — T-0127 + T-0129 cover plan level.
2. **ADV-2:** T-0127 acceptance refs understate AC-3 linkage; coverage map and task body cover proxy/WebSocket for Traefik embed path.
3. **ADV-3:** No per-slug E2E iframe load for all six dashboards in sprint — proxy smoke + UAT acceptable.
4. **ADV-4:** DEC-0057 decision gate for insufficient anonymous Viewer isolation is documented; execute must escalate if triggered.

## Summary

| Metric | Value |
|--------|-------|
| Acceptance criteria | 7 |
| Covered | 7 |
| Gaps | 0 |
| Tasks | 11 |
| Orphan tasks | 0 |
| DEC-0057 aligned | yes |
| Sprint split needed | no (11/12 tasks) |

## Decision

**Approve for `/execute`** in a fresh Dev subagent context. Execute T-0119 → T-0129 per recommended order in `sprints/S0011/tasks.md`.

## Next phase

`/execute` — implement T-0119 through T-0129; then `/qa` after `handoffs/dev_to_qa.md`.
