# PO to TL archive pack (2026-06-14)

- Rollover trigger: `PO_TO_TL_HOT_MAX_LINES=650, PO_TO_TL_HOT_MAX_SECTIONS=60`
- Source: `handoffs/po_to_tl.md`
- Archived units (oldest first, contiguous prefix): 15
- Retained units in hot file: 60
- First archived heading: `## Research summary`
- Last archived heading: `## Recommended next phase`
- Verification tuple (mandatory):
  - archived_body_lines=112
  - retained_body_lines=506

---

## Research summary

[R-0097 §1–9](docs/engineering/research.md#r-0097--bug-0025-firefly-category-transactions-not-updating-stromkosten) fulfilled. H1 **CONFIRMED** (DEC-0002 backdated skip — mirror only 2026-05 for category 146; next `start ≈ 2026-06-06`). H2 **PARTIAL CONFIRMED** (`last_run=scheduled_exchanges` while last Full 11:53). H3 **CONFIRMED** (Category spending trend surface). Firefly direct oracle deferred (no FIREFLY_URL in probe env); gap analysis sufficient.

## Frozen gates

| Gate | Research verdict |
|------|------------------|
| **GATE-OVERLAP-1** | **A + B** — Sync Status / runbook documents DEC-0002 backdated limits + cursor-reset SQL; **manual Full** uses **365d lookback**; scheduled keeps watermark−7d |
| **GATE-SYNC-UX-1** | Split **`last_firefly_run`** from exchange-only; hero **"Last Firefly sync"** + trigger badge; secondary exchange line when newer |
| **GATE-REMED-1** | Runbook: `DELETE FROM sync_cursors WHERE entity_type='transactions'`; admin API deferred |
| **GATE-TEST-1** | Integration: tx dated before incremental `start` → skip pre-fix, ingest after manual lookback or cursor reset |
| **GATE-DEC-1** | **Extend DEC-0002** (manual-trigger exception); no new DEC unless architecture splits contracts |

## BX / BW closure

- **BW** requires code (**manual 365d lookback**) — document-only insufficient.
- **BX** requires doc tier + ingest path (manual lookback or documented cursor reset for >365d).

## Recommended execute shape

**5–7 tasks** — backend manual lookback + status API fields; Sync Status UI + DEC-0002 callout; runbook; integration test; V1 smoke. Likely `/quick` or small sprint.

## Recommended next phase

`/architecture` — formalize sync start contract, `SyncStatusResponse` shape, acceptance trace BW/BX/BY, size sprint.

---

# discovery-20260613-bug0025 — BUG-0025 Firefly Stromkosten mirror lag

**From:** PO **To:** Tech Lead **Bug:** BUG-0025 **Run:** `auto-20260613-bug0025`
**Date:** 2026-06-13 **Next phase:** `/research` (role: tech-lead)
**Intake evidence:** `handoffs/intake_evidence/intake-20260613-firefly-stale-mirror.json` (read-only)

## Discovery summary

Code audit + live localhost probe confirm intake hypotheses: operator symptom is **`/forecast` Category spending trend** for **Wohnen - Stromkosten** showing **one month with data** (2026-05), not a chart rendering defect. Mirror, `GET /api/v1/categories/expense-series?category_id=146`, and operator screenshot **align** (4 txs, ~€465 outflow, all dated **2026-05**). Primary suspect is **DEC-0002 incremental window** — `sync_transactions` queries Firefly `/api/v1/transactions?start=` where `start = last_successful_sync_at − overlap_days` (default **7**), filtering by **transaction date** not edit time; bulk **backdated** Strom imports outside that window are **skipped** while sync reports success. Secondary UX risk: **`GET /api/v1/sync/status`** `last_run` currently surfaces the latest run of **any** kind — probe shows **`scheduled_exchanges`** success at 12:53 while last **Full** Firefly run was **`scheduled`** at 11:53 — so "Last sync" header can mislead operators who did not trigger manual Full sync.

## Hypothesis resolution

| ID | Verdict | Key evidence |
|----|---------|--------------|
| **H1** | **LIKELY PRIMARY** | `sync_transactions` L373–381: watermark `2026-06-13 11:53:28Z` → next `start ≈ 2026-06-06`; mirror `category_id=146` has **4** rows **only** in `2026-05`; operator months **2025-07…2026-04** at €0 match **not ingested**, not filtered in UI |
| **H2** | **PARTIAL CONFIRMED** | Scheduler alternates Full (`scheduled`/`manual`) vs `scheduled_exchanges` when Firefly not due (`sync/mod.rs` L124–142); **Sync now** → `trigger_manual` → `RunMode::Full` ✓; **Last sync** header uses latest run regardless of trigger — confusion risk; history table **does** show `trigger` column |
| **H3** | **CONFIRMED** | Operator surface = `/forecast` **Category spending trend** (`US-0018` / **DEC-0088**); "one transaction" = **one month with bars**; expense-series SQL aggregates mirror `transactions` by month — **not** a frontend bug |

## Sub-defects confirmed

| AC | Verdict | Key evidence |
|----|---------|--------------|
| **BW** | **CONFIRMED GAP** | expense-series + screenshot: **only 2026-05** outflow (~€465); 11 other months €0 despite operator claim of many Strom txs in Firefly |
| **BX** | **LIKELY GAP** | DEC-0002 overlap mechanism + mirror date distribution; no Sync Status / runbook copy explaining backdated-import skip or cursor-reset remediation |
| **BY** | **PARTIAL** | Manual **Sync now** = Full Firefly ingest (code + `manual` runs in history); sync history distinguishes `manual`/`scheduled` vs `scheduled_exchanges`/`manual_exchanges`; **Last sync** summary does **not** distinguish Firefly vs exchange-only |

## Root-cause chain (code + live)

| Layer | Finding |
|-------|---------|
| **Ingest** | `backend/src/firefly/mod.rs` L368–415: `start_date = watermark − overlap_days`; Firefly `start` param filters by **tx date**; upsert by Firefly id; cursor `last_successful_sync_at` updated on every Full run |
| **Watermark probe** | `sync_cursors` transactions: `records_synced=12`, `last_successful_sync_at=2026-06-13 11:53:28+00` |
| **Mirror probe** | `transactions` total **939** (dates **2025-06-05 … 2026-06-11**); `category_id=146` → **4** rows, all **2026-05-11…13** (descriptions: Strom Teilbetrag, go green energy, ABSCHLAG 05/26, Strom Diverses) |
| **API** | `GET /api/v1/categories/expense-series?category_id=146` — 4 txs, 2026-05 only; matches chart |
| **Scheduler** | `sync/mod.rs` L104–142: tick max(firefly_interval, exchange_interval); if Firefly not due → `scheduled_exchanges` only |
| **Sync Status UI** | `SyncStatusPage.tsx` L88–92 "Last sync" from `last_run` (any trigger); L173–194 history table shows `trigger` raw string |
| **BUG-0006** | **RULED OUT** — category_id ingest path works; 146 rows present with correct category binding |

## UX / remediation recommendation (PO)

| Option | Notes | PO preference |
|--------|-------|---------------|
| **A** | **Document + operator remediation**: runbook / Sync Status callout for DEC-0002 — backdated bulk imports need **cursor reset** or **full lookback**; expose when last Full Firefly sync ran vs exchange-only | **Preferred first** — matches existing contract; low blast radius |
| **B** | **Manual Full sync widens lookback** (e.g. 365d on `manual` trigger only) | Fixes operator pain without global overlap change; research must size ingest cost |
| **C** | Increase default `overlap_days` globally | Simple but does not fix deep backfills; may inflate per-run fetch |
| **D** | Sync Status UX only — relabel "Last sync" → "Last Firefly sync" / "Last exchange sync" | Addresses H2 confusion; does not fix missing mirror rows |

**Vision tension:** DEC-0002 intentionally bounds incremental fetch (R-0089). Discovery confirms mechanism behaves as designed; defect is **operator expectation vs contract** plus **status UX**. Research should decide fix-vs-document split for **BX**.

## Architecture gates (research carry)

| Gate | Question | PO default |
|------|----------|------------|
| **GATE-OVERLAP-1** | Backdated import policy | Document + cursor-reset path **or** manual-trigger full lookback — not silent skip |
| **GATE-SYNC-UX-1** | Status surface semantics | Separate **Last Firefly sync** from exchange-only; badge `trigger` on summary card |
| **GATE-REMED-1** | Operator cursor reset | Runbook step + optional admin API/CLI to reset `sync_cursors.transactions` |
| **GATE-TEST-1** | Repro harness | Seed Firefly (or wiremock) txs dated **before** `start` window → assert ingest after manual Full |
| **GATE-DEC-1** | New DEC? | Extend **DEC-0002** doc/UX only unless manual lookback changes contract |

## Acceptance rows (unchanged intent)

- **(BW)** Multi-month Stromkosten bars after manual Full sync when Firefly holds rows in those months — not 2026-05-only
- **(BX)** Backdated/bulk imports ingest **or** documented DEC-0002 limitation + remediation — not silent omission with misleading success
- **(BY)** Manual **Sync now** = Full Firefly; history distinguishes full vs exchange-only runs; summary must not imply Firefly synced when only exchanges ran

## Research questions (carry from R-0097)

1. **Firefly oracle:** Compare Strom tx **dates per month** in Firefly III vs mirror SQL for `category_id=146` — confirm H1 with operator ledger (read-only GET; names-only env)
2. **Repro:** Import/backdate txs to **2025-08** (outside 7-day window) → manual Full → assert mirror count
3. **Manual lookback sizing:** Cost/latency of 365d fetch on 939-tx profile; pagination behavior
4. **Cursor reset:** Safe operator procedure (SQL vs API) without duplicate/orphan rows
5. **BX fix split:** Document-only vs code change threshold for acceptance **BX**

## Related work

**BUG-0006** DONE (category ingest); **BUG-0002**/**BUG-0004** (sync pipeline); **DEC-0002**; **US-0018** / **DEC-0088**; [R-0097](docs/engineering/research.md#r-0097--bug-0025-firefly-category-transactions-not-updating-stromkosten) (stub → discovery evidence); [R-0089](docs/engineering/research.md#r-0089--bug-0019-grafana-cashflow-zeros-account_id-default--sync-entity-counts-per-run-cursor) (overlap semantics)

## Recommended next phase

`/research` — Firefly-vs-mirror month oracle, repro backdated window, freeze GATE-OVERLAP-1 / GATE-SYNC-UX-1, size fix vs document for **BX**.

---

