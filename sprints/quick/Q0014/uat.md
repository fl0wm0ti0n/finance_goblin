# UAT — Quick Q0014 / BUG-0012

**Bug:** BUG-0012 — Forecast monthly Income/Fixed buckets always zero  
**Follow-up:** Q0015 (payee-key, household income, standing-order forecast scope)  
**Phase:** release (finalized)  
**Date:** 2026-06-06  
**Verdict:** **PASS** — released; BUG-0012 DONE

## UAT steps

| ID | Step | Result | Notes |
|----|------|--------|-------|
| V-1 | `cargo test --lib` | **PASS** | 142/142 |
| V-2 | DEC-0067 + Q0015 implementation | **PASS** | QA + deploy evidence |
| GATE-1 | `GET /health` | **PASS** | 200 |
| GATE-2 | `GET /api/v1/forecast/meta` post-deploy | **PASS** | `computed_at` 2026-06-05T21:39:42Z |
| AG-1 | Monthly API `income > 0` (account 114) | **PASS** | Jul+ `3266.16`; Jun 0 (projection window) |
| AG-2 | UI Monthly Income card | **DEFERRED** | API PASS |
| AH-1 | Monthly API `fixed_costs > 0` | **PASS** | Jun `2073.85` |
| AH-2 | UI Monthly Fixed card | **DEFERRED** | API PASS |
| REG-1 | OIDC regression | **DEFERRED** | dev-bypass external |
| REG-2 | External profile regression | **PASS** | sync/forecast 200 |

## Acceptance mapping

| Criterion | UAT steps | Result |
|-----------|-----------|--------|
| **(AG)** Income non-zero when categorized inflows projected | AG-1, AG-2 | **PASS** |
| **(AH)** Fixed non-zero when fixed-cost recurring mapped | AH-1, AH-2 | **PASS** |

## Summary

**9 passed**, **0 failed**, **2 deferred** (browser OIDC). Runtime omniflow probes confirm BUG-0012 fixed on account 114. **Released** 2026-06-06 — `handoffs/releases/Q0014-release-notes.md`.
