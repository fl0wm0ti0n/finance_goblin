# Tasks — Q0023 (BUG-0015)

**Bug:** BUG-0015  
**Task count:** 5 (all P0 mandatory; < `SPRINT_MAX_TASKS=12`; no split)  
**Sprint-plan ref:** `sprint-plan-20260607-q0023-bug0015`

## Architecture → sprint mapping

| Architecture ID | Disposition | Notes |
|-----------------|-------------|-------|
| **AU1** | Task **AU1** | DEC-0084 card billing descriptor rules in `payee_key()` |
| **AU2** | Task **AU2** | DEC-0085/0086 maps + merge upsert + index migration |
| **AU3** | Task **AU3** | DEC-0085 detection skip+merge before pending upsert |
| **AU4** | Task **AU4** | DEC-0085 stale inactive by payee+interval; wire `run_detection` |
| **V1** | Task **V1** | verify-work rebuild smoke AU–AW |
| **H2 probe** | **Ops gate** | Operator SQL before Full sync — not sprint task |

## Execute order

```text
AU1 (normalize.rs)
  → AU2 (repository + migration)
  → AU3 ∥ AU4 (parallel after AU2 — detection paths)
  → single backend release deploy
  → operator: BACKEND_FRONTEND_DEPLOY
  → operator: confirm baseline + rebuild app only
  → operator: POSTGRES_PERSISTENCE_PROBE
  → operator: FULL_FIREFLY_SYNC
  → V1 verify-work
```

**Parallelism:** AU3 and AU4 may proceed in parallel after AU2; V1 blocked on deploy + operator gates.

## Acceptance traceability

| Row | Tasks | Verify |
|-----|-------|--------|
| **AU** | AU1, AU2, AU3, V1 | Confirmed Cursor/Apple in API + UI after rebuild |
| **AV** | AU1–AU4, V1 | No duplicate pending; merge or fingerprint skip |
| **AW** | AU3, V1 | Alert unread reconciles; no spurious new_detection |

| ID | Title | Est. | Status | Acceptance | Priority |
|----|-------|------|--------|------------|----------|
| AU1 | Card billing `payee_key` normalization | 2h | open | **AU**, **AV** | P0 |
| AU2 | Payee+interval maps + merge upsert | 4h | open | **AU**, **AV** | P0 |
| AU3 | Detection skip + merge path | 3h | open | **AU**, **AV**, **AW** | P0 |
| AU4 | Stale inactive by payee+interval | 2h | open | **AV** | P0 |
| V1 | verify-work rebuild smoke | 2h | open | **AU**–**AW** | P0 |

---

## AU1 — Card billing `payee_key` normalization

**Status:** open  
**Depends on:** —  
**Estimate:** 2h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0015 **AU**, **AV** — **DEC-0084**

### Description

Implement DEC-0084 card billing descriptor rules in `payee_key()`:

1. **Asterisk split** — token before `*` when present (`DBA*Plan` → `dba`)
2. **Comma memo** — leftmost segment before `,` (case-insensitive trim)
3. **Billing roots** — collapse `apple.com/bill`, `itunes.com`, `apple.com/bill itunes` → `apple`
4. **Domain tail** — known SaaS: strip `.com`, `/bill` suffix after left-segment extract

Rules must be conservative — shared `recurrence` module affects forecast + subscriptions (**DEC-0013**). Layer 2 catches residual drift.

**Files:** `backend/src/recurrence/normalize.rs` + unit tests in same module.

### Done when

- [ ] Cursor/Apple descriptor fixtures normalize to stable keys per R-0082
- [ ] Existing SEPA/DEC-0072 rules unchanged for non-card paths
- [ ] `cargo test` recurrence normalize paths PASS

---

## AU2 — Repository payee+interval maps and merge upsert

**Status:** open  
**Depends on:** AU1  
**Estimate:** 4h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0015 **AU**, **AV** — **DEC-0085**, **DEC-0086**

### Description

Implement DEC-0085/0086 repository contracts:

