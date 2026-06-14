# Tasks — Q0034 (BUG-0025)

**Bug:** BUG-0025  
**Task count:** 7 mandatory (7/12 under `SPRINT_MAX_TASKS=12`; no split)  
**Sprint-plan ref:** `sprint-plan-20260614-bug0025-q0034`

## Architecture → sprint mapping

| Architecture ID | Disposition | Gate |
|-----------------|-------------|------|
| **B1** | Task **B1** | GATE-OVERLAP-1 |
| **B2** | Task **B2** | GATE-SYNC-UX-1 |
| **F1** | Task **F1** | GATE-SYNC-UX-1 |
| **D1** | Task **D1** | GATE-REMED-1 |
| **T1** | Task **T1** | GATE-TEST-1 |
| **G1** | Task **G1** | — |
| **BW/BX/BY runtime** | Task **V1** | — |
| **F3** (HomePage) | **Out of scope** (optional P1) | — |

## Execute order

```text
B1 ∥ B2
  → F1 (needs B2)
  → D1 (after B1)
  → T1 (needs B1)
  → G1
  → operator: BACKEND_REBUILD + FRONTEND_DEPLOY
  → V1 verify-work
```

## Acceptance traceability

| Row | Tasks | Verify |
|-----|-------|--------|
| **BW** | B1, T1, G1, V1 | Manual Sync now → multi-month Stromkosten; expense-series category **146** bars per month |
| **BX** | B1, D1, F1, G1, V1 | Manual Full ingest or callout + runbook DEC-0002 + cursor reset |
| **BY** | B2, F1, G1, V1 | last_firefly_run hero; exchange secondary; Sync now → manual in history |

| ID | Title | Est. | Status | Acceptance | Gate |
|----|-------|------|--------|------------|------|
| B1 | Manual 365d lookback wiring | 2h | open | **BW**, **BX** | GATE-OVERLAP-1 |
| B2 | last_firefly_run API split | 1.5h | open | **BY** | GATE-SYNC-UX-1 |
| F1 | SyncStatusPage hero + callout | 2h | open | **BX**, **BY** | GATE-SYNC-UX-1 |
| D1 | Runbook remediation | 1h | open | **BX** | GATE-REMED-1 |
| T1 | Integration backdated-window repro | 3h | open | **BW** | GATE-TEST-1 |
| G1 | Automated gate | 1h | open | all | — |
| V1 | verify-work + OIDC smoke | 2h | open | all | — |

---

## B1 — Manual 365d lookback wiring

**Status:** open  
**Depends on:** —  
**Estimate:** 2h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0025 **BW**, **BX** — **GATE-OVERLAP-1**

### Description

Implement frozen sync start contract in `backend/src/firefly/mod.rs`:

```rust
// MANUAL_LOOKBACK_DAYS: i64 = 365 (module-local constant)
sync_transactions(client, pool, overlap_days, trigger):
  if trigger == "manual":
    start_date = (Utc::now() - 365 days).date()
  else if watermark present:
    start_date = (watermark - overlap_days).date()  // unchanged scheduled path
  else:
    start_date = (Utc::now() - 365 days).date()    // cold-start unchanged
```

Wire `backend/src/sync/mod.rs` `execute_run` to pass `trigger` into `sync_transactions`. Update test callers in `firefly_readonly_test.rs`.

**Files:** `backend/src/firefly/mod.rs` L368–415; `backend/src/sync/mod.rs` L196–230

### Done when

- [ ] `trigger=manual` uses 365d lookback by transaction date
- [ ] `trigger=scheduled` unchanged (`watermark − overlap_days`)
- [ ] All callers compile with new signature

### Verification

Unit/integration compile; T1 asserts ingest behavior.

---

## B2 — last_firefly_run API split

**Status:** open  
**Depends on:** —  
**Estimate:** 1.5h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0025 **BY** — **GATE-SYNC-UX-1**

### Description

Extend `SyncStatusResponse` (additive):

```rust
pub struct SyncStatusResponse {
    pub last_run: Option<SyncRunRow>,           // unchanged
    pub last_firefly_run: Option<SyncRunRow>,   // NEW
    // ... existing fields
}
```

Add `latest_firefly_run()` query: `WHERE trigger IN ('manual', 'scheduled') ORDER BY started_at DESC LIMIT 1`.

Update `frontend/src/lib/api.ts` with `last_firefly_run: SyncRun | null`.

**Files:** `backend/src/sync/mod.rs`; `frontend/src/lib/api.ts`

### Done when

- [ ] API returns `last_firefly_run` distinct from exchange-only `last_run`
- [ ] Frontend types updated

### Verification

