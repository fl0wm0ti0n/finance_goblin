# Sprint Summary — Q0009 (BUG-0003)

**Bug:** BUG-0003  
**Sprint:** Q0009 (`/quick`)  
**Execute date:** 2026-06-05  
**Release date:** 2026-06-05  
**Status:** **DONE** — verify-work PASS; acceptance checked; backlog DONE

## Tasks delivered

| ID | Status | Summary |
|----|--------|---------|
| F2 | done | `.env.example` omniflow `DATABASE_HOST=postgres` guard; runbook § mis-host symptom table; compose comment |
| G1 | done | `ExchangeService::build_connectors` uses `effective_enabled()` for binance/bybit/bitunix |
| F1 | done | Operator F1 complete — `DATABASE_HOST=postgres`, recreate `flow-finance-ai` + `grafana` |
| G2 | skipped | Gated — no auth-failure smoke post-deploy; skip documented in progress |

## Tests

| Check | Result |
|-------|--------|
| `cargo test --lib` | **PASS** (103 tests @ release) |
| `npm test` (frontend) | **PASS** (2/2) |
| `npm run build` (frontend) | **PASS** |

## Acceptance (verify-work + release)

| Row | Result |
|-----|--------|
| **(F)** | **PASS** — `database_host: postgres`; product GET APIs **200** &lt;0.1s |
| **(G)** | **PASS** — `POST …/bitunix/test` **200** `Spot balance read OK` |
| **(H)** | **PASS** — Grafana `ds/query` **200**; `SELECT 1` executes |

## Files changed (primary)

- `backend/src/exchanges/service.rs` (G1)
- `.env.example` (F2)
- `docs/engineering/runbook.md` (F2, F1 ops, §11 release)
- `docker-compose.external.yml` (F2 comment)

## Release artifacts

- `handoffs/releases/Q0009-release-notes.md`
- `docs/product/acceptance.md` (BUG-0003 checked)
- `docs/product/backlog.md` (BUG-0003 DONE)

## Next

- **Refresh-context:** complete 2026-06-05 — `docs/engineering/state-archive/state-pack-20260605-q0009-bug0003.md`
- **Recommended bug target:** BUG-0004 (post-sync pipeline empty analytics)
