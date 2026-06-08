# Tasks — Sprint S0019

**Story:** US-0020  
**Task count:** 12 (at SPRINT_MAX_TASKS=12; T-0210 P2 optional)

| ID | Title | Status | Slice | Est. | Acceptance refs |
|----|-------|--------|-------|------|-----------------|
| T-0198 | Migration `display_category_id` + tag tables | open | S1/S3/S4 | 3h | AC-5 |
| T-0199 | Discover service + GET `/discover` route | open | S1 | 4h | AC-1 |
| T-0200 | Discover tab UI | open | S1 | 4h | AC-1 |
| T-0201 | POST `discover/confirm` + merge | open | S2 | 4h | AC-2 |
| T-0202 | Majority category compute + merge refresh | open | S3 | 3h | AC-3, AC-5 |
| T-0203 | Majority category badge + tooltip UI | open | S3 | 3h | AC-3 |
| T-0204 | Operator tag CRUD API | open | S4 | 3h | AC-4, AC-5 |
| T-0205 | PUT tag assign + list `?tag=` filter | open | S4 | 3h | AC-4 |
| T-0206 | Tag manager modal + filter chips UI | open | S5 | 4h | AC-4 |
| T-0207 | User guide US-0020 | open | S6 | 2h | — |
| T-0208 | US-0003/US-0008 regression tests | open | S6 | 2h | AC-6 |
| T-0209 | UAT OIDC smoke + AC-1..AC-6 template | open | V1 | 2h | AC-6 |
| T-0210 | Optional Grafana `$tag` variable | open | S7 | 2h | — |

---

## T-0198 — Migration `display_category_id` + tag tables

**Status:** open  
**Depends on:** —  
**Decisions:** DEC-0100, DEC-0101  
**Architecture slice:** US-0020-S1/S3/S4  
**Research:** R-0085

### Description

Add dedicated migration:

| Change | Contract |
|--------|----------|
| `display_category_id` | `TEXT NULL` on `subscription_patterns` — product overlay; no Firefly write-back |
| `operator_tags` | `id`, `slug` (unique), `label`, `created_at` |
| `subscription_pattern_tags` | junction `(pattern_id, tag_id)` PK; CASCADE on tag hard delete |

Update `backend/src/subscriptions/types.rs` (or equivalent) for new fields.

### Done when

- [ ] Migration applies cleanly on fresh and existing DBs
- [ ] Rust types expose `display_category_id` and tag relations
- [ ] `cargo test` migration/types tests green

---

## T-0199 — Discover service + GET `/discover` route

**Status:** open  
**Depends on:** T-0198  
**Decisions:** DEC-0098, DEC-0084, DEC-0086  
**Architecture slice:** US-0020-S1

### Description

**New** `backend/src/subscriptions/discovery.rs`:

| Step | Contract |
|------|----------|
| Load | `load_expense_transactions(window_days)` with `account_id` + date cutoff push-down |
| Group | `detect_recurrence_groups(&txs, &RecurrenceConfig::default())` |
| Filter | Payee `ILIKE`; interval via `interval_matches` (±3d); exclude confirmed/rejected fingerprints and payee-interval maps |
| Cap | Sort confidence DESC, tx count DESC; LIMIT 50; `meta.truncated` |

**Route:** `GET /api/v1/subscriptions/discover?account_id=&payee=&interval_days=&limit=50`

Frozen response shape per architecture § S1.

### Done when

- [ ] Service returns candidates with payee_key, interval, median_amount, confidence, tx ids
- [ ] Confirmed/rejected exclusions match `run_detection` semantics
- [ ] Route registered; 50-cap + meta documented
- [ ] `cargo test` discover service tests green

---

## T-0200 — Discover tab UI

**Status:** open  
**Depends on:** T-0199  
**Decisions:** DEC-0098  
**Architecture slice:** US-0020-S1

### Description

`SubscriptionsPage.tsx` — new **Discover** tab:

- Account select (required / first account default)
- Payee text input (substring)
- Interval bucket select (Weekly 7, Biweekly 14, Monthly 30, Quarterly 90, Custom)
- Results table: display name, interval, median amount, confidence, tx count
- Truncation notice when `meta.truncated`
- Confirm action wired in T-0201 (stub OK until C1 lands)

Extend `frontend/src/lib/api.ts` with discover fetcher + types.

### Done when

- [ ] Discover tab renders search form + results table
- [ ] Account filter required before search
- [ ] Cap/truncation copy visible when applicable
- [ ] `npm test` component smoke passes if applicable

---

## T-0201 — POST `discover/confirm` + merge

**Status:** open  
**Depends on:** T-0199, T-0198  
**Decisions:** DEC-0099, DEC-0085, DEC-0086  
**Architecture slice:** US-0020-S2

