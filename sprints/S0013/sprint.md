# Sprint S0013

**ID:** S0013  
**Story:** US-0016 — Root README for operators and contributors (living documentation)  
**Status:** PLANNED  
**Created:** 2026-06-08  
**Orchestrator:** `auto-20260606-us0016-001`

## Goal

Deliver **DEC-0070** root README living documentation: create a DEC-0059 split-layout `README.md` with Flow Finance AI-specific content, capped **`### Product status`** under **`## Purpose`**, related-doc cross-links (including `docs/user-guides/` per `USER_GUIDE_MODE=1`), **`--no-template-parity`** validator gate in CI/local test path, and phase-boundary maintenance hooks documented in runbook § README maintenance (US-0016) plus developer shard pointer.

## Scope

| Slice | Tasks | Primary files |
|-------|-------|---------------|
| R1 — Split layout + content | T-0137, T-0139 | `README.md` |
| R2 — Product status | T-0138 | `README.md` § Purpose |
| R3 — Related docs + compose | T-0140 | `README.md` § Related documentation |
| R4 — Validator + CI gate | T-0141 | `tests/run-tests.sh`, `docs/engineering/runbook.md` (TEST_COMMAND) |
| R5 — Runbook maintenance hooks | T-0142 | `docs/engineering/runbook.md` |
| R6 — Developer shard pointer | T-0143 | `docs/developer/README.md` |

**Out of scope:** Full `template/` tree (T1 flip gate deferred); per-commit README automation; application code; per-story `docs/user-guides/US-0016.md` (root links only per architecture).

## Risks

| Risk | Mitigation | Ref |
|------|------------|-----|
| Stale Product status | Release fail-closed validator + refresh-context verify step | R-0067 §3, T-0142 |
| `--no-template-parity` left on after template ships | Runbook flip-gate note + DEC-0070 contract | DEC-0070, T-0142 |
| Scope creep (backlog dump in README) | 8-bullet cap + backlog link only | R-0066, T-0138 |
| Operator confusion (two validator commands) | Runbook documents both commands | R-0067 §1, T-0142 |
| Non-stub content rejected by validator | Content sourced from `.env.example`, runbook, product vision | R-0066, T-0137–T-0140 |

## Definition of Done

- All 7 sprint tasks complete (`T-0137` … `T-0143`)
- `/plan-verify` coverage: no gaps against `docs/product/acceptance.md` § US-0016 (6 AC; AC-6 vacuous until `template/` exists)
- `python scripts/validate_doc_profile.py --repo . --no-template-parity` exits **0** locally and in CI test path
- Root `README.md` passes split-layout validator with Flow Finance AI-specific (non-stub) content
- Runbook § README maintenance documents release + refresh-context hooks and template flip gate

## Architecture references

- `docs/engineering/architecture.md` § US-0016
- `decisions/DEC-0070.md`
- Research: R-0066, R-0067
- Spec-pack: `docs/engineering/spec-pack/US-0016-{design-concept,crs,technical-specification}.md`
- Discovery: `handoffs/po_to_tl.md#discovery-20260608-us0016`
- Acceptance: `docs/product/acceptance.md` § US-0016
- Handoff: `handoffs/tl_to_dev.md` (`sprint-plan-20260608-s0013-us0016`)

## Next phase

`/plan-verify` in fresh subagent/chat
