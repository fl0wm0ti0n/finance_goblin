# QA Findings — Quick Q0012 / BUG-0005

**Work item:** BUG-0005 (defect)  
**Quick task:** Q0012  
**QA phase:** `/qa`  
**Date:** 2026-06-05  
**Verdict:** **PASS** (ready for deploy → operator exchange sync → `/verify-work` O1)

## Scope

Bitunix futures multi-product connector per `architecture.md` § BUG-0005 and **DEC-0062**, **DEC-0063**, **DEC-0064**:

- **N1** — Futures header-auth client + `futures_base_url` (DEC-0062)
- **N3** — `effective_enabled_futures()` policy + settings exposure (DEC-0063)
- **M1** — Futures wallet ingestion (`product_type: futures`)
- **N2** — `sync_positions` via `get_pending_positions` (`product_type: linear`, DEC-0064)
- **N4** — Dual-path `test_connection` (spot + futures sub-status)
- **O1** — Omniflow runtime probes (M/N/O) — **deferred** to verify-work

**Inputs reviewed:** `handoffs/dev_to_qa.md`, `sprints/quick/Q0012/summary.md`, `sprints/quick/Q0012/plan-verify.json`, `sprints/quick/Q0012/plan-verify.md`, `sprints/quick/Q0012/tasks.md`, `docs/product/acceptance.md` (BUG-0005 rows M/N/O), `docs/engineering/architecture.md` (§ BUG-0005), `decisions/DEC-0062.md`, `decisions/DEC-0063.md`, `decisions/DEC-0064.md`, `backend/src/exchanges/http.rs`, `backend/src/exchanges/bitunix.rs`, `backend/src/config/mod.rs`, `backend/config/default.toml`, `.env.example`, `sprints/quick/Q0012/uat.md`. No host `.env`, `.env_prod`, or operator secret values read.

## Test plan

| # | Test | Method | Result |
|---|------|--------|--------|
| T-1 | Backend unit tests | `cd backend && cargo test --lib` | **PASS** (123/123) |
| T-2 | Frontend unit tests | `cd frontend && npm test` | **PASS** (2/2) |
| T-3 | Frontend production build | `cd frontend && npm run build` | **PASS** |
| T-4 | N1 futures sign fixture (R-0058) | `http::tests::bitunix_futures_sign_matches_official_fixture` | **PASS** |
| T-5 | N1 spot sign regression | `http::tests::bitunix_spot_sign_unchanged_regression` | **PASS** |
| T-6 | N1 wiremock header-auth | `bitunix::tests::futures_signed_get_sends_header_auth` | **PASS** |
| T-7 | N3 effective_enabled_futures matrix | `config::tests` (auto-enable, env opt-out, settings view) | **PASS** |
| T-8 | M1 futures wallet parse | `bitunix::tests::sync_balances_futures_wallet_product_type` | **PASS** |
| T-9 | M1 spot-only regression | `bitunix::tests::sync_balances_spot_only_when_futures_disabled` | **PASS** |
| T-10 | N2 linear positions (DEC-0064) | `bitunix::tests::sync_positions_linear_holdings` | **PASS** |
| T-11 | N2 futures-disabled no-op | `bitunix::tests::sync_positions_empty_when_futures_disabled` | **PASS** |
| T-12 | N4 dual test_connection | `test_connection_spot_ok_futures_fail_partial`, `_spot_fail`, `_spot_only_when_futures_disabled` | **PASS** |
| T-13 | Frozen boundaries | No Binance/Bybit changes; no `sync_funding` impl; GET-only unchanged | **PASS** |
| T-14 | Rows M/N/O live smoke | Omniflow deploy + exchange sync + O1 probes | **DEFERRED** — verify-work |

### Environment dependencies (non-blocking)

- **Operator deploy:** Q0012 backend image to omniflow before O1.
- **Exchange sync gate:** Manual exchange sync required before holdings/wealth probes per architecture operator gate (not Full Firefly sync).
- **Live Bitunix credentials:** Required for O1 runtime validation on operator profile.