### Description

`POST /api/v1/subscriptions/discover/confirm`:

| Step | Contract |
|------|----------|
| Validate | Txs exist; share normalized `payee_key` (DEC-0084) |
| Confirmed map | `merge_confirmed_pattern` (DEC-0085/0086) → 200 `{ merged: true }` |
| Rejected map | 409 — operator must clear rejection |
| New | INSERT `confirmed` directly (skip pending); link txs; no `new_detection` alert |
| Fingerprint conflict | 409 same as detection |

**UI** — Discover row confirm button; kind override modal if needed; merge vs create toast.

### Done when

- [ ] Direct confirm creates confirmed row without pending intermediate
- [ ] Merge path preserves DEC-0085 behavior
- [ ] 409 on rejected payee-interval
- [ ] No alert emitted on manual confirm
- [ ] `cargo test` confirm-from-discover tests green

---

## T-0202 — Majority category compute + merge refresh

**Status:** open  
**Depends on:** T-0198, T-0201  
**Decisions:** DEC-0100, DEC-0087  
**Architecture slice:** US-0020-S3

### Description

Helper `compute_display_category_id(transaction_ids)`:

```sql
-- RANK: cnt DESC, last_date DESC (DEC-0100)
```

Invoke at:

- New confirm insert (T-0201)
- `merge_confirmed_pattern` refresh (DEC-0085 path)

NULL when no categorized txs → UI shows "Uncategorized".

### Done when

- [ ] `display_category_id` set on confirm and merge
- [ ] RANK tie-break matches DEC-0100
- [ ] Category names resolved via DEC-0087 catalog at API layer
- [ ] `cargo test` majority compute tests green

---

## T-0203 — Majority category badge + tooltip UI

**Status:** open  
**Depends on:** T-0202  
**Decisions:** DEC-0100, DEC-0087  
**Architecture slice:** US-0020-S3

### Description

Confirmed list + detail drawer:

- Badge showing category name from `GET /api/v1/categories`
- Tooltip: "Majority category from linked transactions; ties broken by most recent"
- NULL → "Uncategorized" label

### Done when

- [ ] Badge visible on All/Standing confirmed rows
- [ ] Tooltip documents tie-break rule (AC-3)
- [ ] `npm test` badge smoke passes if applicable

---

## T-0204 — Operator tag CRUD API

**Status:** open  
**Depends on:** T-0198  
**Decisions:** DEC-0101  
**Architecture slice:** US-0020-S4

### Description

**New** `backend/src/api/subscription_tags.rs`:

| Route | Contract |
|-------|----------|
| `GET /api/v1/subscription-tags` | List all tags |
| `POST /api/v1/subscription-tags` | Create `{ slug, label }` |
| `PUT /api/v1/subscription-tags/:id` | Rename label |
| `DELETE /api/v1/subscription-tags/:id` | Hard delete + CASCADE junction |

Slug normalization: lowercase, hyphenated.

### Done when

- [ ] CRUD routes registered and validated
- [ ] Hard delete removes junction rows (DEC-0101)
- [ ] `cargo test` tag CRUD tests green

---

## T-0205 — PUT tag assign + list `?tag=` filter

**Status:** open  
**Depends on:** T-0204, T-0198  
**Decisions:** DEC-0102  
**Architecture slice:** US-0020-S4

### Description

| Route | Contract |
|-------|----------|
| `PUT /api/v1/subscriptions/:id/tags` | Replace full tag set `{ tag_ids: [] }` |
| `GET /api/v1/subscriptions?tag=` | Filter by tag slug on list API |

Extend subscription DTO with `tags: [{ id, slug, label }]`.

### Done when

- [ ] PUT replaces tag set atomically
- [ ] List filter by slug returns matching confirmed subscriptions
- [ ] DTO includes tags array
- [ ] `cargo test` assign/filter tests green

---

## T-0206 — Tag manager modal + filter chips UI

**Status:** open  
**Depends on:** T-0205, T-0204  
**Decisions:** DEC-0101, DEC-0102  
**Architecture slice:** US-0020-S5

### Description

`SubscriptionsPage.tsx`:

- **Tag manager** modal: create/rename/delete with delete confirm dialog
- **Detail drawer:** multi-select tag chips on confirmed subscription
- **All/Standing tabs:** filter chips by tag slug (drives `?tag=` query)

### Done when

- [ ] Tag CRUD accessible from subscriptions page
- [ ] Multiple tags assignable per subscription
- [ ] List filter by tag chip works
- [ ] Delete confirm dialog on tag removal
- [ ] `npm test` tag UI smoke passes if applicable

---

## T-0207 — User guide US-0020

