# PO to TL archive pack (2026-06-13)

- Rollover trigger: `PO_TO_TL_HOT_MAX_LINES=500, PO_TO_TL_HOT_MAX_SECTIONS=40`
- Source: `handoffs/po_to_tl.md`
- Archived units (oldest first, contiguous prefix): 27
- Retained units in hot file: 40
- First archived heading: `## Architecture summary`
- Last archived heading: `## Recommended next phase`
- Verification tuple (mandatory):
  - archived_body_lines=217
  - retained_body_lines=357

---

## Architecture summary

Nine research gates **frozen**. Three new decisions formalized: **DEC-0112** (transaction search API), **DEC-0113** (dual-mode Discover UX), **DEC-0114** (hint pass boundary). Extends **DEC-0098**, **DEC-0099**, **DEC-0111** — no amendments to auto-detection or patterns tab contract.

## Gates resolved

| Gate | Resolution | DEC |
|------|------------|-----|
| **GATE-UX-1** | Dual mode — Transactions (default) \| Suggested patterns | **DEC-0113** |
| **GATE-API-1** | `GET /api/v1/subscriptions/transactions/search` | **DEC-0112** |
| **GATE-FILTER-1** | SQL push-down + accounts JOIN (DEC-0111 COALESCE) | **DEC-0112** |
| **GATE-HINT-1** | Separate hint pass; min 60; row metadata only | **DEC-0114** |
| **GATE-HINT-2** | P2 defer — 2-tx weak hints | — |
| **GATE-PAGE-1** | Offset 100/page; `total_count` + `has_more` | **DEC-0112** |
| **GATE-IDX-1** | P2 defer — composite index | — |
| **GATE-CONFIRM-1** | Reuse `POST /discover/confirm`; add preview-group | **DEC-0112** |
| **GATE-DEC-1** | **DEC-0112**, **DEC-0113**, **DEC-0114** | closed |

## Frozen contracts

- **Transactions mode (default):** paginated expense ledger + rich filters + hint badges + multi-select confirm
- **Suggested patterns mode:** DEC-0098 candidate table unchanged
- **Search API:** required `account_id`; optional payee, category, Geldbereich, date, recurring_hint; 100/page max
- **Hint pass:** display-only; global detection + discover candidates unchanged (AC-5)
- **Confirm:** preview-group → existing DEC-0099 confirm body

## Sprint recommendation

**S0020** — 12 tasks at `SPRINT_MAX_TASKS` (9 P0 + 2 P1 + V1); P2 optional (amount band, index, weak hints) excluded from mandatory count. See architecture § US-0021 task table (TX1–TX3, UI1–UI4, PT1, T1–T2, R1, V1).

## Operator repro fixture (qa)

- `localhost:18080`, account **114**, SEPA-Lastschrift 11 txs @ 31d / 95% — individual rows + hint badge in Transactions mode

## Blockers

**None** — ready for `/sprint-plan`.

## Architecture artifact

