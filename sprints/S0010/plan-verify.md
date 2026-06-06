# Plan-verify — Sprint S0010 / US-0010

**Sprint:** S0010  
**Story:** US-0010 — External Firefly/Postgres & Traefik deployment on omniflow host  
**Verified at:** 2026-06-02T14:00:00Z  
**Role:** QA  
**Verdict:** **PASS**

## Inputs

| Source | Path |
|--------|------|
| Acceptance | `docs/product/acceptance.md#US-0010` |
| Tasks | `sprints/S0010/tasks.md` |
| Architecture | `decisions/DEC-0056.md`, `docs/engineering/architecture.md#US-0010` |
| Sprint plan handoff | `handoffs/tl_to_dev.md#sprint-plan-20260602-s0010` |

## Test plan (coverage review)

For each acceptance criterion, confirm at least one task with explicit done-when checks and execution-order feasibility.

| AC | Criterion (abbrev.) | Tasks | Covered |
|----|---------------------|-------|---------|
| AC-1 | External profile does not start `firefly-iii` / `postgres` | T-0109, T-0111, T-0113, T-0115, T-0117, T-0118 | yes |
| AC-2 | App on `traefik`; DNS to `postgres` / `firefly` | T-0111, T-0114, T-0117, T-0118 | yes |
| AC-3 | Traefik route + `auth` + TLS | T-0110, T-0112, T-0114, T-0116, T-0118 | yes |
| AC-4 | `/health` OK with external DB + PAT | T-0112, T-0114, T-0116, T-0117, T-0118 | yes |
| AC-5 | `.env.example` operator vars | T-0112, T-0116 | yes |
| AC-6 | Operator smoke recorded on Debian host | T-0114, T-0116, T-0118 | yes |

### Task traceability

| Task | Acceptance refs (tasks.md) | Orphan |
|------|---------------------------|--------|
| T-0109 | AC-1 | no |
| T-0110 | AC-3 | no |
| T-0111 | AC-1, AC-2 | no |
| T-0112 | AC-5 (+ docs for AC-3/4) | no |
| T-0113 | AC-1 | no |
| T-0114 | AC-1–AC-4, AC-6 | no |
| T-0115 | AC-1 | no |
| T-0116 | AC-5, AC-6 | no |
| T-0117 | AC-2 | no |
| T-0118 | AC-4, AC-6 | no |

## Findings

### Gaps

None. All six acceptance criteria have task coverage with done-when criteria and CI/smoke validation paths.

### Advisories (non-blocking)

1. **ADV-1:** Runtime AC-3/AC-4/AC-6 proof is operator-host dependent (T-0118). Plan allows PENDING smoke with documented blocker; QA phase must enforce closure or explicit PASS-with-prerequisites.
2. **ADV-2:** AC-1 wording allows `--profile external` alone; sprint canonicalizes two-file merge (`docker-compose.yml` + `docker-compose.external.yml`) as documented equivalent — consistent with DEC-0056 and T-0114/T-0116.

## Summary

| Metric | Value |
|--------|-------|
| Acceptance criteria | 6 |
| Covered | 6 |
| Gaps | 0 |
| Tasks | 10 |
| Orphan tasks | 0 |
| Sprint split needed | no (10/12 tasks) |

## Decision

**Approve for `/execute`** in a fresh Dev subagent context. Execute T-0109 → T-0118 per recommended order in `sprints/S0010/tasks.md`.

## Next phase

`/execute` — implement T-0109 through T-0118; then `/qa` after `handoffs/dev_to_qa.md`.
