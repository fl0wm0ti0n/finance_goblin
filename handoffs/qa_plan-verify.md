# QA plan-verify handoff — BUG-0009 / Q0016

**From:** QA (`/plan-verify`)  
**To:** Dev (`/execute`)  
**Date:** 2026-06-06  
**Bug:** BUG-0009  
**Quick task:** Q0016  
**Orchestrator:** `auto-20260606-bug0009-001`  
**Verdict:** **PASS**

## Summary

Validated `sprints/quick/Q0016/tasks.md` and `task.json` against `docs/product/acceptance.md` BUG-0009 rows **(Y)**, **(Z)** and `docs/engineering/architecture.md` § BUG-0009 / **DEC-0068**. All six tasks (Z1, Z2, Y1, Y2, T1, V1) map to acceptance with no gaps and no orphans. Execute order Z1→Z2→Y1→Y2→T1 is acyclic; V1 gated on deploy + operator **GRAFANA_PROVISIONING_RELOAD** before omniflow smoke.

## Evidence

- `sprints/quick/Q0016/plan-verify.json`
- `sprints/quick/Q0016/plan-verify.md`
- `handoffs/plan_verify_report.md`
- `sprints/quick/Q0016/tasks.md`
- `handoffs/tl_to_dev.md` (`sprint-plan-20260606-q0016-bug0009`)

## Execute instructions

1. Implement **Z1 → Z2 → Y1 → Y2 → T1** in single provisioning PR.
2. **Z1:** Fix portfolio panel id 5 — latest-snapshot subquery + `CROSS JOIN LATERAL jsonb_array_elements`; remove global `LIMIT 1`.
3. **Z2:** Stat row visibility + retitle/reposition all-accounts overview table on portfolio dashboard only.
4. **Y1:** Update `$account_id` query in `cashflow.json` + `forecast-horizons.json` — `ORDER BY ABS(COALESCE(balance,0)) DESC, name`; omit `current`.
5. **Y2:** ML text banner + `noValue: "ML unavailable"` on ML panels in `forecast-horizons.json`; align with DEC-0066 copy.
6. **T1:** SQL fixtures — 3-row breakdown; ABS sort picks funded acct over zero wallet; optional provisioning JSON snapshot.
7. Deploy image to omniflow; operator **Grafana provisioning reload** (container restart or poll).
8. **V1** verify-work: create `uat.md`; omniflow probes rows Y/Z on `financegnome.omniflow.cc`; six `/analytics/{slug}` routes; ds/query 200 regression.
9. Do **not** merge US-0013 ML enablement; no backend/React code; no seventh dashboard; no dynamic hide rules.

## Advisories

- ADV-1: OIDC + bundled-firefly regression footer — operator verify-work post-V1.
- ADV-2: Row **(Y)** default-load probes blocked until deploy + **GRAFANA_PROVISIONING_RELOAD**.
- ADV-3: Subscriptions/budgets lack `$account_id` — V1 ds/query 200 regression sufficient; non-empty not gated on Y1.
- ADV-4: All-zero balances → alphabetical default — documented edge case.
- ADV-5: Overview portfolio-only — Z3 `/wealth` docs supplementary.
- ADV-6: ML charts empty below banner until US-0013 — Y2 closes honest empty-state only.
- ADV-7: Runbook warning — do not save variables in Grafana UI (bakes `current`).

## Next phase

`/execute` in fresh subagent context. No `handoffs/qa_to_dev.md` fix list — plan approved as-is.

---

# QA plan-verify handoff — BUG-0012 / Q0014

**From:** QA (`/plan-verify`)  
**To:** Dev (`/execute`)  
**Date:** 2026-06-06  
**Bug:** BUG-0012  
**Quick task:** Q0014  
**Orchestrator:** `auto-20260605-bug0012-001`  
**Verdict:** **PASS**

## Summary

Validated `sprints/quick/Q0014/tasks.md` and `task.json` against `docs/product/acceptance.md` BUG-0012 rows **(AG)**, **(AH)** and `docs/engineering/architecture.md` § BUG-0012. All five tasks (AH1, AG1, T1, D1, V1) map to acceptance with no gaps and no orphans. Execute order AH1→AG1→T1→D1 is acyclic; V1 gated on deploy + operator **manual Full Firefly sync + recompute** + conditional TOML extend. **DEC-0067** governs component-level monthly bucket attribution.

## Evidence