## Acceptance criteria matrix (BUG-0005)

| Row | Criterion | Verdict | Evidence |
|-----|-----------|---------|----------|
| **(M)** | Futures/margin balances and positions ingested; non-spot `product_type` in holdings | **PASS** (code) / **DEFERRED** (runtime) | M1: `sync_balances` appends `product_type: futures` USDT wallet; N2: `sync_positions` emits `product_type: linear`. Wiremock tests PASS. Live holdings probe **DEFERRED** until deploy + exchange sync |
| **(N)** | Futures REST uses `fapi.bitunix.com` header-auth; sync/test populate when keys permit | **PASS** (code) / **DEFERRED** (runtime) | N1: `bitunix_futures_sign` + `futures_signed_get` on `futures_base_url`; N3: `effective_enabled_futures()` gates all futures paths; N4: dual test message. Live test endpoint **DEFERRED** until deploy |
| **(O)** | Wealth crypto subtotal reflects spot + futures wallet holdings | **PASS** (code) / **DEFERRED** (runtime) | DEC-0064: M1 wallet rows priced (`market_value_usd` for USDT); N2 positions `market_value_usd: None` — no WealthService change. Live `GET /api/v1/wealth` **DEFERRED** until deploy + exchange sync |
| Regression | OIDC-enabled + bundled-firefly deploy checks | **DEFERRED** | plan-verify; verify-work uat.md REG-1/REG-2 |

**Summary:** N1–N4 **PASS** on static/automated path; full acceptance runtime deferred with `OPERATOR_DEPLOY_PENDING` + `OPERATOR_EXCHANGE_SYNC_PENDING`.

## Architecture compliance

### N1 — Futures header-auth (DEC-0062)

| Contract element | Expected | Observed | Verdict |
|------------------|----------|----------|---------|
| Sign helper | `bitunix_futures_sign(nonce+timestamp+api-key+query+body)` | `http.rs` L93–107 | PASS |
| Official fixture | R-0058 published sign | `bitunix_futures_sign_matches_official_fixture` | PASS |
| Futures GET wrapper | Header auth on `futures_base_url` | `bitunix.rs` L54–73 `futures_signed_get` | PASS |
| Spot unchanged | `signed_get` + `bitunix_sign` preserved | L37–52; spot regression test | PASS |
| Config default | `https://fapi.bitunix.com` | `default.toml` L113; `default_bitunix_futures_url()` | PASS |
| GET-only audit | Shared HTTP client | `ExchangeHttpClient::audit_method` unchanged | PASS |

### N3 — effective_enabled_futures (DEC-0063)

| Contract element | Expected | Observed | Verdict |
|------------------|----------|----------|---------|
| Env opt-out | `BITUNIX_ENABLED_FUTURES=false` → off | `config/mod.rs` L535–538; unit test | PASS |
| Env opt-in | `true` → on | L540–542 | PASS |
| TOML flag | `enabled_futures = true` → on | L544–546 | PASS |
| Auto-enable | `effective_enabled() && credentials()` | L547 | PASS |
| Settings API | Exposes effective value | `settings_view` L611 `effective_enabled_futures()` | PASS |
| `.env.example` | Document opt-out | L118–119 | PASS |
| Gate usage | All futures paths use effective | `sync_balances`, `sync_positions`, `sync_funding`, `test_connection` | PASS |

### M1 — Futures wallet ingestion

| Contract element | Expected | Observed | Verdict |
|------------------|----------|----------|---------|
| Endpoint | `/api/v1/futures/account?marginCoin=USDT` | L327 | PASS |
| product_type | `"futures"` | L162 | PASS |
| USDT pricing | `market_value_usd: Some(qty)` for stablecoins | L153–157 | PASS |
| Append spot | Spot path unchanged; futures appended | L322–339 | PASS |
| Opt-out regression | Spot-only when futures disabled | `sync_balances_spot_only_when_futures_disabled` | PASS |
| Partial failure | Futures wallet error does not fail spot sync | L335–337 warn-only | PASS (acceptable) |

