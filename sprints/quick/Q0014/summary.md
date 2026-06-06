# Sprint Summary — Q0014 (BUG-0012)

**Bug:** BUG-0012  
**Sprint:** Q0014 (`/quick`) + follow-up **Q0015**  
**Execute date:** 2026-06-05  
**QA date:** 2026-06-06  
**Verify-work date:** 2026-06-06  
**Release date:** 2026-06-06  
**Status:** **RELEASED** — BUG-0012 DONE

## Tasks delivered

| ID | Status | Summary |
|----|--------|---------|
| AH1 | done | `RecurringPattern.category_id` from group mode; subscription override inherit/lookup |
| AG1 | done | Component-level `monthly_map` attribution (DEC-0067) |
| T1 | done | Unit tests salary/rent/mixed/Variable regression + category wiring |
| D1 | done | Removed net-delta `categorize_delta` from monthly bucket path |
| V1 | done | Runbook TOML checklist + verify-work omniflow smoke — **PASS** |
| Q0015 | done | Payee-key due; household income; standing-order scope; German buckets |

## Tests

| Check | Result |
|-------|--------|
| `cargo test --lib` (execute) | **PASS** (139 tests) |
| `cargo test --lib` (QA) | **PASS** (139/139) |
| `cargo test --lib` (verify-work + release) | **PASS** (142/142) |
| Frontend tests/build | **N/A** — backend-only fix |

## QA verdict

**PASS** — AH1–D1 validated against DEC-0067 and plan-verify.

## Verify-work verdict

**PASS** — omniflow account 114: Fixed **2073.85** (Jun), Income **3266.16** (Jul+).

## Release verdict

**PASS** — `handoffs/releases/Q0014-release-notes.md`; backlog DONE; acceptance AG/AH checked.

## Acceptance

| Row | Code | Runtime | Release |
|-----|------|---------|---------|
| **(AG)** | **PASS** | **PASS** | **DONE** |
| **(AH)** | **PASS** | **PASS** | **DONE** |

## Files changed (primary)

- `backend/src/forecast/types.rs` — `RecurringPattern.category_id`
- `backend/src/forecast/recurring.rs` — mode carry; payee-key due (Q0015)
- `backend/src/forecast/categories.rs` — `resolve_bucket`, `accumulate_bucket`
- `backend/src/forecast/project.rs` — per-component monthly attribution; household income (Q0015)
- `docs/engineering/runbook.md` — BUG-0012 §16 hotfix
- `sprints/quick/Q0014/release-findings.md` — release gate audit

## Evidence

- `handoffs/releases/Q0014-release-notes.md`
- `sprints/quick/Q0014/verify-work-findings.md`
- `sprints/quick/Q0015/summary.md`
- `decisions/DEC-0067.md`