- `sprints/quick/Q0014/plan-verify.json`
- `sprints/quick/Q0014/plan-verify.md`
- `sprints/quick/Q0014/tasks.md`
- `handoffs/tl_to_dev.md` (`architecture-20260605-bug0012`, `sprint-plan-20260605-q0014-bug0012`)

## Execute instructions

1. Implement **AH1 → AG1 → T1 → D1** in single backend PR.
2. **AH1:** Add `category_id: Option<String>` to `RecurringPattern`; mode carry from `RecurrenceGroup.category_ids`; subscription override inherit/lookup.
3. **AG1:** Replace net-delta `categorize_delta` with per-component `monthly_map`; rolling → Variable; recurring due → `resolve_bucket` via category_id per DEC-0067.
4. **T1:** Unit tests — salary+rent scenario, same-day mixed, Variable regression, map_category wiring, subscription override carry.
5. **D1:** Retire `categorize_delta` for monthly bucket path; wire `category_names` through projection (no dead binding).
6. Deploy image to omniflow; operator **manual Full Firefly sync + forecast recompute**.
7. Operator extend `[forecast.category_buckets]` in TOML if German/custom labels miss default keys.
8. **V1** verify-work: create `uat.md` + runbook checklist; omniflow probes rows AG/AH on `financegnome.omniflow.cc`.
9. Do **not** merge US-0015 AI buckets or US-0013 ML overlay; no frontend changes; no default.toml code expansion.

## Advisories

- ADV-1: OIDC + bundled-firefly regression footer — operator verify-work post-V1.
- ADV-2: Rows **(AG)(AH)** require Full Firefly sync + recompute after deploy before V1 probes.
- ADV-3: Sprint D1 adds code cleanup beyond architecture runbook-only D1 — aligned enhancement.
- ADV-4: Conditional TOML extend for Gehalt/Miete Nebenkosten etc. before V1 pass.
- ADV-5: Income via categorized recurring only — positive rolling stays Variable per DEC-0067.
- ADV-6: Variable shrink when fixed moves out is intended — T1 regression required.
- ADV-7: US-0015 / US-0013 explicitly out of scope.

## Next phase

`/execute` in fresh subagent context. No `handoffs/qa_to_dev.md` fix list — plan approved as-is.

---

# QA plan-verify handoff — BUG-0010 / Q0013

**From:** QA (`/plan-verify`)  
**To:** Dev (`/execute`)  
**Date:** 2026-06-06  
**Bug:** BUG-0010  
**Quick task:** Q0013  
**Orchestrator:** `auto-20260605-bug0010-001`  
**Verdict:** **PASS**

## Summary

Validated `sprints/quick/Q0013/tasks.md` and `task.json` against `docs/product/acceptance.md` BUG-0010 rows **(AA)**, **(AB)**, **(AC)** and `docs/engineering/architecture.md` § BUG-0010. All seven tasks (AA1, AB1, AC1, AA3, AB2, AC2, V1) map to acceptance with no gaps and no orphans. Execute order AA1→AB1→AC1→AA3→AB2→AC2 is acyclic; V1 gated on deploy + operator **manual Full Firefly sync** backfill. **DEC-0065** governs AB1; **DEC-0066** governs AC1; **DEC-0060** extended by AA1.

## Evidence

- `sprints/quick/Q0013/plan-verify.json`
- `sprints/quick/Q0013/plan-verify.md`
- `sprints/quick/Q0013/tasks.md`
- `handoffs/tl_to_dev.md` (`architecture-20260605-bug0010`, `sprint-plan-20260605-q0013-bug0010`)

## Execute instructions

1. Implement **AA1 → AB1 → AC1 → AA3 → AB2 → AC2** in single PR (backend + frontend).
2. **AA1:** Extend `sync_accounts` with `balance_ingest` structured diagnostics; trust `current_balance` via `parse_split_amount` (DEC-0060).
3. **AB1:** Remove `COALESCE(balance, 0) >= 0` filter; add `is_overdrawn` per DEC-0065; signed `firefly.subtotal_eur`.
4. **AC1:** When `!forecast_ml.enabled`, call `record_skip_on_baseline(Disabled)`; meta derive `sidecar_disabled` per DEC-0066.
5. **AA3:** Add `balance_warnings` to forecast meta when negative start + tx history; ForecastPage banner.
6. **AB2:** Zero-total empty-state callout with Full Firefly sync guidance on WealthPage.
7. **AC2:** Three-state ML UI copy — not enabled / skipped / available; remove false skip on null reason.
8. Deploy image to omniflow; operator **manual Full Firefly sync** (balance backfill + baseline/wealth recompute).
9. **V1** verify-work: omniflow probes rows AA/AB/AC on `financegnome.omniflow.cc`.
10. Do **not** merge with BUG-0009/0011; AC3 → US-0013 only; no tx-sum balance recompute; no ML default enable on external.

