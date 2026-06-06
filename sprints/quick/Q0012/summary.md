# Sprint Summary — Q0012 (BUG-0005)

**Bug:** BUG-0005  
**Sprint:** Q0012 (`/quick`)  
**Execute date:** 2026-06-05  
**Status:** **DONE** — verify-work PASS + release PASS (2026-06-05)

## Tasks delivered

| ID | Status | Summary |
|----|--------|---------|
| N1 | done | Futures header-auth client + `futures_base_url` (DEC-0062) |
| N3 | done | `effective_enabled_futures()` policy + settings exposure (DEC-0063) |
| M1 | done | Futures wallet balance ingestion (`product_type: futures`) |
| N2 | done | `sync_positions` via `get_pending_positions` (`product_type: linear`, DEC-0064) |
| N4 | done | Dual-path `test_connection` (spot + futures sub-status) |
| O1 | done | verify-work omniflow probes — rows M/N/O PASS |

## Tests

| Check | Result |
|-------|--------|
| `cargo test --lib` | **PASS** (123 tests) |
| `npm test` (frontend) | **PASS** (2/2) |
| `npm run build` (frontend) | **PASS** |

## Acceptance (verified)

| Row | Verdict | Evidence |
|-----|---------|----------|
| **(M)** | **PASS** | Bitunix holdings 4 after exchange sync `f0906348`; non-spot `product_type` rows |
| **(N)** | **PASS** | `fapi.bitunix.com` header-auth; `enabled_futures: true`; dual test OK |
| **(O)** | **PASS** | Wealth `crypto.holdings_count: 4`; combined spot+futures per DEC-0064 |

## Files changed (primary)

- `backend/src/exchanges/http.rs` (N1)
- `backend/src/exchanges/bitunix.rs` (N1, M1, N2, N4)
- `backend/src/config/mod.rs` (N1, N3)
- `backend/config/default.toml` (N1)
- `.env.example` (N3)

## Archive

- `docs/engineering/state-archive/state-pack-20260605-q0012-bug0005.md`
