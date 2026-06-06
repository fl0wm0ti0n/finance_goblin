# UAT — Q0012 (BUG-0005)

**Status:** verify-work **PASS** (2026-06-05 re-run)  
**Acceptance:** `docs/product/acceptance.md` — BUG-0005 rows **(M)**, **(N)**, **(O)**  
**Orchestrator:** `auto-20260605-bug0005-002`

## Acceptance ↔ task traceability

| Row | Task(s) | Acceptance criterion (canonical) | Verify-work |
|-----|---------|-----------------------------------|-------------|
| **(M)** | M1, N2, O1 | Bitunix sync ingests futures/margin balances and positions; post-sync holdings include non-spot `product_type` | **PASS** |
| **(N)** | N1, N2, N3, N4, O1 | Futures REST uses `fapi.bitunix.com` header-auth; sync/test populate data when read-only keys permit | **PASS** |
| **(O)** | M1, N2, O1 | Wealth snapshot and portfolio crypto totals reflect combined spot + futures holdings | **PASS** |
| Regression | post-O1 | OIDC-enabled and bundled-firefly deploy regression checks pass | DEFERRED |

## Verify-work step results (re-run)

| Step | Description | Result |
|------|-------------|--------|
| V-1 | `cargo test --lib` | **PASS** (123/123) |
| V-2 | `npm test` | **PASS** (2/2) |
| V-3 | `npm run build` | **PASS** |
| V-4 | Omniflow reachability | **PASS** |
| V-5 | Q0012 deploy detection | **PASS** |
| V-6 | Post-deploy exchange sync | **PASS** — `f0906348` |
| M-1 | Row **(M)** non-spot holdings | **PASS** — holdings 4 |
| N-1 | Row **(N)** futures auth + test | **PASS** — dual test OK |
| O-1 | Row **(O)** wealth crypto subtotal | **PASS** — holdings_count 4 |
| REG-1 | OIDC regression | DEFERRED |
| REG-2 | Bundled-firefly regression | DEFERRED |

## Release gate

Proceed to **`/release`**.
