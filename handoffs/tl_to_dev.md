# Sprint Plan Handoff — BUG-0027 → Q0035

**Phase:** sprint-plan  
**Work item:** BUG-0027 — Firefly sync 401 Unauthorized (PAT invalid/expired after deploy)  
**Architect:** tech-lead (fresh isolated context)  
**Sprint:** Q0035 (`/quick`)  
**Sprint ID:** sprint-plan-20260622-bug0027-q0035  
**Timestamp:** 2026-06-22T22:30:00Z  
**Phase ID:** sprint-plan  
**Fresh context marker:** sprint-plan-20260622-bug0027-tl-fresh

## Sprint Summary

Sprint Q0035 (quick) — 5 tasks implementing Firefly 401 error taxonomy to surface PAT invalid/expired diagnosis to operator.

## Tasks

| ID | Title | Acceptance | Est | Gate |
|----|-------|------------|-----|------|
| **E1** | `FireflyError::Unauthorized` variant + Display | CC | 0.25h | GATE-ERROR-1 |
| **E2** | Match 401 → `Unauthorized` in `request()` | CC | 0.25h | GATE-ERROR-1 |
| **T1** | wiremock 401 test | CC, CD | 0.5h | GATE-TEST-1 |
| **G1** | Regression gates | CB, CC, CD | 0.25h | — |
| **V1** | Operator smoke (PAT regen) | CB, CD | 0.5h | — |

## Acceptance Rows

**CB** — Sync status surfaces 401 error
**CC** — App surfaces clear diagnosis on 401
**CD** — Operator regenerates PAT, sync recovers, no further 401

## File Targets

- `backend/src/firefly/mod.rs` L20-37 (enum), L128-158 (match arm)
- `backend/tests/firefly_integration.rs` (wiremock test)

## References

- Architecture: `docs/engineering/architecture.md` § BUG-0027
- Research: [R-0099 §10](docs/engineering/research.md#10-research-phase-findings-tech-lead-2026-06-22-isolated-fresh-context)
- Acceptance: `docs/product/acceptance.md` § BUG-0027
- Sprint artifacts: `sprints/quick/Q0035/`

## Next Phase

`/plan-verify` (QA) — then `/execute` (Dev).
