# Q0021 — US-0017 README living-doc expansion and troubleshooting

| Field | Value |
|-------|-------|
| **ID** | Q0021 |
| **Type** | `/quick` |
| **Status** | PLANNED |
| **Story** | US-0017 |
| **Created** | 2026-06-09 |
| **Architecture** | `architecture-20260609-us0017` (`docs/engineering/architecture.md` § US-0017) |
| **Handoff** | `handoffs/tl_to_dev.md` (`sprint-plan-20260609-q0021-us0017`) |
| **Acceptance** | `docs/product/acceptance.md` § US-0017 (AC-1 … AC-5) |
| **Task count** | 7 (6 architecture slices + 1 user guide; `USER_GUIDE_MODE=1`) |
| **Next phase** | `/plan-verify` |

## Goal

Close **US-0017** as a **DEC-0070** extension: add H3 **Omniflow smoke** and **Troubleshooting** subsections to root `README.md`, tighten per-segment Product status maintenance hooks in developer shard and runbook, verify Product status bullets, publish operator user guide, and pass `validate_doc_profile --no-template-parity`. **Doc-only — no application code.**

## Scope

| Slice | Tasks | Primary files |
|-------|-------|---------------|
| E1 — Omniflow smoke H3 | E1 | `README.md` § Examples |
| E2 — Troubleshooting H3 | E2 | `README.md` § Limitations |
| E3 — Product status verify | E3 | `README.md` § Purpose |
| E4 — Developer shard hooks | E4 | `docs/developer/README.md` |
| E5 — Runbook maintenance delta | E5 | `docs/engineering/runbook.md` § README maintenance |
| E6 — Validator gate | E6 | `scripts/validate_doc_profile.py` (run only) |
| UG — Operator user guide | UG1 | `docs/user-guides/US-0017.md` |

**Out of scope:** Analytics code; dedicated `## Troubleshooting` H2; full runbook §23 inline; DEC-0081; per-commit README automation; credential literals in docs.

## Task summary

| ID | Title | Est. | Depends | Acceptance hook | Priority |
|----|-------|------|---------|-----------------|----------|
| E1 | Omniflow smoke H3 under Examples | 1.5h | — | **AC-1** | P0 |
| E2 | Troubleshooting H3 under Limitations | 1.5h | — | **AC-2** | P0 |
| E3 | Product status verify-only | 0.5h | E1, E2 | **AC-3** | P0 |
| E4 | Developer README per-segment wording | 0.5h | — | **AC-4** | P0 |
| E5 | Runbook § README maintenance delta | 0.5h | — | **AC-4** | P0 |
| UG1 | Publish `docs/user-guides/US-0017.md` | 0.5h | E1, E2 | **AC-1**, **AC-2** (operator distill) | P0 |
| E6 | `validate_doc_profile --no-template-parity` gate | 0.5h | E1–E5, UG1 | **AC-5** | P0 |

**Total estimate:** ~5.5h (6 architecture slices + user guide; < `SPRINT_MAX_TASKS=12`; no split).

## Execute order

```text
E1 ∥ E2 ∥ E4 ∥ E5 (parallel doc surfaces)
  → E3 (verify Product status)
  → UG1 (operator guide from E1/E2 content)
  → E6 (validator gate — fail closed)
```

## Acceptance mapping

| AC | Primary tasks | Verify |
|----|---------------|--------|
| **AC-1** | E1, UG1 | H3 omniflow curls, routes, exchange sanity; external profile not localhost-only |
| **AC-2** | E2, UG1 | H3 gates + symptom table; ML-unavailable vs empty-panel distinction |
| **AC-3** | E3 | Product status lists US-0015 + post-US-0016 closures (verify-only; append only if new segment closures) |
| **AC-4** | E4, E5 | Per-segment Product status hook in developer README + runbook |
| **AC-5** | E6 | `validate_doc_profile --no-template-parity` exit 0; split layout preserved |

## Architecture → sprint mapping

| Architecture ID | Task | Disposition |
|---------------|------|-------------|
| E1 | **E1** | R-0078 §2 curl block; Traefik `-u` placeholders only |
| E2 | **E2** | R-0078 §3 / Q0020 uat symptom table |
| E3 | **E3** | Verify-only per architecture |
| E4 | **E4** | R-0078 §5 developer shard |
| E5 | **E5** | Release-segment definition + per-segment hooks |
| E6 | **E6** | Fail-closed validator after all edits |
| — | **UG1** | `USER_GUIDE_MODE=1` operator guide |

## Frozen boundaries

See `task.json` `frozen_boundaries`.

## Content sources (copy, do not duplicate secrets)

- Omniflow curls: [R-0078 §2](docs/engineering/research.md#r-0078--us-0017-readme-omniflow-smoke-templates-h3-layout-validate_doc_profile-gates)
- Symptom table: R-0078 §3 / `sprints/quick/Q0020/uat.md`
- Operator gates: **BACKEND_FRONTEND_DEPLOY** → **GRAFANA_PROVISIONING_RELOAD** → **FULL_FIREFLY_SYNC** + recompute
