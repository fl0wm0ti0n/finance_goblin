# Design Concept — BUG-0011

## Summary

BUG-0011 restores operator-usable **Planning** mode on `/planning`: editable empty plans (AD), correct Compare overlay deltas (AE), and Plan vs Actual tab survival without 404 (AF). Two architecture decisions — **DEC-0073** and **DEC-0074** — fix backend contracts; AD UX wiring is execute scope without a third DEC. Holistic first-visit polish remains **US-0014**.

## Goals

- Overlay-only `monthly_delta_sum` on Compare (DEC-0073 AE)
- HTTP 200 tagged `no_active_plan` for plan-vs-actual (DEC-0074 AF)
- First-run **Create empty plan** + inline add-adjustment form (AD)
- Preserve US-0004 explicit **Set active** semantics
- Grafana Dashboard 3 unchanged (R-0020)
- OIDC `/planning` three-tab regression smoke

## Non-goals

- Grafana `budgets` panel SQL changes
- Auto-activate first plan globally
- Full planning wizard / tooltip epic (US-0014)
- AI `simulate_plan` chat changes (US-0006)
- Hypertable overlay series storage

## Key decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| DEC-0073 | Overlay-only monthly delta | R-0070 §2; R-0016 label semantics |
| DEC-0073 | Projected balance unchanged | Full scenario total — not overlay metric |
| DEC-0074 | 200 `no_active_plan` tagged JSON | Mirror risk-score; acceptance AF |
| DEC-0074 | No auto-activate | US-0004 explicit Set active |
| AD bundle | Inline form + empty create | R-0070 §4–5; no modal except Savings |

**Spec-pack traceability:** `docs/engineering/spec-pack/BUG-0011-crs.md`, `docs/engineering/spec-pack/BUG-0011-technical-specification.md`
