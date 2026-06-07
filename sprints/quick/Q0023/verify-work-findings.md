# Verify-work Findings — Quick Q0023 / BUG-0015

**Work item:** BUG-0015 (defect)  
**Quick task:** Q0023  
**Phase:** `/verify-work`  
**Orchestrator:** `auto-20260607-bug0015-001`  
**Date:** 2026-06-07  
**Decisions:** DEC-0084, DEC-0085, DEC-0086  
**Verify-work agent:** fresh subagent (`verify-work-20260607-q0023-bug0015`)  
**Verdict:** **PASS** — rows **AU**, **AV**, **AW** satisfied at code/test level; V1 runtime probes pass-with-prerequisites per BUG-0013/0014 precedent; proceed to `/release`

## Summary

Verify-work populated UAT artifacts from QA PASS evidence (`sprints/quick/Q0023/qa-findings.md`, `handoffs/dev_to_qa.md`). Independent re-run confirms **187/187** lib tests and **6/6** frontend vitest. Acceptance rows **AU** (DEC-0084 payee_key + confirm inheritance), **AV** (merge/skip on payee+interval, no duplicate pending), and **AW** (merge suppresses `new_detection`) pass at code/test level. V1 omniflow rebuild smoke (confirm → rebuild → Full sync → no re-prompt for Cursor/Apple) recorded as **pass-with-prerequisites** pending operator gates. Zero blocking findings.

## Per-row verdict (acceptance AU / AV / AW)

| Row | Verdict | Summary |
|-----|---------|---------|
| **AU** | **pass** | AU1 `payee_key` card rules (CURSOR, APPLE.COM/BILL); AU2–AU3 confirm inheritance + merge. Live AU-BASE/AU-1/AU-2/H2-1 **pass-with-prerequisites**. |
| **AV** | **pass** | AU2 `load_confirmed_payee_intervals` + index; AU3 skip/merge before pending upsert; AU4 stale inactive by payee+interval. Live AV-1 **pass-with-prerequisites**. |
| **AW** | **pass** | AU3 merge path `continue` before `upsert_pending_pattern` / `new_detection` alert. Live AW-1 **pass-with-prerequisites**. |

## Operator gate

| Gate | Status |
|------|--------|
| Code (AU1–AU4) | **CLEARED** |
| `cargo test --lib` | **CLEARED** — 187/187 PASS |
| `npm test -- --run` | **CLEARED** — 6/6 PASS |
| Operator **BACKEND_FRONTEND_DEPLOY** | **PENDING** |
| Operator **POSTGRES_PERSISTENCE_PROBE** (H2 SQL) | **PENDING** |
| Operator **FULL_FIREFLY_SYNC** | **PENDING** |
| V1 omniflow smoke (AU–AW live) | **PENDING** — pass-with-prerequisites |
| OIDC regression (AW footnote) | **PENDING** — pass-with-prerequisites |

## Automated verification (verify-work independent re-run)

| Check | Result |
|-------|--------|
| `cd backend && cargo test --lib` | **PASS** (187/187) |
| AU1 — `card_billing_*` normalize tests | **PASS** (4/4) |
| AU2 — `interval_matches` + `find_confirmed_payee_interval` | **PASS** (3/3) |
| AU3 — merge path code review + detection tests | **PASS** |
| AU4 — `build_active_payee_intervals` + `mark_stale_inactive` wiring | **PASS** |
| `npm test -- --run` (frontend regression) | **PASS** (6/6) |
| Omniflow reachability | **pass_with_prerequisites** — root 401; `/api/health` and `/api/v1/health` 404; deploy pending |

### Test output (lib suite)

```
test result: ok. 187 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## Code contract verification

| Contract | Evidence |
|----------|----------|
| **DEC-0084** — asterisk split, comma left-segment, Apple roots → `apple`, `.com`/`/bill` tail strip | `normalize.rs` `apply_card_billing_rules`; tests `card_billing_*` |
| **DEC-0085** — `load_confirmed_payee_intervals`, `merge_confirmed_pattern` in-place refresh | `repository.rs` L112–220; migration `012_subscription_patterns_payee_status.sql` |
| **DEC-0086** — `interval_matches` ±3d; fingerprint rotation on merge; UNIQUE fail-safe | `repository.rs` `interval_matches`; merge UNIQUE → `Ok(false)` → pending path |
| **AU3** — merge before pending; no `new_detection` on confirmed merge | `detection.rs` L59–76 `continue` after merge |
| **AU4** — `mark_stale_inactive` uses `build_active_payee_intervals` | `service.rs` L55–57 |

## Acceptance impact

| Row | Verify-work | `acceptance.md` checkbox |
|-----|-------------|--------------------------|
| **AU** | **PASS** | Release phase |
| **AV** | **PASS** | Release phase |
| **AW** | **PASS** | Release phase |

## Release gate

| Gate | Status |
|------|--------|
| QA PASS (dev handoff) | yes |
| Verify-work PASS | **yes** |
| UAT populated (DEC-0009) | **yes** — `uat.json`, `uat.md` |
| Acceptance AU–AW | **PASS** (code) + runtime prerequisites documented |
| Isolation evidence (verify-work) | **yes** |
| Release proceed | **yes** |

## Operator advisory (non-blocking)

1. **BACKEND_FRONTEND_DEPLOY:** Deploy Q0023 backend (AU1–AU4) on `financegnome.omniflow.cc`.
2. **AU-BASE:** Confirm Cursor + Apple on `/subscriptions` before rebuild.
3. **Rebuild:** Rebuild `flow-finance-ai` only — postgres volume untouched.
4. **POSTGRES_PERSISTENCE_PROBE:** Run H2 SQL immediately after rebuild, **before** Full sync; document outcome per `uat.md` table.
5. **FULL_FIREFLY_SYNC:** Full sync (not exchanges-only) + subscription detection phase.
6. **Post-gate smoke:** Execute 10-step checklist in `sprints/quick/Q0023/uat.json` `operator_smoke_checklist`.
7. **Reopen criteria:** Zero `confirmed` rows after rebuild (no operator action) → ops volume/DB target issue; duplicate pending for same payee+interval → reopen execute.

## Artifacts

- `sprints/quick/Q0023/uat.json`
- `sprints/quick/Q0023/uat.md`
- `sprints/quick/Q0023/qa-findings.md`
- `handoffs/dev_to_qa.md`
- `handoffs/verify_work_to_release.md`

## Next steps

1. **`/release`** — check BUG-0015 acceptance AU–AW; publish release notes; update backlog → DONE

---

**Stop here.** Continue in a **new** subagent/chat with **`/release`**.
