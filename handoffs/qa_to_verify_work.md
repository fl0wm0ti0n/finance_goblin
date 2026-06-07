# QA → Verify-work handoff

**Bug:** BUG-0015  
**Quick task:** Q0023  
**QA verdict:** **PASS** (2026-06-07)  
**Orchestrator:** `auto-20260607-bug0015-001`  
**Decisions:** DEC-0084, DEC-0085, DEC-0086  
**Next phase:** `/verify-work`

## QA summary

- **Blocking findings:** 0
- **Tasks verified PASS:** AU1, AU2, AU3, AU4
- **Tasks deferred (expected):** V1 (operator gates)
- **Tests re-run:** card_billing 4/4, interval_matches 2/2, build_active_payee 1/1, frontend 6/6, cargo lib 187/187

## Acceptance row status (post-QA)

| Row | Code QA | Runtime (verify-work) |
|-----|---------|------------------------|
| **AU** | PASS — AU1 DEC-0084 + AU2/AU3 inheritance | Pending H2-1 SQL + AU-1/AU-2 omniflow probes |
| **AV** | PASS — AU3 skip+merge + AU4 stale map | Pending AV-1 duplicate pending probe post-Full sync |
| **AW** | PASS — AU3 merge suppresses `new_detection` | Pending AW-1 unread-count reconciliation |

## Operator prerequisites (required before live probes)

1. **BACKEND_FRONTEND_DEPLOY** — Q0023 AU1–AU4 backend bundle on financegnome.omniflow.cc
2. **POSTGRES_PERSISTENCE_PROBE** — H2 SQL on `subscription_patterns` after rebuild, **before** Full sync
3. **FULL_FIREFLY_SYNC** — Full sync (not exchanges-only) + subscription detection phase

## Verify-work instructions

1. Read `sprints/quick/Q0023/uat.md` and populate `uat.json` steps with live results
2. Establish AU baseline: confirm Cursor + Apple on `/subscriptions` before rebuild (AU-BASE)
3. Rebuild `flow-finance-ai` only — postgres volume untouched
4. Run H2 SQL probe; document outcome per uat.md decision table
5. After Full sync: AU-1 confirmed API, AU-2 UI, AV-1 no duplicate pending, AW-1 unread reconcile, OIDC-1 regression

## Artifacts

- `sprints/quick/Q0023/qa-findings.md`
- `sprints/quick/Q0023/uat.json` (qa phase populated)
- `handoffs/dev_to_qa.md`
- `decisions/DEC-0084.md`, `DEC-0085.md`, `DEC-0086.md`

No code rework required for QA PASS. V1 runtime smoke remains operator-gated at verify-work.
