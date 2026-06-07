# Q0019 — BUG-0011 planning mode broken (AD/AE/AF)

| Field | Value |
|-------|-------|
| **ID** | Q0019 |
| **Type** | `/quick` |
| **Status** | PLANNED |
| **Bug** | BUG-0011 |
| **Created** | 2026-06-08 |
| **Architecture** | `architecture-20260608-bug0011` (`docs/engineering/architecture.md` § BUG-0011) |
| **Handoff** | `handoffs/tl_to_dev.md` (`sprint-plan-20260608-q0019-bug0011`) |
| **Acceptance** | `docs/product/acceptance.md` — BUG-0011 rows **AD**, **AE**, **AF** |
| **Task count** | 11 |
| **Next phase** | `/plan-verify` |

## Goal

Close BUG-0011 on US-0010 external omniflow: **DEC-0073** AE overlay-only compare `monthly_delta_sum` (zero-overlay → 0.00), **DEC-0074** AF HTTP 200 tagged `no_active_plan` for plan-vs-actual, **AD** first-run empty create + inline add/edit adjustment UX; integration tests (**T1**); operator OIDC `/planning` three-tab smoke (**V1**) after deploy.

## Scope

| Slice | Tasks | Layer |
|-------|-------|-------|
| AE — Compare overlay delta (DEC-0073) | AE1–AE3 | backend plan, tests |
| AF — Plan-vs-actual empty API (DEC-0074) | AF1–AF2 | backend API, frontend PVA tab |
| AD — First-run + add-line UX | AD1–AD4 | frontend Scenarios/Compare |
| Tests | T1 | backend integration |
| Verify | V1 | uat + operator omniflow smoke |

**Out of scope:** Grafana Dashboard 3 (`budgets.json`) SQL; auto-activate on create; US-0014 holistic UX epic; AI `simulate_plan` changes.

## Task summary

| ID | Title | Est. | Depends | Acceptance hook |
|----|-------|------|---------|-----------------|
| AE1 | Overlay delta helper (`monthly_overlay_delta_sum`) | 2h | — | **AE** |
| AE2 | Wire repository + service compare paths | 2h | AE1 | **AE** |
| AE3 | Compare metric unit tests | 2h | AE2 | **AE** |
| AF1 | Tagged PVA API 200 `no_active_plan` | 2h | AE1 | **AF** |
| AF2 | PVA guided empty state | 2h | AF1 | **AF** |
| AD1 | First-run Create empty plan | 2h | AF1 | **AD** |
| AD2 | Inline add/edit adjustment form | 3h | AD1 | **AD** |
| AD3 | Custom Apply toast + invalidation | 1h | AD2 | **AD** |
| AD4 | Compare footnote + Set active banner | 1h | AD2 | **AD**, **AE** |
| T1 | Compare + PVA integration tests | 2h | AE3, AF1 | **AD/AE/AF** |
| V1 | verify-work OIDC `/planning` smoke | 1h | AF2, AD4, T1 deploy | footer |

**Total estimate:** ~20h (~19h dev + ~1h operator V1).

## Deploy order

```text
(AE1 → AE2) overlay helper + compare paths
→ AE3 unit tests
→ AF1 tagged PVA API
→ (AF2 + AD1 → AD2 → AD3 → AD4) frontend
→ T1 integration tests
→ deploy backend + frontend on omniflow
→ V1 verify-work
```

**Sequencing rule:** AE-before-AF mandatory — AE1 helper frozen before AF1 API shape. AD PVA UX after AF1 contract frozen. Grafana Dashboard 3 unchanged (R-0020).

## Acceptance mapping

| Row | Primary tasks | Verify |
|-----|---------------|--------|
| **AD** | AD1–AD3, T1, V1 | Create empty plan + add-line UX; not silent no-op |
| **AE** | AE1–AE3, AD4, T1, V1 | Zero/neutral compare deltas on empty plan; footnote explains projected balance |
| **AF** | AF1–AF2, T1, V1 | PVA 200 JSON; guided tab when no active plan |
| Regression | T1, V1 | OIDC `/planning` Scenarios + Compare + Plan vs Actual |

## Architecture → sprint mapping

| Architecture ID | Disposition |
|-----------------|-------------|
| AE1 | Task **AE1** |
| AE2 | Task **AE2** |
| AE3 | Task **AE3** |
| AF1 | Task **AF1** |
| AF2 | Task **AF2** |
| AD1 | Task **AD1** |
| AD2 | Task **AD2** |
| AD3 | Task **AD3** |
| AD4 | Task **AD4** |
| T1 | Task **T1** |
| V1 | Task **V1** |

## Frozen boundaries

See `task.json` `frozen_boundaries`.
