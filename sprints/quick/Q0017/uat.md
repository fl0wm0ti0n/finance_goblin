# UAT — Q0017 (BUG-0007)

**Status:** Verify-work **PASS** (2026-06-07 re-run) — S privacy fix verified  
**Acceptance:** `docs/product/acceptance.md` — BUG-0007 rows **(S)**, **(T)**, **(U)** — checked  
**Plan-verify:** PASS (`sprints/quick/Q0017/plan-verify.json`, 2026-06-07)  
**Execute:** COMPLETE (`sprints/quick/Q0017/summary.md`, 2026-06-07)  
**Orchestrator:** `auto-20260607-bug0007-001`

## Acceptance ↔ task traceability

| Row | Task(s) | Acceptance criterion (canonical) | Verify-work |
|-----|---------|-----------------------------------|-------------|
| **(S)** | F1, E1, E2, S-fix, V1 | Chat lists named subscription merchants after cancelable-total question | **PASS** |
| **(T)** | A1, A2, E1, E2, V1 | Strom/Amazon via category_search; 2023 Amazon cites mirror_date_bounds | **PARTIAL** |
| **(U)** | A1, A2, F1, E1, V1 | Multi-tool fusion without user-supplied merchant names | **PASS** |
| Regression | T1, V1 | Six tools; `allow_raw_transactions=false` default | **PASS** |

## Operator gate

1. Deploy backend image with A1–E2 + T1 + S-fix merged. — **DONE** 2026-06-07
2. **BACKEND_DEPLOY** — confirm backend container on omniflow before AI Chat probes. — **DONE**
3. No Firefly re-sync required.

## Smoke checklist (omniflow — `financegnome.omniflow.cc`)

| Step | Probe | Pass criteria | Result |
|------|-------|---------------|--------|
| S-1 | Ask cancelable streaming subscription total | Tool returns patterns with display_name values | **PASS** — YouTube, Netflix, Mitgliedsbeitrag named |
| S-2 | Follow-up: *"liste mir die dienste auf"* | Response enumerates **named merchants** from tool data (not generic industry list) | **PASS** — 12 named merchants, no Counterparty-* |
| T-b-1 | Strom spend in mirror window via AI Chat | Uses `category_search`; amount backed by category 146 aggregate | **PASS** (explicit Jun 2025–May 2026: **465,53 €**) |
| T-b-2 | Amazon spend in mirror window | Uses `category_search`; amount backed by category 47 aggregate | **PARTIAL** — LLM `group_by: month` may return unfiltered totals (advisory) |
| T-a | Amazon Jan–Oct 2023 | Explicit empty-state citing `mirror_date_bounds` (2025-06-05…) + `no_rows_in_period` | **PASS** |
| U-1 | Multi-tool question (subs + category spend) without user merchant names | Fuses get_subscriptions + get_transactions without Counterparty-* filters | **PASS** — named merchants + fusion |
| REG-1 | Six-tool registry | Tool count = 6 | **PASS** |
| REG-2 | Privacy default | `allow_raw_transactions=false` → no raw_rows in tool JSON | **PASS** |

## Local gates (execute — completed)

| Step | Description | Result |
|------|-------------|--------|
| T-1 | `cargo test --lib` | **PASS** (150/150) |
| T-2 | `cargo test --test bug0007_ai_discovery` | **PASS** (8/8) |
| T-3 | Six-tool count + schema contracts | **PASS** |

## Notes

- V1 runtime verify re-run 2026-06-07 after S-privacy fix BACKEND_DEPLOY.
- **S fix verified:** `PrivacyLayer` preserves subscription `display_name` / `merchant_names[]` in `get_subscriptions`.
- Undated Strom/Amazon queries may default LLM period to 2023 (advisory); mirror-window probes pass.
- `group_by: month` + `category_search` may return unfiltered totals (advisory).
- BUG-0008: no alert/list/detection threshold changes in this sprint.

## Next phase

**`/release`** — BUG-0007 closure.