[architecture.md § US-0021](docs/engineering/architecture.md#us-0021--subscription-transaction-explorer-with-rich-filters) · [DEC-0112](../decisions/DEC-0112.md) · [DEC-0113](../decisions/DEC-0113.md) · [DEC-0114](../decisions/DEC-0114.md) · spec-pack US-0021

## Recommended next phase

`/sprint-plan` (tech-lead) — materialize **S0020** from architecture task table.

---

# research-20260613-us0021 — US-0021 Subscription transaction explorer with rich filters

**From:** Tech Lead **To:** Tech Lead (architecture phase) **Story:** US-0021 **Run:** `auto-20260613-us0021`
**Date:** 2026-06-13 **Next phase:** `/architecture` (role: tech-lead)
**Intake evidence:** `handoffs/intake_evidence/intake-20260612-subscription-tx-explorer.json` (read-only)

## Research summary

Discovery gaps confirmed. [R-0092 §5–8](docs/engineering/research.md#r-0092--us-0021-subscription-transaction-explorer-vs-recurrence-only-discover) extended with architecture gates, API contract draft, hint threshold analysis, and pagination cap. **Nine gates** documented; **no blockers** for architecture.

## Discovery gaps (confirmed)

| AC | Verdict |
|----|---------|
| **AC-1** | CONFIRMED GAP — no tx ledger API |
| **AC-2** | CONFIRMED GAP — missing category, Geldbereich, date, amount, recurring filters |
| **AC-3** | CONFIRMED GAP — no hint-only pass below auto threshold |
| **AC-4** | PARTIAL GAP — candidate-row confirm only; no tx multi-select |
| **AC-5** | BASELINE OK |
| **AC-6** | DEFERRED — OIDC to qa |

## Architecture gates (mandatory decisions)

| Gate | Question | TL default |
|------|----------|------------|
| **GATE-UX-1** | Discover layout | **Dual mode** — Transactions (default) \| Suggested patterns (DEC-0098 unchanged) |
| **GATE-API-1** | Transaction search route | New `GET /api/v1/subscriptions/transactions/search` — not `/discover?mode=transactions` |
| **GATE-FILTER-1** | Filter execution | SQL push-down (account, date, category, payee, amount) + accounts JOIN for Geldbereich (DEC-0111) |
| **GATE-HINT-1** | Hint engine | Separate pass on filtered txs; `min_emit_confidence: 60`; row hints only; **global detection unchanged** |
| **GATE-HINT-2** | Sub-threshold 2-tx hints | **P2 defer** — current scorer returns 0 below 60 |
| **GATE-PAGE-1** | Pagination | Offset **100 txs/page** hard cap; `total_count` + `has_more`; document in UI |
| **GATE-IDX-1** | DB index | P2 `idx_transactions_account_date` — not blocking MVP |
| **GATE-CONFIRM-1** | Multi-select activate | Reuse `POST /discover/confirm`; add `POST .../transactions/preview-group` for server median/interval |
| **GATE-DEC-1** | New DEC? | **Yes** — tx-search API + dual-mode UX + hint boundary (extends DEC-0098/0099) |

## Frozen contracts (research recommendations)

### Dual-mode UX

- **Transactions** tab (default): paginated expense ledger + rich filters + hint badges + multi-select confirm.
- **Suggested patterns** tab: existing US-0020 recurrence-candidate table — filters account/payee/interval only; DEC-0098 contract frozen.

### API contract

`GET /api/v1/subscriptions/transactions/search` — required `account_id`; optional `payee`, `category_id`, `account_role`, `date_from`, `date_to`, `amount_min`, `amount_max`, `recurring_hint`, `page`, `limit` (max 100). Response `transactions[]` with optional `recurring_hint` object; `meta` includes `page`, `limit`, `total_count`, `has_more`, `window_days`.

### Hint threshold

- Auto-detection + discover candidates: **min_emit_confidence 60 unchanged** (AC-5).
- Tx-search hint pass: run `detect_recurrence_groups` on SQL-filtered subset; attach hints to matching rows; **no** auto-emit or pending creation.

### Pagination

- **100/page** offset MVP; keyset cursor deferred (EARLY_RESEARCH: household account-scoped windows typically &lt;500 txs).

## Operator repro fixture (qa)

- `localhost:18080`, account **114**, SEPA-Lastschrift 11 txs @ 31d / 95% — individual rows + hint badge expected in Transactions mode.

## Blockers

**None** — architecture must freeze gates and emit DEC records.

## Acceptance rows (unchanged)

- **AC-1** Transaction search — individual expense txs, paginated/capped
- **AC-2** Rich filters — account, payee, category, Geldbereich, date (+ optional amount/recurring)
- **AC-3** Pattern hint — recurring suggestion on filtered txs below auto threshold
- **AC-4** Manual activate — confirm tx group as subscription/standing order (DEC-0085/0099)
- **AC-5** Regression — US-0020 tags/majority + US-0003/US-0008 unchanged
- **AC-6** OIDC external profile smoke

## Research artifact

[R-0092 §5–8](docs/engineering/research.md#r-0092--us-0021-subscription-transaction-explorer-vs-recurrence-only-discover)

## Recommended next phase

`/architecture` (tech-lead) — freeze GATE decisions, emit DEC-0112+ records, document contracts, size sprint (est. 11–13 tasks).

---

# discovery-20260613-us0021 — US-0021 Subscription transaction explorer with rich filters

**From:** PO **To:** Tech Lead **Story:** US-0021 **Run:** `auto-20260613-us0021`
**Date:** 2026-06-13 **Next phase:** `/research` (role: tech-lead)
**Intake evidence:** `handoffs/intake_evidence/intake-20260612-subscription-tx-explorer.json` (read-only)

## Discovery summary

Code + live audit confirms **US-0020 shipped recurrence-candidate Discover** (`detect_recurrence_groups` → `DiscoverCandidate` rows) while operator expectation (intake) is **transaction-first ledger search** with **category**, **Geldbereich**, **date**, and **manual tx-group activate**. Scope expansion — not a US-0020 defect. Live probe **`GET http://localhost:18080/api/v1/subscriptions/discover?account_id=114&limit=2`** (2026-06-13) returns grouped candidates (e.g. SEPA-Lastschrift 11 txs @ 31d / 95%) — no individual transaction rows, no category/Geldbereich dimensions.

## Acceptance sub-defects (AC audit)

| AC | Verdict | Key evidence |
|----|---------|--------------|
| **AC-1** Transaction search | **CONFIRMED GAP** | `discovery.rs` L36–108 emits `DiscoverCandidate` only; UI table columns Name/Interval/Median/Confidence/Tx count — not per-tx ledger; no pagination over txs |
| **AC-2** Rich filters | **CONFIRMED GAP** (partial baseline) | **Done:** account (required), payee substring, interval bucket (`SubscriptionsPage.tsx` L396–465; `DiscoverQuery` account/payee/interval). **Missing:** category, Geldbereich (`account_role`), date range, amount band, recurring/pattern-hint toggle — no `CategoryFilter` / `formatAccountRole` on Discover tab (contrast `WealthPage.tsx`, `ForecastPage.tsx`) |
| **AC-3** Pattern hint | **CONFIRMED GAP** | `min_emit_confidence: 60` hard-coded in `run_discover` L33; groups below threshold never surface; no hint-only pass on filtered subset; no inline interval suggestion on tx rows |
| **AC-4** Manual activate | **PARTIAL GAP** | `POST /api/v1/subscriptions/discover/confirm` accepts `transaction_ids` + DEC-0085 merge (`subscriptions.rs` L224–270) — but UI confirm is **candidate-row only** (`DiscoverCandidate` modal L654–685); no multi-select tx group from ledger |
| **AC-5** Regression | **BASELINE OK** (unchanged) | US-0020 tags, majority category, Pending/auto-detection paths untouched in audit; regression proof deferred to execute/qa |
| **AC-6** OIDC | **DEFERRED** | External profile smoke not run this phase — carry to qa |

## Partial implementation review

| Area | Status |
|------|--------|
| `GET /api/v1/subscriptions/discover` recurrence candidates | **Done** (US-0020 / DEC-0098) |
| Account + payee + interval filters | **Done** |
| `POST /discover/confirm` + kind override + merge | **Done** |
| `load_expense_transactions` + mirror `category_id` | **Done** (reuse for tx search) |
| Individual tx search API + paginated response | **Missing** (AC-1) |
| Category / Geldbereich / date / amount filters | **Missing** (AC-2) |
| Hint engine on filtered txs (below auto threshold) | **Missing** (AC-3) |
| Tx multi-select → confirm modal | **Missing** (AC-4) |
| `CategoryFilter` + `formatAccountRole` on Discover | **Missing** |

## Root-cause chain (code + live)

| Layer | Finding |
|-------|---------|
| **Backend discover** | `run_discover` loads txs then **only** `detect_recurrence_groups`; filters apply to groups not raw txs (`discovery.rs` L18–108) |
| **API contract** | `DiscoverResponse.candidates[]` = grouped patterns; no `transactions[]` endpoint or `mode=transactions` |
| **Repository** | `load_expense_transactions` supports account scoping only — no SQL push-down for category, date, Geldbereich (`repository.rs` L62–96) |
| **Frontend Discover** | Heading "Discover recurring **candidates**"; results table is candidate-centric; account required default |
| **Reusable** | `confirm_from_discover`, DEC-0085 merge, US-0018 categories, DEC-0111 `account_role` labels — ready for extension per R-0092 |

## Decision gates (PO recommendation — carry to research)

| Topic | Recommendation | Alternative |
|-------|----------------|-------------|
| Discover layout | **Dual mode** — Transactions (default) \| Suggested patterns (current) | Replace recurrence table entirely |
| Transaction search API | New `GET /api/v1/subscriptions/transactions/search` | Extend `/discover?mode=transactions` |
| Hint threshold | Separate hint pass with lowered `min_emit_confidence` — hints only, no auto-emit | Client-side interval calc on selection |
| Geldbereich filter | Join `accounts.payload` `account_role` per DEC-0111 | Post-filter in Rust after load |
| Pagination cap | **100 txs/page**; document in UI (intake R-0092) | 50 (US-0020 candidate cap) |
| Manual confirm | Multi-select txs → compute median amount + interval → existing confirm body | New confirm payload shape |

## Open questions (carry to research — extend R-0092)

1. Dual-mode tab UX vs single merged table with hint badges?
2. SQL filter push-down vs in-memory filter after `load_expense_transactions`?
3. Hint engine: run `detect_recurrence_groups` on filtered subset with `min_emit_confidence` lowered for display only?
4. Operator repro: which payee/category missed auto-detection — document for qa fixture?
5. Performance: indexed filters on `account_id`, `date`, `category_id` for 730d window?

## Blockers

**None** for discovery → research handoff. Architecture must freeze API shape and dual-mode layout before execute.

## Acceptance rows (unchanged)

- **AC-1** Transaction search — individual expense txs, paginated/capped
- **AC-2** Rich filters — account, payee, category, Geldbereich, date (+ optional amount/recurring)
- **AC-3** Pattern hint — recurring suggestion on filtered txs below auto threshold
- **AC-4** Manual activate — confirm tx group as subscription/standing order (DEC-0085/0099)
- **AC-5** Regression — US-0020 tags/majority + US-0003/US-0008 unchanged
- **AC-6** OIDC external profile smoke

## Research artifact

[R-0092](docs/engineering/research.md#r-0092--us-0021-subscription-transaction-explorer-vs-recurrence-only-discover) — extend with discovery audit findings and frozen gates.

## Recommended next phase

`/research` (tech-lead) — API shape, dual-mode UX, hint threshold, Geldbereich join, pagination cap, operator repro fixture.

---

