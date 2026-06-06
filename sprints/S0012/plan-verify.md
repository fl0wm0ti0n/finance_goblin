# Plan-verify — Sprint S0012 / US-0012

**Sprint:** S0012  
**Story:** US-0012 — Auto-provision application database on first start  
**Verified at:** 2026-06-03T06:00:00Z  
**Role:** QA  
**Verdict:** **PASS**

## Inputs

| Source | Path |
|--------|------|
| Acceptance | `docs/product/acceptance.md#US-0012` |
| Tasks | `sprints/S0012/tasks.md` |
| Architecture / decision | `decisions/DEC-0058.md`, `docs/engineering/architecture.md` (US-0012) |
| Sprint plan handoff | `handoffs/tl_to_dev.md` (sprint-plan-20260603-s0012) |
| Research | R-0055, R-0053 §1 |

## Test plan (coverage review)

For each acceptance criterion, confirm at least one task with explicit done-when checks and execution-order feasibility.

| AC | Criterion (abbrev.) | Tasks | Covered |
|----|---------------------|-------|---------|
| AC-1 | Cold start creates missing DB idempotently before migrations | T-0130, T-0132, T-0135 | yes |
| AC-2 | Existing DB never dropped or recreated | T-0130, T-0135 | yes |
| AC-3 | TimescaleDB extension attempt; actionable fail when server lacks extension | T-0130, T-0134, T-0135 | yes |
| AC-4 | DATABASE_BOOTSTRAP_URL when app role lacks CREATEDB; fail closed on privilege | T-0130, T-0131, T-0135 | yes |
| AC-5 | .env.example and runbook document bootstrap env and omniflow behavior | T-0133, T-0134 | yes |
| AC-6 | Automated test proves create-if-missing path | T-0135, T-0136 | yes |

### DEC-0058 alignment

| Contract element | Task(s) | Aligned |
|------------------|---------|---------|
| `ensure_database` before `connect_with_retry` | T-0130, T-0132 | yes |
| `backend/src/db/bootstrap.rs` | T-0130 | yes |
| `DATABASE_BOOTSTRAP_URL` + `maintenance_database_url()` | T-0131 | yes |
| `CREATE DATABASE … OWNER` idempotent | T-0130 | yes |
| Extension via maintenance creds on app DB | T-0130 | yes |
| `DATABASE_NAME` allowlist at config load | T-0131, T-0136 | yes |
| Share DEC-0003 `startup_retry_*` | T-0130 | yes |
| `bootstrap_reason` log codes + redaction | T-0130, T-0136 | yes |
| `.env.example` omniflow block | T-0133 | yes |
| Runbook Omniflow preflight delta | T-0134 | yes |
| Gated integration test + `run-tests.sh` | T-0135 | yes |

Frozen boundaries (no Compose init, no embedded Postgres, no role auto-create, host TimescaleDB install operator-owned, migration 001 unchanged, no secret logging) are reflected in sprint scope and task descriptions.

### Task traceability

| Task | Acceptance refs (tasks.md) | Orphan |
|------|---------------------------|--------|
| T-0130 | AC-1, AC-2, AC-3, AC-4 | no |
| T-0131 | AC-4 | no |
| T-0132 | AC-1 | no |
| T-0133 | AC-5 | no |
| T-0134 | AC-5 | no |
| T-0135 | AC-6 | no |
| T-0136 | AC-6 | no |

### Dependency review

| Check | Result |
|-------|--------|
| Circular dependencies | none |
| Execution order feasible | yes — T-0131 → T-0130 → T-0132 → T-0133 ∥ T-0134 → T-0136 → T-0135 |
| Parallel paths valid | T-0133 ∥ T-0134 after T-0131/T-0130 |

### Test coverage review

| Layer | Task | Scope |
|-------|------|-------|
| Unit | T-0136 | Name allowlist, URL builder, reason-code mapping, log redaction |
| Integration (gated) | T-0135 | Create-if-missing, idempotent skip, optional privilege-fail |
| UAT (post-execute) | `sprints/S0012/uat.md` | UAT-1..UAT-6 mapped to AC-1..AC-6 |

## Findings

### Gaps

None. All six acceptance criteria have task coverage with done-when criteria and test/UAT validation paths.

### Advisories (non-blocking)

1. **ADV-1:** AC-3/AC-4 runtime proof on omniflow or TimescaleDB fixture is operator/env dependent at QA — uat.md operator smoke covers post-execute.
2. **ADV-2:** T-0132 acceptance refs understate AC-2–AC-4 linkage; T-0130 and coverage map are authoritative.
3. **ADV-3:** Default CI may skip T-0135 when `DATABASE_BOOTSTRAP_TEST_URL` unset; tiered test strategy is documented in architecture.
4. **ADV-4:** T-0134 optional DEC-0056 footnote is enhancement only.

## Summary

| Metric | Value |
|--------|-------|
| Acceptance criteria | 6 |
| Covered | 6 |
| Gaps | 0 |
| Tasks | 7 |
| Orphan tasks | 0 |
| DEC-0058 aligned | yes |
| Sprint split needed | no (7/12 tasks) |

## Decision

**Approve for `/execute`** in a fresh Dev subagent context. Execute T-0131 → T-0130 → T-0132 → T-0133/T-0134 → T-0136 → T-0135 per `sprints/S0012/tasks.md`.

## Next phase

`/execute` — implement T-0130 through T-0136; then `/qa` after `handoffs/dev_to_qa.md`.
