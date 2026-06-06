# Q0014 — BUG-0012 forecast monthly Income/Fixed buckets

| Field | Value |
|-------|-------|
| **ID** | Q0014 |
| **Type** | `/quick` |
| **Status** | PLANNED |
| **Bug** | BUG-0012 |
| **Created** | 2026-06-05 |
| **Architecture** | `architecture-20260605-bug0012` (`docs/engineering/architecture.md` § BUG-0012) |
| **Handoff** | `handoffs/tl_to_dev.md` (`sprint-plan-20260605-q0014-bug0012`) |
| **Acceptance** | `docs/product/acceptance.md` — BUG-0012 rows **(AG)**, **(AH)** |
| **Task count** | 5 |
| **Next phase** | `/execute` |

## Goal

Close BUG-0012 on US-0010 external omniflow: carry `category_id` on recurring patterns (**AH1**), attribute monthly buckets per component not net daily delta (**AG1**), unit-test AG/AH contract (**T1**), retire `categorize_delta` monthly path (**D1**), operator verify + TOML checklist (**V1**) on `financegnome.omniflow.cc` after deploy + Full Firefly sync + recompute.

## Scope

| Slice | Tasks | Layer |
|-------|-------|-------|
| AH — Fixed bucket routing | AH1 | backend forecast/recurring |
| AG — Income bucket routing | AG1 | backend project |
| Tests | T1 | backend tests |
| Cleanup | D1 | backend project |
| Verify | V1 | uat + runbook |

**Out of scope:** US-0015 (AI buckets); US-0013 (ML overlay); frontend changes; fuzzy name matching; `default.toml` expansion in code.

## Task summary

| ID | Title | Est. | Depends | Acceptance hook |
|----|-------|------|---------|-----------------|
| AH1 | RecurringPattern.category_id + recurring bucket path | 2h | — | **(AH)** |
| AG1 | Income from categorized recurring inflows | 2.5h | AH1 | **(AG)**, **(AH)** |
| T1 | Unit tests monthly_map component attribution | 1.5h | AG1 | **(AG)**, **(AH)** |
| D1 | Retire net-delta categorize_delta for monthly buckets | 0.5h | AG1 | **(AG)**, **(AH)** |
| V1 | verify-work + runbook TOML checklist | 1h | AH1–D1 deploy + sync | **(AG)**, **(AH)** |

**Total estimate:** ~7.5h (dev ~6.5h + operator V1 ~1h).

## Deploy order

```text
(AH1 + AG1 + T1 + D1) single PR  →  deploy image
                                 └→ manual Full Firefly sync + recompute (operator gate)
                                 └→ TOML extend if label mismatch (operator gate)
                                 └→ V1 verify-work on financegnome.omniflow.cc
```

## Acceptance mapping

| Row | Primary tasks | Verify |
|-----|---------------|--------|
| **(AG)** | AG1, T1, V1 | Income non-zero when mirror has income-category recurring inflows in forecast month |
| **(AH)** | AH1, AG1, T1, V1 | Fixed non-zero when mirror has fixed-cost category recurring outflows |
| Regression | post-V1 | OIDC + bundled-firefly footer; Variable still non-zero for discretionary |

## Architecture → sprint mapping

| Architecture ID | Disposition |
|-----------------|-------------|
| AH1 | Task **AH1** |
| AG1 | Task **AG1** |
| T1 | Task **T1** |
| D1 (runbook) | Folded into **V1** runbook TOML checklist |
| D1 (categorize_delta retire) | Task **D1** |
| V1 | Task **V1** |

## Frozen boundaries

See `task.json` `frozen_boundaries`.