## Advisories

- ADV-1: OIDC + bundled-firefly regression footer — operator verify-work post-V1.
- ADV-2: Rows **(AA)(AB)** require Full Firefly sync after deploy before V1 probes.
- ADV-3: AA2/AB3 operator gates consolidated into V1 — AB3 via wealth/history probes.
- ADV-4: AC3 ML production deferred US-0013 — honest degraded messaging closes BUG-0010 AC.
- ADV-5: Legitimate Firefly overdraft shows signed truth + warnings.
- ADV-6: Parallel open bugs must not merge into Q0013 scope.

## Next phase

`/execute` in fresh subagent context. No `handoffs/qa_to_dev.md` fix list — plan approved as-is.

---

# QA plan-verify handoff — BUG-0005 / Q0012

**From:** QA (`/plan-verify`)  
**To:** Dev (`/execute`)  
**Date:** 2026-06-05  
**Bug:** BUG-0005  
**Quick task:** Q0012  
**Orchestrator:** `auto-20260605-bug0005-001`  
**Verdict:** **PASS**

## Summary

Validated `sprints/quick/Q0012/tasks.md` and `task.json` against `docs/product/acceptance.md` BUG-0005 rows **(M)**, **(N)**, **(O)** and `docs/engineering/architecture.md` § BUG-0005. All six tasks (N1, N3, M1, N2, N4, O1) map to acceptance with no gaps and no orphans. Execute order N1→N3→M1→N2→N4 is acyclic; O1 gated on deploy + operator **manual exchange sync** backfill. **DEC-0062** governs N1; **DEC-0063** governs N3; **DEC-0064** governs M1/N2 wealth accounting.

## Evidence

- `sprints/quick/Q0012/plan-verify.json`
- `sprints/quick/Q0012/plan-verify.md`
- `sprints/quick/Q0012/tasks.md`
- `handoffs/tl_to_dev.md` (`architecture-20260605-bug0005`, `sprint-plan-20260605-q0012-bug0005`)

## Execute instructions

1. Implement **N1 → N3 → M1 → N2 → N4** in single backend PR (config + connector).
2. **N1:** Add `bitunix_futures_sign` and `futures_signed_get`; `futures_base_url` default `https://fapi.bitunix.com` per DEC-0062.
3. **N3:** Implement `effective_enabled_futures()` per DEC-0063; expose effective flag in settings API; document `BITUNIX_ENABLED_FUTURES` in `.env.example`.
4. **M1:** When effective futures enabled, fetch `/api/v1/futures/account?marginCoin=USDT`; emit `product_type: "futures"` holdings.
5. **N2:** Implement `sync_positions` via `get_pending_positions`; emit `product_type: "linear"`, `market_value_usd: None` per DEC-0064.
6. **N4:** Dual-path `test_connection` — spot always; futures when effective enabled; partial OK messaging.
7. Deploy image to omniflow; operator **manual exchange sync** (holdings backfill via exchange upsert).
8. **O1** verify-work: omniflow probes rows M/N/O on `financegnome.omniflow.cc`.
9. Do **not** merge with BUG-0006 scope; no `sync_funding`; USDT marginCoin MVP only; no Binance/Bybit changes.

## Advisories

- ADV-1: OIDC + bundled-firefly regression footer — operator verify-work post-O1.
- ADV-2: Rows **(M)(O)** require exchange sync after deploy before O1 probes.
- ADV-3: Row **(O)** portfolio crypto totals — O1 probes wealth API; portfolio cascades from holdings per DEC-0064.
- ADV-4: Linear positions excluded from wealth subtotal by design — wallet futures row drives O subtotal.
- ADV-5: USDT marginCoin MVP — multi-coin deferred.
- ADV-6: `sync_funding` stub remains no-op — out of scope.
- ADV-7: BUG-0006 Q0010 parallel — do not merge.

## Next phase

`/execute` in fresh subagent context. No `handoffs/qa_to_dev.md` fix list — plan approved as-is.

---

# QA plan-verify handoff — BUG-0004 / Q0011