| Function | Purpose |
|----------|---------|
| `load_confirmed_payee_intervals()` | `HashMap<(payee_key, interval_days), ConfirmedRow>` |
| `load_rejected_payee_intervals()` | `HashSet<(payee_key, interval_days)>` |
| `merge_confirmed_pattern()` | UPDATE existing confirmed row; rotate fingerprint in-place |
| `interval_matches(stored, detected)` | ±3 day tolerance (**DEC-0086**) |

Add migration: `CREATE INDEX IF NOT EXISTS idx_subscription_patterns_payee_status ON subscription_patterns (payee_key, status)`.

**Merge contract:** refresh amounts/dates/display; set `fingerprint = newly_computed`; preserve `status = confirmed`, `confirmed_at`; no pending INSERT; no `new_detection` alert.

**Files:** `backend/src/subscriptions/repository.rs`, `backend/migrations/`.

### Done when

- [ ] Maps load confirmed/rejected by payee+interval with tolerance
- [ ] Merge updates same row by `id` — no UNIQUE violation
- [ ] Index migration applied
- [ ] `cargo test` subscriptions repository paths PASS

---

## AU3 — Detection skip + merge path

**Status:** open  
**Depends on:** AU2  
**Estimate:** 3h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0015 **AU**, **AV**, **AW** — **DEC-0085**

### Description

Update `run_candidates` per architecture:

```text
for each group:
  fingerprint = compute_fingerprint(...)
  if fingerprint in rejections or confirmed_fps: continue  // retain fast path
  if (payee_key, interval_days) in rejected_payee_intervals: continue
  if match confirmed_payee_intervals via interval_matches:
    merge_confirmed_pattern(...); continue
  else:
    upsert_pending_pattern(...)  // existing pending path
```

**`service.rs`:** Load payee+interval maps at run start alongside fingerprint sets.

**Files:** `backend/src/subscriptions/detection.rs`, `backend/src/subscriptions/service.rs`.

### Done when

- [ ] Confirmed payee+interval → merge; no pending row; no alert
- [ ] Rejected payee+interval → skip
- [ ] Exact fingerprint fast path retained
- [ ] Merge runs before pending upsert (order prevents duplicate INSERT)
- [ ] `cargo test` detection paths PASS

---

## AU4 — Stale inactive by payee+interval

**Status:** open  
**Depends on:** AU2  
**Estimate:** 2h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0015 **AV** — **DEC-0085**

### Description

Replace fingerprint-only `mark_stale_inactive` with payee+interval composite:

1. Build `active_payee_intervals` from current detection groups (normalized `payee_key` + `interval_days`)
2. For each confirmed pattern, if `(payee_key, interval_matches(interval_days))` not in active set **and** gap > `2 × interval_days`, call `mark_inactive`
3. Wire `mark_stale_inactive` into `run_detection` after candidates (currently defined but unwired)

**Files:** `backend/src/subscriptions/detection.rs`, `backend/src/subscriptions/service.rs`.

### Done when

- [ ] Drifted fingerprint does not incorrectly mark confirmed row stale
- [ ] `mark_stale_inactive` invoked after candidate pass in `run_detection`
- [ ] Gap threshold `2 × interval_days` enforced

---

## V1 — verify-work rebuild smoke AU–AW

**Status:** open  
**Depends on:** AU1–AU4 deploy + operator gates  
**Estimate:** 2h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0015 **AU**–**AW**

### Description

Prepare `sprints/quick/Q0023/uat.md` smoke checklist. After deploy + gates, probe `financegnome.omniflow.cc`:

- **AU:** Confirm Cursor + Apple → rebuild app → Full sync → still confirmed in API + `/subscriptions`
- **AV:** No duplicate pending rows for same merchant identity
- **AW:** Unread subscription alert count reconciles with pending tab
- **H2:** SQL probe before Full sync documents persistence outcome
- OIDC regression per acceptance AW footnote

### Done when

- [ ] Rows **AU**–**AW** probed per acceptance.md
- [ ] H2 probe outcome documented (execute vs ops-only)
- [ ] Operator gates documented in uat.md

**Operator gates:** BACKEND_FRONTEND_DEPLOY + POSTGRES_PERSISTENCE_PROBE + FULL_FIREFLY_SYNC before runtime probes.
