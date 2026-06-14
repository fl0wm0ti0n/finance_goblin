# Resume Brief — S0021 / US-0022

**Sprint:** S0021
**Story:** US-0022 — Deploy version stamp & stale-frontend detection
**Release version:** `0.22.0-us0022`
**Date:** 2026-06-14
**Orchestrator:** `auto-20260613-bug0025`

---

## Release outcome

**RELEASE PASS** — All gates satisfied; backlog DONE; acceptance AC-1..AC-6 checked.

| Gate | Result |
|------|--------|
| Check-in test | PASS (cargo lib 221/221; meta_test 3/3; npm 31/31; build PASS) |
| QA completion | PASS (0 blockers) |
| UAT / verify-work | PASS-WITH-PREREQUISITES (6 pass / 2 pass-with-prerequisites / 0 fail) |
| Isolation compliance | PASS |
| Strict runtime proof | PASS |
| Legacy drift | PASS |
| README feature coverage | PASS (advisory) |
| Project README coverage | PASS |
| User guide | PASS |
| Release finalization | PASS |

---

## Acceptance criteria

| Row | Result | Notes |
|-----|--------|-------|
| AC-1 | PASS | Sidebar-footer subtle stamp |
| AC-2 | PASS | Hover tooltip with release/build/timestamp |
| AC-3 | PASS | Backend metadata endpoint (3/3 tests) |
| AC-4 | PASS | SPA compile-time embed (Vite define) |
| AC-5 | PASS-WITH-PREREQUISITES | Code PASS; live browser deferred BACKEND_FRONTEND_DEPLOY |
| AC-6 | PASS-WITH-PREREQUISITES | Code+test PASS; OIDC smoke deferred BACKEND_FRONTEND_DEPLOY |

---

## Artifacts

- `handoffs/releases/S0021-release-notes.md` (canonical release notes)
- `sprints/S0021/release-findings.md` (release findings)
- `sprints/S0021/qa-findings.md` (QA findings)
- `sprints/S0021/uat.json` (UAT results)
- `sprints/S0021/uat.md` (UAT summary)
- `sprints/S0021/verify-work-findings.md` (verify-work findings)
- `sprints/S0021/summary.md` (execute summary)
- `docs/user-guides/US-0022.md` (operator user guide)

---

## Operator gates pending

- **BACKEND_FRONTEND_DEPLOY** — live AC-5 stale-detection browser smoke + AC-6 OIDC external profile smoke

---

## Advisories (non-blocking)

1. Tooltip timestamp uses client-side `Date.now()`, not build timestamp
2. StaleBanner not dismissible (reload only)
3. Spec-pack advisory (no artifact; architecture § US-0022 documents gates inline)

---

## Next phase

**refresh-context** — in new subagent/chat.

---

## Queue linkage

- `handoffs/release_queue.md` — S0021 `status=released`
- `release_notes_ref`: `handoffs/releases/S0021-release-notes.md`
- `release_version`: `0.22.0-us0022`

---

## Isolation evidence

- `phase_id`: release
- `role`: release
- `fresh_context_marker`: release-20260614-us0022-release-fresh
- `timestamp`: 2026-06-14T19:23:00Z
- `runtime_proof_id`: runtime-proof-release-20260614-us0022-001