curl `GET /api/v1/sync/status` after mixed run history.

---

## F1 — SyncStatusPage hero + DEC-0002 callout

**Status:** open  
**Depends on:** B2  
**Estimate:** 2h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0025 **BX**, **BY** — **GATE-SYNC-UX-1**

### Description

In `frontend/src/pages/SyncStatusPage.tsx`:

1. Hero primary: **Last Firefly sync:** ← `last_firefly_run?.finished_at` or **Never**
2. Trigger badge pill: `manual` → **Manual**; `scheduled` → **Scheduled**
3. Secondary **Last exchange sync:** when `last_run.trigger ∈ {scheduled_exchanges, manual_exchanges}` and newer than Firefly run
4. Info callout: DEC-0002 7-day overlap by transaction date; backdated imports need **Sync now** (365d) or runbook cursor reset; link `docs/engineering/runbook.md`
5. **Sync now** button unchanged; history table keeps raw `trigger` column

**Files:** `frontend/src/pages/SyncStatusPage.tsx` L88–92 hero block + new callout

### Done when

- [ ] Hero never shows exchange-only timestamp as Firefly sync
- [ ] Callout visible with runbook link
- [ ] Exchange secondary line when applicable

### Verification

Browser `/sync` after scheduled_exchanges run — hero unchanged; after manual Full — hero updates.

---

## D1 — Runbook remediation

**Status:** open  
**Depends on:** B1  
**Estimate:** 1h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0025 **BX** — **GATE-REMED-1**

### Description

Add section to `docs/engineering/runbook.md`:

| Topic | Content |
|-------|---------|
| Symptom | Category trend / expense-series missing months after Firefly backdated import |
| Cause | DEC-0002 — Firefly `start` filters by transaction date; scheduled overlap 7d |
| Fix ≤365d | **Sync now** on `/sync` (manual Full — 365d lookback post-fix) |
| Fix >365d | `DELETE FROM sync_cursors WHERE entity_type = 'transactions';` then manual Full |
| Safety | Upsert by Firefly `id` — no duplicate rows |

### Done when

- [ ] Runbook section published with anchor linkable from F1 callout

### Verification

Doc review; link from Sync Status callout resolves.

---

## T1 — Integration backdated-window repro

**Status:** open  
**Depends on:** B1  
**Estimate:** 3h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0025 **BW** — **GATE-TEST-1**

### Description

Extend backend integration test harness:

| Step | Assert |
|------|--------|
| Seed watermark + mirror tx **outside** `watermark − 7d` | Not ingested on `trigger=scheduled` |
| Same fixture + `trigger=manual` (post-fix) | Row present in `transactions` mirror |
| Optional | Cursor delete + scheduled Full ingests via 365d cold-start |

**Files:** new or extend `backend/tests/` sync transaction window test

### Done when

- [ ] Test fails pre-fix / passes post-fix behavior documented
- [ ] `cargo test` green

### Verification

`cargo test` includes new case.

---

## G1 — Automated gate

**Status:** open  
**Depends on:** B1, B2, F1, D1, T1  
**Estimate:** 1h  
**Acceptance hook:** BUG-0025 **BW**, **BX**, **BY**

### Description

Run and record in `sprints/quick/Q0034/progress.md`:

1. `cargo test` → PASS
2. `npm test` → PASS
3. `npm run build` → PASS
4. `git diff --stat` blast radius matches frozen file list

### Done when

- [ ] All checks PASS, recorded in progress.md

### Verification

Test output pasted in progress.md.

---

## V1 — verify-work BW/BX/BY + OIDC smoke

**Status:** open  
**Depends on:** G1 + BACKEND_REBUILD + FRONTEND_DEPLOY  
**Estimate:** 2h  
**Acceptance hook:** `docs/product/acceptance.md` BUG-0025 **BW**, **BX**, **BY**

### Description

Populate `sprints/quick/Q0034/uat.md` and `uat.json` after deploy on localhost:18080 (and optional omniflow OIDC):

1. **BW-API/UI** — Manual Sync now → expense-series category **146** + `/forecast` Category spending trend multi-month bars
2. **BX-UI/DOC** — `/sync` DEC-0002 callout; runbook section present
3. **BY-API/UI/HIST** — `last_firefly_run` hero; exchange secondary; history `trigger` distinguishes run kinds
4. **OIDC-1** — `/sync`, `/forecast`, sync trigger smoke

**Operator gates:** **BACKEND_REBUILD** + **FRONTEND_DEPLOY**

### Done when

- [ ] Rows **BW**, **BX**, **BY** probed per acceptance.md matrix
- [ ] `uat.md` and `uat.json` populated with results
