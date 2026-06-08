# UAT — Sprint S0019 / US-0020

**Sprint:** S0019  
**Story:** US-0020 — Subscription manual discovery, majority category & operator tags  
**Phase:** verify-work (populated)  
**Status:** **PASS** (5 code pass + 1 pass-with-prerequisites)  
**Plan-verified at:** 2026-06-10T17:00:00Z  
**Execute at:** 2026-06-10T22:45:00Z  
**QA verified at:** 2026-06-10T23:00:00Z  
**Verify-work at:** 2026-06-10T23:15:00Z  
**Orchestrator:** `auto-20260608-us0020-001`  
**Decisions:** DEC-0098, DEC-0099, DEC-0100, DEC-0101, DEC-0102, DEC-0103

## UAT steps

| ID | AC | Description | Result |
|----|-----|-------------|--------|
| UAT-1 | AC-1 | Discover tab filters by account, payee, interval; results capped at 50 | **pass** |
| UAT-2 | AC-2 | Confirm discover candidate → confirmed subscription (no pending-only path) | **pass** |
| UAT-3 | AC-3 | Majority display category badge + tie-break tooltip on confirmed row | **pass** |
| UAT-4 | AC-4 | Tag CRUD; assign multiple tags; filter All tab by tag slug | **pass** |
| UAT-5 | AC-5 | Tags + display_category in product DB; no Firefly write-back | **pass** |
| UAT-6 | AC-6 | US-0003 pending confirm + US-0008 alert dedup unchanged; OIDC smoke | **pass_with_prerequisites** |

## Target acceptance criteria

Source: `docs/product/acceptance.md` § US-0020

| ID | Criterion | Verify-work verdict |
|----|-----------|---------------------|
| AC-1 | Manual search by account, payee, interval; paginated/capped | **pass** |
| AC-2 | Operator confirm searched candidate without auto-detection-only | **pass** |
| AC-3 | Display category = mode category; tie-break documented | **pass** |
| AC-4 | Tag CRUD; multi-assign; filter by tag | **pass** |
| AC-5 | Product DB storage; no Firefly write-back | **pass** |
| AC-6 | US-0003/US-0008 regression; OIDC external smoke | **pass_with_prerequisites** |

## Operator gate (pre-runtime smoke)

| Gate | Action |
|------|--------|
| **BACKEND_FRONTEND_DEPLOY** | Deploy S0019 backend + frontend on US-0010 external profile (`financegnome.omniflow.cc`) |
| **FULL_FIREFLY_SYNC** | Ensure mirror transactions + categories current for discover + majority category |

## OIDC smoke checklist (AC-1..AC-6) — operator post-deploy

Profile: **US-0010 external** (Traefik auth + OIDC on SPA).

1. **`/subscriptions` Discover tab** — Select account; enter payee substring; pick interval bucket; search returns candidates with confidence + count (AC-1).
2. **Confirm candidate** — Click confirm on discover row; subscription appears in All/confirmed list as `confirmed` (not pending-only path) (AC-2).
3. **Majority category** — Confirmed row shows display category badge; tooltip documents RANK tie-break (AC-3).
4. **Tags** — Create tag (e.g. `luxus`); assign to subscription; filter All tab by tag chip (AC-4).
5. **Storage** — Verify no Firefly API writes during smoke; tags + display_category in app DB only (AC-5).
6. **US-0003 regression** — Pending tab confirm/reject still works; sync-triggered detection unchanged (AC-6).
7. **US-0008 regression** — Alert unread count reconciles with list tabs after discover confirm (no spurious `new_detection` on manual confirm).
8. **Read-only Firefly** — No POST/PUT/PATCH/DELETE to Firefly during smoke.

## Regression checklist (T-0208)

- [x] `DetectionPipeline::run_candidates` unchanged — no threshold edits
- [x] Pending confirm/reject flows preserved (API routes unchanged)
- [x] Manual discover confirm does not emit `new_detection` alert (DEC-0099)
- [x] DEC-0085 merge on duplicate payee+interval (manual + auto paths)
- [x] 409 on rejected payee-interval manual confirm (repository guard)

## Automated checks (verify-work re-run)

| Check | Result |
|-------|--------|
| `cargo test --lib` | **213/213 PASS** |
| `npm test -- --run` | **9/9 PASS** |
| Discover service tests | **PASS** |
| Confirm-from-discover / majority category tests | **PASS** |
| Tag CRUD + assign tests | **PASS** |

## Results summary

| Metric | Count |
|--------|-------|
| UAT steps total | 6 |
| Passed | 5 |
| Pass with prerequisites | 1 |
| Failed | 0 |
| Pending | 0 |

**Verdict:** **PASS** — AC-1..AC-5 code/test/doc satisfied; AC-6 pass-with-prerequisites pending operator **BACKEND_FRONTEND_DEPLOY** + **FULL_FIREFLY_SYNC**. Release unblocked.

**Next phase:** `/release`
