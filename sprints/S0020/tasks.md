# Tasks — Sprint S0020

**Story:** US-0021  
**Task count:** 12 (at SPRINT_MAX_TASKS=12; P2 stretch excluded)

| ID | Title | Status | Slice | Est. | Acceptance refs |
|----|-------|--------|-------|------|-----------------|
| TX1 | Repository SQL search + COUNT + role JOIN | open | S1 | 4h | AC-1, AC-2 |
| TX2 | Search service + hint pass | open | S1 | 4h | AC-3 |
| TX3 | GET search + POST preview-group routes | open | S1 | 3h | AC-1, AC-4 |
| UI1 | Dual-mode tab shell (DEC-0113) | open | S2 | 3h | AC-1 |
| UI2 | Rich filter bar | open | S2 | 4h | AC-2 |
| UI3 | Tx table + pagination + hints | open | S2 | 4h | AC-1, AC-3 |
| UI4 | Multi-select confirm flow | open | S2 | 4h | AC-4 |
| PT1 | Patterns sub-tab extraction | open | S3 | 3h | AC-5 |
| T1 | Search + hint integration tests | open | S4 | 3h | AC-1, AC-3 |
| T2 | AC-5 regression tests | open | S4 | 2h | AC-5 |
| R1 | User guide US-0021 | open | S4 | 2h | — |
| V1 | UAT OIDC smoke + AC-1..AC-6 template | open | V1 | 2h | AC-6 |

---

## TX1 — Repository SQL search + COUNT + role JOIN

**Status:** open  
**Depends on:** —  
**Decisions:** DEC-0112, DEC-0111  
**Architecture slice:** US-0021-S1  
**Research:** R-0092 §6.2

### Description

Extend `backend/src/subscriptions/repository.rs` with parameterized transaction search:

| Filter | SQL contract |
|--------|--------------|
| `account_id` | Required — `WHERE t.account_id = $n` |
| `date_from` / `date_to` | Clamped to `detection_window_days` (730) |
| `category_id` | `t.category_id = $n` |
| `payee` | `t.description ILIKE '%' \|\| $n \|\| '%'` |
| Geldbereich | `JOIN accounts a` + DEC-0111 COALESCE `account_role` |
| Pagination | `LIMIT` max 100; offset from `page` |

Separate `COUNT(*)` query for `meta.total_count`. Expense-only mirror txs (reuse `load_expense_transactions` patterns).

### Done when

- [ ] Search query returns individual tx rows with account_role, category fields
- [ ] COUNT matches filtered set for pagination meta
- [ ] Missing `account_id` rejected at route layer (400)
- [ ] `cargo test` repository search tests green

---

## TX2 — Search service + hint pass

**Status:** open  
**Depends on:** TX1  
**Decisions:** DEC-0114, DEC-0098  
**Architecture slice:** US-0021-S1

### Description

Search service layer (`discovery.rs` or new `transaction_search.rs`):

1. Call repository search (≤100 rows per page for response)
2. When `recurring_hint` enabled (default true in UI): load up to **500 tx** hint budget from same SQL filters
3. Run `detect_recurrence_groups` with `min_emit_confidence: 60` on hint subset
4. Map group membership → row `recurring_hint` object (interval, confidence, tx count)
5. Exclude confirmed/rejected fingerprints (same as DEC-0098 discover)
6. **No** auto-emit, pending creation, or `new_detection` alerts

### Done when

- [ ] Hint metadata attached only to matching rows in search response
- [ ] Global `DetectionPipeline` / `run_discover` untouched
- [ ] 500 tx scan cap enforced
- [ ] `cargo test` hint attachment tests green

---

## TX3 — GET search + POST preview-group routes

**Status:** open  
**Depends on:** TX2  
**Decisions:** DEC-0112, DEC-0099  
**Architecture slice:** US-0021-S1

### Description

**Routes** in `backend/src/api/subscriptions.rs`:

| Route | Contract |
|-------|----------|
| `GET /api/v1/subscriptions/transactions/search` | Query params per DEC-0112; response `transactions[]` + `meta` |
| `POST /api/v1/subscriptions/transactions/preview-group` | Body `{ transaction_ids: [...] }` (≥2); response `{ payee_key, interval_days, median_amount, transaction_ids }` |

Preview-group feeds existing `DiscoverConfirmBody` → `POST /discover/confirm` (DEC-0099).