**From:** QA (`/plan-verify`)  
**To:** Dev (`/execute`)  
**Date:** 2026-06-05  
**Bug:** BUG-0004  
**Quick task:** Q0011  
**Orchestrator:** `auto-20260605-bug0004-001`  
**Verdict:** **PASS**

## Summary

Validated `sprints/quick/Q0011/tasks.md` and `task.json` against `docs/product/acceptance.md` BUG-0004 rows **(I)**, **(J)**, **(K)**, **(L)** and `docs/engineering/architecture.md` § BUG-0004. All seven tasks (I1, K1, L1, L2, J1, J2, L3) map to acceptance with no gaps and no orphans. Execute order I1→K1→L1→L2→J1→J2 is acyclic; L3 gated on deploy + operator **Full Firefly sync** backfill. **DEC-0060** governs L1; **DEC-0061** governs J1.

## Evidence

- `sprints/quick/Q0011/plan-verify.json`
- `sprints/quick/Q0011/plan-verify.md`
- `sprints/quick/Q0011/tasks.md`
- `handoffs/tl_to_dev.md` (`architecture-20260605-bug0004`, `sprint-plan-20260605-q0011-bug0004`)

## Execute instructions

1. Implement **I1 → K1 → L1 → L2 → J1 → J2** in single PR (backend + Grafana JSON + frontend).
2. **I1:** Call `finish_sync_run` when `RunMode::ExchangesOnly` completes; mirror Full error path; no double-finish Full runs.
3. **K1:** Parenthesize `ORDER BY … LIMIT 1` branches before `UNION ALL` in portfolio pie panel id 8.
4. **L1:** Use `parse_split_amount` for Firefly `current_balance` per DEC-0060.
5. **L2:** Change wealth filter to `COALESCE(balance, 0) >= 0`.
6. **J1:** Add `extract_payee_source` priority chain per DEC-0061 in `by_payee()`.
7. **J2:** Subscriptions empty-state threshold copy + pending-count banner.
8. Deploy image to omniflow; operator **manual Full Firefly sync** (account balance backfill via DEC-0002 upsert).
9. **L3** verify-work: omniflow probes rows I/J/K/L on `financegnome.omniflow.cc`.
10. Do **not** merge with BUG-0005/0006 scope; no SQL migration backfill; no stuck-row cleanup; no auto-confirm.

## Advisories

- ADV-1: OIDC + bundled-firefly regression footer — operator verify-work post-L3.
- ADV-2: Rows **(L)** require Full Firefly sync after deploy before L3 wealth/forecast probes.
- ADV-3: Historical stuck `sync_runs` — forward fix only (I1).
- ADV-4: `net_worth_snapshots` populate via cascade after L1 backfill — no explicit snapshot SQL probe in L3.
- ADV-5: Other Grafana dashboards with `account_id` variable rely on wealth data-path fix — portfolio pie explicitly probed.
- ADV-6: BUG-0006 Q0010 amount sign may improve J1 expense filter — coordinate, not blocking.
- ADV-7: Row **(J)** detection requires Full sync — J2 documents.

## Next phase

`/execute` in fresh subagent context. No `handoffs/qa_to_dev.md` fix list — plan approved as-is.

---

# QA plan-verify handoff — BUG-0006 / Q0010

**From:** QA (`/plan-verify`)  
**To:** Dev (`/execute`)  
**Date:** 2026-06-05  
**Bug:** BUG-0006  
**Quick task:** Q0010  
**Verdict:** **PASS**

## Summary

Validated `sprints/quick/Q0010/tasks.md` and `task.json` against `docs/product/acceptance.md` BUG-0006 rows **(P)**, **(Q)**, **(R)** and `docs/engineering/architecture.md` § BUG-0006. All five tasks (Q1, Q2, Q3, R1, P1) map to acceptance with no gaps and no orphans. Ingest chain Q1→Q2→Q3 precedes aggregate R1; P1 gated on deploy + manual Firefly sync backfill. **DEC-0059** governs Q3; extends DEC-0002, DEC-0032.

## Evidence

- `sprints/quick/Q0010/plan-verify.json`
- `sprints/quick/Q0010/plan-verify-findings.md`
- `sprints/quick/Q0010/tasks.md`
- `handoffs/tl_to_dev.md` (`architecture-20260605-bug0006`, `sprint-plan-20260605-q0010-bug0006`)

## Execute instructions

