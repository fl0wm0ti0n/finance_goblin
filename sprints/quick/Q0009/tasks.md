# Tasks — Q0009 (BUG-0003)

**Bug:** BUG-0003  
**Task count:** 4 (3 required + G2 gated; within `SPRINT_MAX_TASKS=12`)

| ID | Title | Est. | Status | Acceptance |
|----|-------|------|--------|------------|
| F2 | External-profile DATABASE_HOST guard | 1h | open | **(F)** docs |
| G1 | `effective_enabled()` in `ExchangeService::new` | 2h | open | **(G)** code |
| F1 | Operator `DATABASE_HOST=postgres` + recreate | 1h | open | **(F)(H)** operator |
| G2 | R-0058 futures auth spike | 1.5h | gated | **(G)** conditional |

---

## F2 — External-profile DATABASE_HOST guard

**Status:** open  
**Depends on:** —  
**Estimate:** 1h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0003 **(F)** — guardrail; omniflow must not use `host.docker.internal`

### Description

| Artifact | Change |
|----------|--------|
| `.env.example` | Omniflow external block: `DATABASE_HOST=postgres` — do not copy greenfield `host.docker.internal` |
| `docs/engineering/runbook.md` | § Omniflow mis-host symptom table (~30s **500**); remediation → F1 |
| `docker-compose.external.yml` | Optional one-line comment above `DATABASE_HOST` (no behavior change) |

### Done when

- [ ] Doc review: omniflow block + runbook symptom table cite DEC-0056 / R-0052
- [ ] No hardcoded `postgres` in compose (keep `${DATABASE_HOST:-postgres}`)

---

## G1 — `effective_enabled()` in `ExchangeService::new`

**Status:** open  
**Depends on:** —  
**Estimate:** 2h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0003 **(G)** — bitunix test not **400** unknown exchange

### Description

Register binance/bybit/bitunix when `instance.effective_enabled()` (parity with Q0008 E1 mirror/settings).

**Files:** `backend/src/exchanges/service.rs`, `backend/src/config/mod.rs` (unit test)

### Done when

- [ ] Unit: configured + TOML `enabled=false` → connector in `new()` map
- [ ] `cargo test --lib` exchanges/config PASS
- [ ] `test_connection` still validates credentials (no bypass)

---

## F1 — Operator DATABASE_HOST=postgres + recreate

**Status:** open  
**Depends on:** F2 + G1 deployed (code PR) before acceptance smoke  
**Estimate:** 1h (operator)  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0003 **(F)(H)** — settings postgres; GET APIs **200**; Grafana ds/query **200**

### Description

| Step | Action |
|------|--------|
| 1 | Set `DATABASE_HOST=postgres` in operator `.env` (not committed) |
| 2 | Recreate `flow-finance-ai` + `grafana` |
| 3 | `GET /api/v1/settings` → `database_host: postgres` |
| 4 | Smoke representative GETs **200** &lt;2s |
| 5 | `POST …/analytics/grafana/api/ds/query` **200** (row H) |

### Done when

- [ ] Container env names-only: `DATABASE_HOST=postgres` on backend + grafana
- [ ] No **500** ~30s cascade on sample product APIs
- [ ] Grafana SQL panels execute

---

## G2 — R-0058 futures header-auth spike (gated)

**Status:** gated  
**Depends on:** G1, F1 + smoke gate  
**Estimate:** 1.5h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0003 **(G)** — **200** test payload or documented auth failure

### Gate (execute only if)

After **G1 + F1** deploy, `POST /api/v1/exchanges/bitunix/test`:

- Not **400** `unknown exchange: bitunix`, **and**
- Response indicates auth/URL failure (not success)

Then spike per R-0058: `fapi.bitunix.com`, headers `api-key`, `nonce`, `timestamp`, `sign`.

**Files:** `backend/src/exchanges/bitunix.rs` (conditional)

### Done when

- [ ] Gate documented in smoke notes; skip if registry-only fix sufficient
- [ ] Operator smoke: test body documents auth/URL or **200** payload

---

## Execution order

1. **F2**, **G1** — parallel dev work → single PR / image deploy  
2. **F1** — operator recreate (required before F/G/H acceptance)  
3. **Smoke** — rows F, G, H + regression footer  
4. **G2** — only if gate triggers (not unknown exchange)

## Split decision

- **Why 4 tasks:** Maps architecture slices F1/F2, G1/G2; F1 separated as ops gate; H1 = F1 verify.
- **Why not split Q0009a/b:** Shared omniflow deploy contract; 4 ≪ 12 threshold.
- **Why G2 gated:** Discovery proved **&lt;0.2s** **400** registry gap; R-0058 auth only if smoke shows auth failure after G1+F1.
- **No new DEC:** Ops/docs under DEC-0056; G1 completes Q0008 E1 parity in `ExchangeService::new`.