### N2 — sync_positions (DEC-0064)

| Contract element | Expected | Observed | Verdict |
|------------------|----------|----------|---------|
| Endpoint | `get_pending_positions` | L352–354 | PASS |
| product_type | `"linear"` | L221 | PASS |
| market_value_usd | `None` | L222 | PASS |
| active_symbols | Symbols pushed for trade sync | L203–205; test asserts `BTCUSDT` | PASS |
| Disabled no-op | `Ok(vec![])` when futures off | L348–350 | PASS |

### N4 — Dual-path test_connection

| Contract element | Expected | Observed | Verdict |
|------------------|----------|----------|---------|
| Spot always probed | `/api/spot/v1/user/account` | L269 | PASS |
| Futures when enabled | `/api/v1/futures/account?marginCoin=USDT` | L290–292 | PASS |
| Both OK | `Spot: OK; Futures: OK` | L298 | PASS |
| Partial OK | `ok: true`, `Spot: OK; Futures: {err}` | L301–306; unit test | PASS |
| Spot fail | `ok: false`, `Spot: {err}` | L309–314 | PASS |
| Spot-only message | Unchanged when futures disabled | L276 `Spot balance read OK` | PASS |

### Frozen boundaries

| Boundary | Verdict |
|----------|---------|
| No merge with BUG-0006 scope | PASS |
| No `sync_funding` implementation (stub only) | PASS |
| USDT `marginCoin` MVP only | PASS |
| No Binance/Bybit connector changes | PASS |
| Read-only GET-only guarantee (DEC-0037) | PASS |
| O1 deferred to verify-work after deploy + exchange sync | PASS |

## Generated test evidence

| Field | Value |
|-------|-------|
| `generated_test_stack_profile` | `rust` + `typescript` |
| `generated_test_command` | `cd backend && cargo test --lib && cd ../frontend && npm test && npm run build` |
| `generated_test_result` | `pass` |
| `generated_test_output_ref` | QA run 2026-06-05 — 123/123 lib tests; vitest 2/2; tsc + vite build exit 0 |
| `generated_test_paths_ref` | `backend/src/exchanges/http.rs` (N1), `backend/src/exchanges/bitunix.rs` (M1/N2/N4), `backend/src/config/mod.rs` (N3) |

## Runtime QA evidence (omniflow)

| Field | Value |
|-------|-------|
| `runtime_startup_command` | Not executed (operator-owned deploy + exchange sync) |
| `runtime_stack_profile` | `docker-compose` external profile |
| `runtime_mode` | `deferred` |
| `runtime_health_target` | BUG-0005 rows M/N/O + regression on omniflow |
| `runtime_health_result` | `deferred` |
| `runtime_final_verdict` | `deferred` (verify-work O1 / operator) |
| `runtime_reason_code` | `OPERATOR_DEPLOY_PENDING`, `OPERATOR_EXCHANGE_SYNC_PENDING` |
| `runtime_evidence_refs` | `handoffs/dev_to_qa.md`; `sprints/quick/Q0012/uat.md`; `docs/engineering/architecture.md` § BUG-0005 |

## Findings

### Blockers

None for proceeding to deploy → operator **manual exchange sync** → `/verify-work` O1.

### Advisories (non-blocking)

1. **Deploy Q0012** backend image to omniflow before verify-work.
2. **Operator gate:** Trigger **manual exchange sync** (not Full Firefly sync) after deploy before holdings/wealth probes.
3. **Partial futures failure in sync_balances:** Futures wallet errors are logged but do not fail spot sync — acceptable per N4 partial-OK pattern; O1 will surface live futures connectivity.
4. **Acceptance checkbox:** `docs/product/acceptance.md` BUG-0005 remains unchecked until verify-work O1 passes.

## Verdict

**PASS** — proceed to deploy → operator **manual exchange sync** → `/verify-work` O1 in a fresh subagent/chat. No dev rework required; do not populate `handoffs/qa_to_dev.md`.
