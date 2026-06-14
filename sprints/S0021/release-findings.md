# Release Findings — S0021 / US-0022

**Phase:** release
**Role:** release
**Date:** 2026-06-14
**Orchestrator:** `auto-20260613-bug0025`
**Verdict:** **PASS**

## Gate results

| Gate | Result | Evidence |
|------|--------|----------|
| 1. Check-in test | **PASS** | cargo lib 221/221; meta_test 3/3; npm 31/31; build PASS |
| 2. QA completion | **PASS** | sprints/S0021/qa-findings.md (0 blockers) |
| 3. UAT / verify-work | **PASS-WITH-PREREQUISITES** | sprints/S0021/uat.json (6 pass / 2 pass-with-prerequisites / 0 fail) |
| 4. Isolation compliance | **PASS** | execute, qa, verify-work checkpoints in state.md |
| 4b. Strict runtime proof | **PASS** | runtime-proof-verify-work-20260614-us0022-001 |
| 3e. Legacy drift | **PASS** | No drift detected for US-0022 |
| 3f. README feature coverage | **PASS (advisory)** | coverage_missing: []; framework template parity advisory (non-blocking) |
| 3g. Project README coverage | **PASS** | validate_project_readme_coverage.py exit 0 |
| User guide (US-0032) | **PASS** | docs/user-guides/US-0022.md present |
| Spec-pack (US-0031) | **ADVISORY** | No spec-pack artifact; architecture § US-0022 documents gates inline (no new DEC) |
| Release finalization | **PASS** | All gates satisfied |

## Release version

- **release_version:** `0.22.0-us0022`
- **sprint_id:** S0021
- **story_id:** US-0022

## Acceptance criteria results

| Row | Result | Notes |
|-----|--------|-------|
| AC-1 | PASS | Sidebar-footer subtle stamp |
| AC-2 | PASS | Hover tooltip with release/build/timestamp |
| AC-3 | PASS | Backend metadata endpoint (3/3 tests) |
| AC-4 | PASS | SPA compile-time embed (Vite define) |
| AC-5 | PASS-WITH-PREREQUISITES | Code PASS; live browser deferred BACKEND_FRONTEND_DEPLOY |
| AC-6 | PASS-WITH-PREREQUISITES | Code+test PASS; OIDC smoke deferred BACKEND_FRONTEND_DEPLOY |

## Advisories (non-blocking)

1. **Tooltip timestamp client-side:** `AppLayout.tsx` L101 uses `new Date().toISOString()` (client-side), not `build_timestamp` from backend. Release tag + build id sufficient for deploy verification.
2. **StaleBanner not dismissible:** Reload button only; no close/dismiss. Reload solves stale state.
3. **Spec-pack advisory:** US-0022 has no spec-pack artifact; architecture § US-0022 documents five gates inline. No new DEC required. Non-blocking for release.
4. **Framework README parity:** `validate_readme_feature_coverage.py` reports template/ directory parity failures (missing its_magic/README.md etc.). No story coverage gaps (`coverage_missing: []`). Pre-existing framework infrastructure issue.

## Operator gates pending

- **BACKEND_FRONTEND_DEPLOY** — live AC-5 stale-detection browser smoke + AC-6 OIDC external profile smoke

## Blockers

**0 blockers.**

## Release outcome

**PASS** — US-0022 / S0021 released as `0.22.0-us0022`. All mandatory gates satisfied. Acceptance AC-1..AC-6 verified at code+test level. Live browser/API smoke deferred pending operator BACKEND_FRONTEND_DEPLOY.

## Evidence refs

- `handoffs/releases/S0021-release-notes.md`
- `sprints/S0021/qa-findings.md`
- `sprints/S0021/uat.json`
- `sprints/S0021/uat.md`
- `sprints/S0021/verify-work-findings.md`
- `docs/user-guides/US-0022.md`
