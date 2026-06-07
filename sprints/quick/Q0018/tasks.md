# Tasks — Q0018 (BUG-0008)

**Bug:** BUG-0008  
**Task count:** 12 (at `SPRINT_MAX_TASKS=12`; no split)  
**Sprint-plan ref:** `sprint-plan-20260608-q0018-bug0008`

## Architecture → sprint mapping

| Architecture ID | Disposition | Notes |
|-----------------|-------------|-------|
| **W1** | Task **W1** | `fingerprint` column + partial unique index + backfill dedupe |
| **W2** | Task **W2** | `insert_alert` → `upsert_alert` ON CONFLICT |
| **W3** | Task **W3** | Emit only on new pending or tier increase |
| **W4** | Task **W4** | `GET /api/v1/subscriptions/alerts/unread-count` |
| **W5** | Task **W5** | Mark-read orphans on confirm/reject/inactive |
| **W6** | Task **W6** | Banner + toast from unread-count API |
| **W7** | Task **W7** | Dedup, reconciled count, lifecycle tests |
| **X1** | Task **X1** | SEPA token strip, entity suffix collapse |
| **X2** | Task **X2** | Transfer-type counterparty priority guard |
| **X3** | Task **X3** | `detection_window_days` 365 → 730 |
| **X4** | Task **X4** | Forecast + subscription regression integration tests |
| **V1** | Task **V1** | verify-work omniflow smoke rows W/X |

## Execute order

```text
W1 → W2 → W3 → W4 → W5 → W6 → W7 → X1 → X2 → X3 → X4
→ deploy backend + frontend
→ V1 verify-work
```

**Sequencing:** W1–W3 before X1 (W-before-X mandatory). X3 independent of W4–W6 once W3 complete.

## Acceptance traceability

| Row | Tasks | Verify |
|-----|-------|--------|
| **W** | W1–W7, V1 | Reconciled unread-count vs pending; banner uses API not list length |
| **X** | X1–X4, V1 | Patterns > 12 baseline; no alert spam invariant |
| Regression | W7, X4, V1 | OIDC + bundled-firefly deploy smoke |

| ID | Title | Est. | Status | Acceptance |
|----|-------|------|--------|------------|
| W1 | Fingerprint migration + backfill dedupe | 3h | open | **W** |
| W2 | `upsert_alert` repository | 2h | open | **W** |
| W3 | Detection emit gate | 2h | open | **W** |
| W4 | Unread-count API route | 2h | open | **W** |
| W5 | Orphan lifecycle hooks | 1.5h | open | **W** |
| W6 | Frontend banner + toast | 2h | open | **W** |
| W7 | Backend dedup + lifecycle tests | 3h | open | **W** regression |
| X1 | Payee normalization | 3h | open | **X** |
| X2 | Transfer counterparty priority | 2h | open | **X** |
| X3 | `detection_window_days` 730 | 0.5h | open | **X** |
| X4 | Forecast + subscription integration tests | 2h | open | **X** regression |
| V1 | verify-work omniflow smoke | 1h | open | **W**, **X** |

---

## W1 — Fingerprint migration + backfill dedupe

**Status:** open  
**Depends on:** —  
**Estimate:** 3h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0008 **W**

### Description

Add `fingerprint TEXT` column to `subscription_alerts`. Backfill fingerprints per DEC-0071 contract (`sub_alert:{type}:{pattern_id}[:suffix]`). Dedupe duplicates — keep newest unread per fingerprint; mark-read orphans. Set NOT NULL. Create partial unique index `(fingerprint) WHERE read_at IS NULL`.

**Files:** `backend/migrations/`, `backend/src/subscriptions/repository.rs`

### Done when

- [ ] Migration adds column, backfills, dedupes, sets NOT NULL
- [ ] Partial unique index `subscription_alerts_unread_fingerprint` present
- [ ] Backfill reduces duplicate unread rows (expect ~77 orphans from 83→6 steady state)

---

## W2 — `upsert_alert` repository

**Status:** open  
**Depends on:** W1  
**Estimate:** 2h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0008 **W**

### Description

Replace bare `insert_alert` with `upsert_alert` using ON CONFLICT on partial unique index. On conflict update `body`, `sync_run_id`, `created_at`.

**Files:** `backend/src/subscriptions/repository.rs`

### Done when

- [ ] `upsert_alert` implements frozen SQL contract per DEC-0071 §1
- [ ] No duplicate unread fingerprints on repeated calls
- [ ] Fingerprint computed per alert_type rules

---

## W3 — Detection emit gate

**Status:** open  
**Depends on:** W2  
**Estimate:** 2h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0008 **W**

### Description

Gate `upsert_alert` calls in detection pipeline: emit only when pattern upsert returns **new pending** OR confidence tier **increased**. Skip unchanged pending patterns on routine sync.

**Files:** `backend/src/subscriptions/detection.rs`

### Done when

- [ ] No alert emitted on unchanged pending pattern resync
- [ ] Alert emitted on genuinely new pending pattern
- [ ] Tier increase still fires alert

---

## W4 — Unread-count API route

**Status:** open  
**Depends on:** W3  
**Estimate:** 2h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0008 **W**

### Description

Add `GET /api/v1/subscriptions/alerts/unread-count` returning reconciled semantics per DEC-0071 §2: `unread_total`, `unread_new_detection`, `unread_price_change`, `pending_patterns`, `reconciled`, `reconciliation_note`. Actionable unread joins unread `new_detection` to **pending** patterns.

