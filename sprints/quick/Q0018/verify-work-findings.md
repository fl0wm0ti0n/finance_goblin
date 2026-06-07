# Verify-work Findings — Quick Q0018 / BUG-0008

**Work item:** BUG-0008 (defect)  
**Quick task:** Q0018  
**Phase:** `/verify-work`  
**Orchestrator:** `auto-20260608-bug0008-001`  
**Date:** 2026-06-08  
**Decisions:** DEC-0071, DEC-0072  
**Verify-work agent:** fresh subagent (`verify-work-20260608-q0018-bug0008`)  
**Verdict:** **PASS** — rows **W** and **X** satisfied; proceed to `/release`

## Summary

Verify-work populated UAT artifacts from QA PASS code/test evidence. Independent re-run confirms **8/8** BUG-0008 contract tests and **156/156** lib tests. Acceptance rows **W** (alert reconciliation) and **X** (detection recall without spam) pass at code/test level. V1 omniflow runtime probes (W-1–W-4, X-1–X-2, REG-1–REG-2) recorded as **pass-with-prerequisites** pending operator **BACKEND_FRONTEND_DEPLOY** per US-0010 precedent. Zero blocking findings.

## Per-row verdict (acceptance W / X)

| Row | Verdict | Summary |
|-----|---------|---------|
| **W** | **PASS** | Fingerprint migration + `upsert_alert` ON CONFLICT dedup; emit gate on new pending/tier increase only; `GET /api/v1/subscriptions/alerts/unread-count` with `reconciled: true` and orphan JOIN guard; orphan mark-read on confirm/reject/inactive; frontend banner/toast from `unread_new_detection`. Tests: `upsert_alert_dedupes_unread_fingerprints`, `unchanged_resync_does_not_spam_alerts`, `unread_count_api_reconciled_semantics`, `confirm_marks_read_orphan_alerts`. Live W-1–W-4 **pass-with-prerequisites**. |
| **X** | **PASS** | SEPA + legal suffix normalization; transfer counterparty priority guard; `detection_window_days = 730`; forecast regression intact; spam invariant via emit gate + fingerprint dedup. Tests: `sepa_transfer_fixtures_merge_under_single_payee_key`, `detection_window_defaults_to_730_days`, `forecast_recurring_still_detects_after_normalization`. Live X-1/X-2 **pass-with-prerequisites**. |
| Regression | **pass-with-prerequisites** | REG-1 OIDC + REG-2 bundled-firefly deferred post-deploy (non-blocking per prior bug releases) |

## Operator gate

| Gate | Status |
|------|--------|
| Code QA (W1–W7, X1–X4) | **CLEARED** |
| `cargo test --test bug0008_subscription_alerts` | **CLEARED** — 8/8 PASS |
| `cargo test --lib` | **CLEARED** — 156/156 PASS |
| Operator **BACKEND_FRONTEND_DEPLOY** | **PENDING** — runtime probes pass-with-prerequisites |
| V1 omniflow smoke (W/X rows) | **PENDING** — operator post-deploy |

## Automated verification (verify-work independent re-run)

| Check | Result |
|-------|--------|
| `cd backend && cargo test --test bug0008_subscription_alerts` | **PASS** (8/8) |
| `cd backend && cargo test --lib` | **PASS** (156/156) |
| W code paths (W1–W5, W7) | **PASS** — per qa-findings T-5–T-9 |
| X code paths (X1–X4) | **PASS** — per qa-findings T-10–T-13 |
| W6 frontend wiring | **PASS** — SubscriptionsPage.tsx + api.ts |
| `scripts/check-user-visible-metadata.py` | **SKIP** — entrypoint absent (same as execute/qa) |

### Test output (contract suite)

```
running 8 tests
test subscription_detection_persists_pending_pattern ... ok
test forecast_recurring_still_detects_after_normalization ... ok
test confirm_marks_read_orphan_alerts ... ok
test unchanged_resync_does_not_spam_alerts ... ok
test detection_window_defaults_to_730_days ... ok
test unread_count_api_reconciled_semantics ... ok
test upsert_alert_dedupes_unread_fingerprints ... ok
test sepa_transfer_fixtures_merge_under_single_payee_key ... ok

test result: ok. 8 passed; 0 failed
```

## Acceptance impact

| Row | Verify-work | `acceptance.md` checkbox |
|-----|-------------|--------------------------|
| **W** | **PASS** | Release phase |
| **X** | **PASS** | Release phase |
| Regression | **pass-with-prerequisites** | N/A (footer) |

## Release gate

| Gate | Status |
|------|--------|
| QA PASS | yes |
| Verify-work PASS | **yes** |
| UAT populated (DEC-0009) | **yes** — `uat.json`, `uat.md` |
| Acceptance W/X | **PASS** (code) + runtime prerequisites documented |
| Release proceed | **yes** |

## Operator advisory (non-blocking)

1. **BACKEND_FRONTEND_DEPLOY:** Deploy backend + frontend with migration `010_subscription_alert_fingerprint.sql` before live W/X probes.
2. **Post-deploy smoke:** Execute W-1–W-4, X-1–X-2 checklist in `sprints/quick/Q0018/uat.md` on `https://financegnome.omniflow.cc`.
3. **Optional sync:** Full Firefly sync recommended to validate X-1 pattern count > 12 baseline.
4. **OIDC browser smoke:** Deferred on external dev-bypass profile (non-blocking).

## Artifacts

- `sprints/quick/Q0018/uat.json`
- `sprints/quick/Q0018/uat.md`
- `sprints/quick/Q0018/qa-findings.md`
- `handoffs/verify_work_to_release.md`

## Next steps

1. **`/release`** — check BUG-0008 acceptance; publish release notes; update backlog → DONE

---

**Stop here.** Continue in a **new** subagent/chat with **`/release`**.
