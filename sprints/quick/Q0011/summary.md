# Sprint Summary — Q0011 (BUG-0004)

**Bug:** BUG-0004  
**Sprint:** Q0011 (`/quick`)  
**Execute date:** 2026-06-05  
**Release date:** 2026-06-05  
**Status:** **DONE** — released; acceptance checked

## Tasks delivered

| ID | Status | Summary |
|----|--------|---------|
| I1 | done | `finish_sync_run` on `RunMode::ExchangesOnly` terminal path in `sync/mod.rs` |
| K1 | done | Portfolio pie panel id 8 UNION branches wrapped in parentheses |
| L1 | done | `parse_split_amount` for Firefly `current_balance` (DEC-0060) |
| L2 | done | `COALESCE(balance, 0) >= 0` in `load_asset_accounts` |
| J1 | done | `extract_payee_source` + DEC-0061 fallbacks in `by_payee()` |
| J2 | done | Subscriptions empty-state thresholds + pending-review banner |
| L3 | done | verify-work PASS — omniflow rows I/J/K/L live |

## Tests

| Check | Result |
|-------|--------|
| `cargo test --lib` | **PASS** (110 tests) |
| `npm test` (frontend) | **PASS** (2/2) |
| `npm run build` (frontend) | **PASS** |

## Acceptance (released)

| Row | Code task | Runtime verify |
|-----|-----------|----------------|
| **(I)** | I1 | **PASS** — manual_exchanges terminal `success`; `finished_at` set |
| **(K)** | K1 | **PASS** — portfolio ds/query **200** |
| **(L)** | L1, L2 | **PASS** — wealth 2 accounts; forecast daily **200** |
| **(J)** | J1, J2 | **PASS** — 11 pending patterns; J2 UI deferred (401) |

## Files changed (primary)

- `backend/src/sync/mod.rs` (I1)
- `grafana/provisioning/dashboards/analytics/portfolio.json` (K1)
- `backend/src/firefly/mod.rs` (L1)
- `backend/src/wealth/repository.rs` (L2)
- `backend/src/recurrence/group.rs` (J1)
- `frontend/src/pages/SubscriptionsPage.tsx` (J2)

## Release artifacts

- `handoffs/releases/Q0011-release-notes.md`
- `docs/product/acceptance.md` — BUG-0004 checked
- `docs/product/backlog.md` — BUG-0004 DONE
- `docs/engineering/runbook.md` — §12 BUG-0004

## Known issues (non-blocking)

- 10 historical pre-I1 stuck `scheduled_exchanges` rows — out of scope
- J2 UI threshold copy — operator browser smoke recommended
- Zero balances — structural population met; verify Firefly source if non-zero expected

## Next

- Recommend `/auto` **BUG-0005** or **BUG-0006** (Q0010)
