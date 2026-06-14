# CRS — US-0021

## Purpose

Enable operators to **search individual expense transactions** with rich filters, see **recurring hints** on filtered subsets below the candidate-only UX, and **manually activate** selected transaction groups — extending US-0020 Discover without regressing auto-detection or the patterns tab.

## Scope

**In scope**

- `GET /api/v1/subscriptions/transactions/search` (DEC-0112)
- `POST /api/v1/subscriptions/transactions/preview-group` (DEC-0112)
- Hint pass on search results (DEC-0114)
- Dual-mode Discover UX: Transactions \| Suggested patterns (DEC-0113)
- Rich filters: account, payee, category, Geldbereich, date range
- Multi-select confirm via existing `POST /discover/confirm` (DEC-0099)
- `docs/user-guides/US-0021.md`
- US-0003/US-0008 regression tests (AC-5)
- OIDC external profile smoke (AC-6)

**Out of scope**

- DEC-0098 `/discover` contract changes
- Global detection threshold changes
- Firefly write-back
- Amount band filters (P2)
- `idx_transactions_account_date` index (P2)
- 2-tx weak hints (P2 GATE-HINT-2)

## Acceptance criteria ref

See `docs/product/acceptance.md` § US-0021 — AC-1 through AC-6.

## Dependencies

- US-0020 DONE (DEC-0098 patterns tab, DEC-0099 confirm)
- US-0018 DONE (CategoryFilter / category catalog)
- DEC-0111 DONE (account_role SQL + formatAccountRole)
- DEC-0085 DONE (merge on manual confirm)
- R-0092 research gates resolved
