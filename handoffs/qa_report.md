# QA Report — Q0017 / BUG-0007 (S-privacy re-run)

**From:** QA  
**To:** Verify-work (`/verify-work`)  
**Date:** 2026-06-07  
**Bug:** BUG-0007  
**Sprint:** Q0017  
**Orchestrator:** `auto-20260607-bug0007-001`  
**Verdict:** **PASS**

## Summary

Re-ran QA after S privacy fix (`display_name` / `merchant_names[]` exempt from counterparty hashing in `get_subscriptions`). DEC-0069 implementation **PASS** on static review, `cargo test --lib` (150/150 — includes 2 new privacy tests), and `bug0007_ai_discovery` (8/8). V1 omniflow runtime smoke **DEFERRED** until operator **BACKEND_DEPLOY**.

## Test results

| Suite | Command | Result |
|-------|---------|--------|
| Regression lib | `cargo test --lib` | **PASS** (150/150) |
| S privacy unit | `get_subscriptions_preserves_*`, `get_subscriptions_still_redacts_*` | **PASS** |
| T1 BUG-0007 contract | `cargo test --test bug0007_ai_discovery` | **PASS** (8/8) |
| V1 runtime | Omniflow AI Chat smoke per `uat.md` | **DEFERRED** — BACKEND_DEPLOY |

### Test output — cargo test --lib (PASS)

```
running 150 tests
...
test ai::privacy::tests::get_subscriptions_preserves_display_name_and_merchant_names ... ok
test ai::privacy::tests::get_subscriptions_still_redacts_other_long_strings ... ok
...
test result: ok. 150 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.23s
```

### Test output — bug0007_ai_discovery (PASS)

```
running 8 tests
test get_transactions_schema_has_category_search ... ok
test get_subscriptions_schema_has_kind_enum ... ok
test category_search_cap_ten_with_truncation_flag ... ok
test category_search_ilike_resolves_strom_and_amazon ... ok
test mirror_date_bounds_from_transactions ... ok
test category_search_short_keyword_rejected ... ok
test aggregates_include_discovery_fields ... ok
test six_tool_registry_unchanged ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## Acceptance snapshot

| Row | Code QA | Runtime (V1) |
|-----|---------|--------------|
| **(S)** | **PASS** — privacy exemption preserves merchant labels; other strings still hashed | **PENDING** deploy |
| **(T)** | **PASS** — A1+A2+E1+E2 static; ILIKE/cap/bounds/short-keyword tests | **PENDING** deploy |
| **(U)** | **PASS** — A′+F+E fusion path static | **PENDING** deploy |
| Regression | **PASS** — `cargo test --lib` 150/150 | — |

## S privacy fix verified

1. `get_subscriptions_preserves_display_name_and_merchant_names` — labels preserved through `PrivacyLayer`.
2. `get_subscriptions_still_redacts_other_long_strings` — non-label strings still hashed.
3. Transaction payee/description redaction unchanged (counterparty hash elsewhere).

## Artifacts

- Full findings: `sprints/quick/Q0017/qa-findings.md`
- Dev handoff: `handoffs/dev_to_qa.md`
- UAT checklist: `sprints/quick/Q0017/uat.md`
- State checkpoint: `docs/engineering/state.md` (qa S-privacy re-run)

## Next phase

Operator **BACKEND_DEPLOY** → **`/verify-work`** for V1 omniflow AI Chat smoke (S-1/S-2 re-run).

---

**Stop here.** Continue in a **new** subagent/chat with **`/verify-work`**.