1. Implement **Q1 → Q2 → Q3 → R1** in single backend PR (ordered dependencies).
2. **Q1:** Extract `category_id` from first split; bind on `upsert_transaction`.
3. **Q2:** Parse ISO datetime → `NaiveDate` date component.
4. **Q3:** Normalize amount sign per DEC-0059 before upsert.
5. **R1:** Extend `TransactionAggregates` with totals + `period_status` + Uncategorized label; AI tool passthrough.
6. Deploy backend image to omniflow; operator **manual Firefly sync** (backfill ~922 rows).
7. **P1** verify-work: SQL probe + AI Chat spending question on `financegnome.omniflow.cc`.
8. Do **not** merge with BUG-0002–0005 scope; no PrivacyLayer change; no SQL migration backfill.

## Advisories

- ADV-1: OIDC + six-tool registry regression footer — operator verify-work.
- ADV-2: Row **(P)** requires operator sync after deploy before P1 smoke.
- ADV-3: First-split category MVP acceptable per architecture.
- ADV-4: BUG-0002/0003 parallel tracks — Q0010 frozen boundary forbids merge.
- ADV-5: Confirm six-tool registry count at verify-work.

## Next phase

`/execute` in fresh subagent context. No `handoffs/qa_to_dev.md` fix list — plan approved as-is.

---

# QA plan-verify handoff — BUG-0003 / Q0009

**From:** QA (`/plan-verify`)  
**To:** Dev (`/execute`)  
**Date:** 2026-06-05  
**Bug:** BUG-0003  
**Quick task:** Q0009  
**Verdict:** **PASS**

## Summary

Validated `sprints/quick/Q0009/tasks.md` and `task.json` against `docs/product/acceptance.md` BUG-0003 rows **(F)**, **(G)**, **(H)** and `docs/engineering/architecture.md` § BUG-0003. All four tasks (F1, F2, G1, G2 gated) map to acceptance with no gaps and no orphans. H1 = F1 verify; H2 deferred. Dependency graph is acyclic; deploy order F2∥G1 → deploy → F1 → smoke → G2-if-gate is feasible. **No new DEC** required.

## Evidence

- `sprints/quick/Q0009/plan-verify.json`
- `sprints/quick/Q0009/plan-verify.md`
- `sprints/quick/Q0009/tasks.md`
- `handoffs/tl_to_dev.md` (`architecture-20260605-bug0003`)

## Execute instructions

1. Implement **F2** and **G1** in parallel (single PR recommended).
2. **F2:** Omniflow block in `.env.example`; runbook § Omniflow mis-host symptom table; optional compose comment — no behavior change.
3. **G1:** `ExchangeService::new` registers binance/bybit/bitunix when `effective_enabled()` — parity with Q0008 E1 mirror.
4. Deploy code image to omniflow, then **F1** operator: `DATABASE_HOST=postgres`, recreate `flow-finance-ai` + `grafana`.
5. Smoke rows **(F)**/**(G)**/**(H)**; run **G2** only if bitunix test returns auth/URL failure (not `unknown exchange`).
6. Run `cargo test --lib` for exchanges/config paths; record omniflow smoke at verify-work.
7. Do **not** merge with BUG-0002 / Q0008 track.

## Advisories

- ADV-1: OIDC + bundled-firefly regression footer — operator verify-work.
- ADV-2: Rows **(F)**/**(H)** require F1 on live host before verify-work.
- ADV-3: G2 likely unnecessary if G1 fixes registry-only **400**.
- ADV-4: BUG-0002 PAT deploy is parallel, not a Q0009 blocker.
- ADV-5: Representative GET probes sufficient for row **(F)**.

## Next phase

`/execute` in fresh subagent context. No `handoffs/qa_to_dev.md` fix list — plan approved as-is.

---

# QA plan-verify handoff — BUG-0014 / Q0022

**From:** QA (`/plan-verify`)  
**To:** Dev (`/execute`)  
**Date:** 2026-06-10  
**Bug:** BUG-0014  
**Sprint:** Q0022  
**Orchestrator:** `auto-20260607-bug0014-001`  
**Verdict:** **PASS**

## Summary

Validated Q0022 sprint plan against acceptance AO–AT, architecture § BUG-0014, and DEC-0081/0082/0083. 6/6 rows covered; AP2/AR1 conditional gates clear; V1 e2e smoke; ops-only AO/AT/AP1 documented. 0 gaps.

## Evidence

- `sprints/quick/Q0022/plan-verify.json`
- `handoffs/plan_verify_to_execute.md`

## Next phase

`/execute` in fresh subagent context.