**Files:** `backend/src/subscriptions/routes.rs`, `backend/src/subscriptions/service.rs`

### Done when

- [ ] Response schema matches DEC-0071 frozen JSON
- [ ] `reconciled: true` when counts align and JOIN guard passes
- [ ] Additive route only — no `list_patterns` filter changes (R-0065)

---

## W5 — Orphan lifecycle hooks

**Status:** open  
**Depends on:** W3  
**Estimate:** 1.5h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0008 **W**

### Description

On confirm, reject, and mark_inactive: mark-read unread alerts for `pattern_id` (all alert types when pattern terminal).

**Files:** `backend/src/subscriptions/service.rs`

### Done when

- [ ] confirm/reject mark-read unread `new_detection` for pattern
- [ ] mark_inactive mark-reads all unread alerts for pattern
- [ ] price_change / interval_change orphans cleaned on terminal lifecycle

---

## W6 — Frontend banner + toast

**Status:** open  
**Depends on:** W4  
**Estimate:** 2h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0008 **W**

### Description

Wire `/subscriptions` banner and post-sync toast to unread-count API. Banner uses `unread_new_detection`; subtitle when `!= pending_patterns`. Toast fires on sessionStorage delta of `unread_new_detection`. Header bell unchanged (US-0005-only).

**Files:** `frontend/src/pages/SubscriptionsPage.tsx`

### Done when

- [ ] Banner count from unread-count API — not `alerts.length`
- [ ] Post-sync toast uses delta semantics
- [ ] Header bell badge unchanged

---

## W7 — Backend dedup + lifecycle tests

**Status:** open  
**Depends on:** W1, W2, W3, W4, W5  
**Estimate:** 3h  
**Acceptance hook:** architecture § Test strategy (W regression)

### Description

Unit/integration tests: migration backfill dedupes; ON CONFLICT upsert; no alert on unchanged resync; unread-count reconciled semantics; confirm/reject mark-read orphans.

**Files:** `backend/tests/` or module `#[cfg(test)]`

### Done when

- [ ] All architecture § Test strategy W checks have automated coverage
- [ ] `cargo test` PASS (or documented DATABASE_URL prerequisite)

---

## X1 — Payee normalization

**Status:** open  
**Depends on:** W3  
**Estimate:** 3h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0008 **X**

### Description

Extend `payee_key()` in recurrence normalize: strip SEPA `SVWZ+`/reference tokens, card suffixes; collapse legal-entity suffixes (`GmbH`, `AB`).

**Files:** `backend/src/recurrence/normalize.rs`

### Done when

- [ ] SEPA fixture strings merge under single payee key
- [ ] Legal suffix collapse applied
- [ ] Unit tests for normalization rules

---

## X2 — Transfer counterparty priority

**Status:** open  
**Depends on:** X1  
**Estimate:** 2h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0008 **X**

### Description

For bank-transfer-shaped descriptions (`SVWZ|UEBERWEISUNG|Lastschrift` regex guard), prefer `counterparty_name` before full memo in `extract_payee_source` / grouping.

**Files:** `backend/src/recurrence/group.rs`, `backend/src/subscriptions/detection.rs`

### Done when

- [ ] Transfer-type guard regex applied
- [ ] Counterparty preferred for transfer-shaped memos only
- [ ] Non-transfer paths unchanged

---

## X3 — `detection_window_days` 730

**Status:** open  
**Depends on:** — (after W3 in deploy order)  
**Estimate:** 0.5h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0008 **X**

### Description

Change `detection_window_days` from 365 to 730 in `backend/config/default.toml`. Config loader unchanged.

**Files:** `backend/config/default.toml`

### Done when

- [ ] TOML value 730
- [ ] Config loads correctly in detection pipeline

---

## X4 — Forecast + subscription integration tests

**Status:** open  
**Depends on:** X1, X2, X3  
**Estimate:** 2h  
**Acceptance hook:** architecture § Test strategy (X regression)

### Description

Integration tests: SEPA fixture merges; forecast recurring detection unaffected or improved after payee_key changes (DEC-0013 shared core guard).

**Files:** `backend/tests/` or module `#[cfg(test)]`

### Done when

- [ ] X1–X2 normalization fixtures pass
- [ ] Forecast regression guard passes
- [ ] No alert spam invariant testable post-W3

---

## V1 — verify-work omniflow smoke

**Status:** open  
**Depends on:** W6, X4, deploy  
**Estimate:** 1h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0008 **W**, **X**

### Description

Prepare `sprints/quick/Q0018/uat.md` smoke checklist. After backend + frontend deploy, probe `financegnome.omniflow.cc`:

1. **W:** unread-count API reconciled; banner matches API; confirm/reject cleans orphans
2. **X:** pattern count > 12 baseline; no alert spam after resync
3. Regression: OIDC + bundled-firefly deploy checks

**Files:** `sprints/quick/Q0018/uat.md`, `docs/user-guides/BUG-0008.md` (optional cross-ref)

### Done when

- [ ] Row **W**: reconciled unread-count PASS
- [ ] Row **X**: improved recall without spam PASS
- [ ] Deploy regression footer PASS

**Operator gate:** Backend + frontend deploy after W1–X4+W7 before V1 runtime probes.