Extend `frontend/src/lib/api.ts` with search + preview types and fetchers.

### Done when

- [ ] GET search registered with OpenAPI if applicable
- [ ] POST preview-group validates shared payee_key + ≥2 txs
- [ ] API types exported in `api.ts`
- [ ] `cargo test` route handler tests green

---

## UI1 — Dual-mode tab shell (DEC-0113)

**Status:** open  
**Depends on:** — (can parallel TX1)  
**Decisions:** DEC-0113  
**Architecture slice:** US-0021-S2

### Description

`SubscriptionsPage.tsx` Discover section:

- Segmented control: **Transactions** | **Suggested patterns**
- **Transactions** default on tab visit
- Shared account/payee state where sensible between modes
- Patterns mode placeholder until PT1 extracts existing UI

### Done when

- [ ] Mode toggle renders; Transactions selected by default
- [ ] Switching modes preserves account selection
- [ ] No regression to All/Standing/Pending tabs

---

## UI2 — Rich filter bar

**Status:** open  
**Depends on:** UI1, TX3  
**Decisions:** DEC-0112, DEC-0111, US-0018  
**Architecture slice:** US-0021-S2

### Description

Transactions mode filter bar:

| Control | Source |
|---------|--------|
| Account | Required select (default first account) |
| Payee | Text input (substring) |
| Category | `CategoryFilter` (US-0018) |
| Geldbereich | `account_role` select with `formatAccountRole` (DEC-0111) |
| Date range | `date_from` / `date_to` inputs |
| Recurring hint | Toggle (default on) — drives `recurring_hint` param |

Search triggers GET `/transactions/search` with composed query.

### Done when

- [ ] All AC-2 filters wired to API params
- [ ] Account required before search (400 guard mirrored in UI)
- [ ] CategoryFilter + Geldbereich labels match Wealth/Forecast surfaces

---

## UI3 — Tx table + pagination + hints

**Status:** open  
**Depends on:** UI2  
**Decisions:** DEC-0112, DEC-0114  
**Architecture slice:** US-0021-S2

### Description

Transactions results table:

| Column | Source |
|--------|--------|
| Select checkbox | Multi-select for UI4 |
| Date | `date` |
| Description | `description` |
| Amount | `amount` |
| Category | `category_name` |
| Geldbereich | `formatAccountRole(account_role)` |
| Hint badge | `recurring_hint` when present |

Pagination controls for `page`; banner when `meta.has_more` or `truncated`; document 100/page cap in UI copy.

### Done when

- [ ] Table renders individual txs (not candidate groups)
- [ ] Hint badge shows interval/confidence when `recurring_hint` set
- [ ] Pagination + truncation banner visible when applicable
- [ ] `npm test` table smoke passes if applicable

---

## UI4 — Multi-select confirm flow

**Status:** open  
**Depends on:** UI3, TX3  
**Decisions:** DEC-0099, DEC-0085  
**Architecture slice:** US-0021-S2

### Description

Multi-select flow:

1. Operator selects ≥2 txs in table
2. **Activate** → `POST /transactions/preview-group`
3. Confirm modal shows payee, interval, median amount (reuse discover confirm modal patterns)
4. Kind override (subscription vs standing order) if needed
5. `POST /discover/confirm` with DEC-0099 body — merge path DEC-0085 preserved

### Done when

- [ ] Multi-select enables activate when ≥2 rows selected
- [ ] Preview-group populates confirm modal fields
- [ ] Confirm creates/merges confirmed subscription without pending-only path
- [ ] 409 on rejected payee-interval preserved

---

## PT1 — Patterns sub-tab extraction

**Status:** open  
**Depends on:** UI1  
**Decisions:** DEC-0098, DEC-0113  
**Architecture slice:** US-0021-S3

### Description

Move existing US-0020 Discover candidate UI (account + payee + interval filters, candidate table, row confirm) behind **Suggested patterns** sub-mode.

- No API or confirm contract changes
- `GET /discover` behavior unchanged
- Filters: account, payee, interval only (DEC-0098 frozen)

### Done when

- [ ] Patterns mode matches pre-US-0021 discover behavior
- [ ] Candidate confirm modal still works from patterns rows
- [ ] Transactions mode does not show candidate-only table by default

---

## T1 — Search + hint integration tests

**Status:** open  
**Depends on:** TX2, TX3  
**Decisions:** DEC-0112, DEC-0114  
**Architecture slice:** US-0021-S4