**Status:** open  
**Depends on:** T-0200, T-0203, T-0206  
**Decisions:** DEC-0098, DEC-0099, DEC-0100, DEC-0101, DEC-0102  
**Architecture slice:** US-0020-S6

### Description

Publish `docs/user-guides/US-0020.md` (`USER_GUIDE_MODE=1`):

- Discover tab workflow (account required, interval buckets, 50-cap)
- Manual confirm vs pending auto-detection path
- Majority category + tie-break semantics
- Tag manager + filter chips
- No Firefly write-back statement

### Done when

- [ ] User guide complete with operator-facing steps
- [ ] Links from README/runbook if required by doc profile

---

## T-0208 — US-0003/US-0008 regression tests

**Status:** open  
**Depends on:** T-0201, T-0206  
**Decisions:** DEC-0099, DEC-0084, DEC-0085, DEC-0086  
**Architecture slice:** US-0020-S6

### Description

Regression coverage for AC-6:

- `DetectionPipeline::run_candidates` unchanged — no threshold edits
- Pending confirm/reject flows preserved
- Manual discover confirm does not emit `new_detection` alert
- DEC-0085 merge behavior consistent across auto + manual paths
- US-0008 alert dedup contracts untouched

Add/extend `backend/src/subscriptions/` tests; document checklist in `uat.md`.

### Done when

- [ ] Detection pipeline regression tests pass
- [ ] Manual confirm no-alert test passes
- [ ] Pending tab flows smoke-tested
- [ ] Regression checklist captured for T-0209

---

## T-0209 — UAT OIDC smoke + AC-1..AC-6 template

**Status:** open  
**Depends on:** T-0207, T-0208  
**Decisions:** DEC-0098..DEC-0102  
**Architecture slice:** V1

### Description

Populate `sprints/S0019/uat.md` and `uat.json` (finalize probe steps):

- OIDC US-0010 external profile smoke checklist for AC-1..AC-6
- Operator gate **BACKEND_FRONTEND_DEPLOY** documented
- Regression scope: US-0003 pending flows, US-0008 alert dedup, read-only Firefly

### Done when

- [ ] UAT template lists 6 acceptance rows with probe steps
- [ ] Operator gates documented
- [ ] `uat.json` structured for plan-verify / verify-work

---

## T-0210 — Optional Grafana `$tag` variable

**Status:** open  
**Depends on:** T-0205  
**Decisions:** DEC-0103  
**Architecture slice:** US-0020-S7  
**Priority:** P2 optional

### Description

`grafana/provisioning/dashboards/analytics/subscriptions.json`:

- Add `$tag` template variable sourced from `operator_tags.slug`
- Filter panel SQL when variable set

Does not block MVP — SPA `?tag=` filter sufficient per DEC-0103.

### Done when

- [ ] Grafana variable provisioned
- [ ] Panel SQL respects `$tag` when non-empty
- [ ] Operator reload Grafana provisioning documented

---

## Sequencing

```text
1. T-0198 (migration)
2. T-0199 (discover service)
3. T-0200 (discover UI, after T-0199)
4. T-0201 (confirm, after T-0199)
5. T-0202 (majority compute, after T-0201)
6. T-0203 (majority UI, after T-0202)
7. T-0204 (tag CRUD, after T-0198)
8. T-0205 (assign/filter, after T-0204)
9. T-0206 (tag UI, after T-0205)
10. T-0207 → T-0208 (after integrations)
11. T-0209 (after T-0208)
12. T-0210 optional after T-0205
```

Dependency graph:

```text
T-0198
 ├── T-0199 ── T-0200
 │         └── T-0201 ── T-0202 ── T-0203
 └── T-0204 ── T-0205 ── T-0206
T-0207 → T-0208 → T-0209
T-0210 (optional, after T-0205)
```

## Acceptance map

| Row | Tasks | Verify |
|-----|-------|--------|
| AC-1 | T-0199, T-0200, T-0209 | Discover search: account + payee + interval; cap 50 |
| AC-2 | T-0201, T-0209 | Manual confirm without pending-only path |
| AC-3 | T-0202, T-0203, T-0209 | Majority category badge + tie-break tooltip |
| AC-4 | T-0204, T-0205, T-0206, T-0209 | Tag CRUD; multi-assign; list filter |
| AC-5 | T-0198, T-0202, T-0204, T-0209 | Product DB only; no Firefly write-back |
| AC-6 | T-0207, T-0208, T-0209 | US-0003/US-0008 regression; OIDC smoke |

## Notes

- **P2 optional:** T-0210 does not block MVP — SPA tag filter sufficient per DEC-0103.
- **Detection unchanged:** No edits to `DetectionPipeline::run_candidates` thresholds or skip order.
- **DEC-0085 preserved:** Manual confirm uses same merge semantics as auto-detected confirms (BUG-0015).