### Description

Integration tests in `backend/src/subscriptions/`:

- SQL filter push-down (account, category, date, payee, role)
- Pagination meta (`total_count`, `has_more`, limit 100)
- Hint attachment on filtered subset without pending/auto-emit
- Preview-group median/interval computation

### Done when

- [ ] Integration tests pass in `cargo test`
- [ ] Account 114-style fixture covered in test data or documented seed

---

## T2 — AC-5 regression tests

**Status:** open  
**Depends on:** PT1, UI4  
**Decisions:** DEC-0098, DEC-0099, US-0003, US-0008  
**Architecture slice:** US-0021-S4  
**Priority:** P1

### Description

Regression coverage for AC-5:

- `DetectionPipeline::run_candidates` unchanged — no threshold edits
- `run_discover` candidate pass unchanged (DEC-0098)
- US-0020 tags + majority category flows preserved
- US-0008 alert dedup contracts untouched
- Manual tx-search confirm does not emit `new_detection` alert

### Done when

- [ ] Regression tests pass
- [ ] Checklist captured for V1 in `uat.md`

---

## R1 — User guide US-0021

**Status:** open  
**Depends on:** UI4  
**Decisions:** DEC-0112, DEC-0113, DEC-0114  
**Architecture slice:** US-0021-S4  
**Priority:** P1

### Description

Publish `docs/user-guides/US-0021.md` (`USER_GUIDE_MODE=1`):

- Dual-mode Discover (Transactions vs Suggested patterns)
- Rich filters + 100/page cap + truncation banner
- Hint badges vs auto-detection boundary (AC-3 / AC-5)
- Multi-select manual activate workflow
- Operator repro: account 114 SEPA-Lastschrift fixture

### Done when

- [ ] User guide complete with operator-facing steps
- [ ] README/runbook links if required by doc profile

---

## V1 — UAT OIDC smoke + AC-1..AC-6 template

**Status:** open  
**Depends on:** T1, T2, R1  
**Decisions:** DEC-0112..DEC-0114  
**Architecture slice:** V1

### Description

Populate `sprints/S0020/uat.md` and `uat.json`:

- OIDC US-0010 external profile smoke checklist for AC-1..AC-6
- Operator gate **BACKEND_FRONTEND_DEPLOY** documented
- QA fixture: `localhost:18080`, account **114**, SEPA-Lastschrift 11 txs @ 31d / 95%
- Regression scope: US-0020 tags/majority, US-0003 pending, US-0008 dedup

### Done when

- [ ] UAT template lists 6 acceptance rows with probe steps
- [ ] Operator gates documented
- [ ] `uat.json` structured for plan-verify / verify-work

---

## Sequencing

```text
1. TX1 (repository)
2. TX2 (service + hints, after TX1)
3. TX3 (routes, after TX2)
4. UI1 (shell, parallel with TX1)
5. UI2 (filters, after UI1 + TX3)
6. UI3 (table, after UI2)
7. UI4 (confirm, after UI3)
8. PT1 (patterns extract, after UI1)
9. T1 (after TX3)
10. T2 (after PT1 + UI4)
11. R1 (after UI4)
12. V1 (after T1 + T2 + R1)
```

Dependency graph:

```text
TX1 → TX2 → TX3
UI1 → UI2 → UI3 → UI4
UI1 → PT1
TX3 → UI2, UI4, T1
PT1 + UI4 → T2
UI4 → R1
T1 + T2 + R1 → V1
```

## Acceptance map

| Row | Tasks | Verify |
|-----|-------|--------|
| AC-1 | TX1, TX2, TX3, UI1, UI3, V1 | Individual txs paginated; not candidates-only |
| AC-2 | TX1, UI2, V1 | category, Geldbereich, date filters |
| AC-3 | TX2, UI3, V1 | hint badge; account 114 fixture |
| AC-4 | TX3, UI4, V1 | multi-select → preview-group → confirm |
| AC-5 | PT1, T2, V1 | patterns tab + detection unchanged |
| AC-6 | V1 | OIDC external profile smoke |

## Notes

- **P2 deferred:** amount band, composite index, 2-tx weak hints — not in sprint mandatory count.
- **Detection unchanged:** No edits to `DetectionPipeline::run_candidates` or global `min_emit_confidence`.
- **DEC-0098 frozen:** Patterns tab uses existing discover API only.
