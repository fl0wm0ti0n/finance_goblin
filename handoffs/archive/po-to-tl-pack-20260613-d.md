# PO to TL archive pack (2026-06-13)

- Rollover trigger: `PO_TO_TL_HOT_MAX_LINES=500, PO_TO_TL_HOT_MAX_SECTIONS=40`
- Source: `handoffs/po_to_tl.md`
- Archived units (oldest first, contiguous prefix): 4
- Retained units in hot file: 40
- First archived heading: `## Operator report`
- Last archived heading: `## Acceptance`
- Verification tuple (mandatory):
  - archived_body_lines=31
  - retained_body_lines=357

---

## Operator report

> Firefly-Änderungen erscheinen nicht in Finance Goblin — viele Transaktionen **Wohnen - Stromkosten** hinzugefügt, weiterhin nur **eine** sichtbar.

## Live probe (2026-06-13)

| Check | Result |
|-------|--------|
| `GET /api/v1/sync/entities` | **939** transactions |
| Category **146** expense-series | **4** tx (May 2026), not 1 |
| Recent sync triggers | Mix of `scheduled`, `manual`, **`scheduled_exchanges`** |
| `sync_transactions` | `start = last_successful_sync_at − overlap_days` (**DEC-0002**) |

## Hypotheses (discovery)

| ID | Hypothesis | Verify |
|----|------------|--------|
| **H1** | Backdated txs outside overlap window | Compare Firefly dates vs mirror; reset cursor repro |
| **H2** | Exchange-only sync mistaken for Firefly sync | Sync Status UX + `sync/runs` trigger labels |
| **H3** | UI shows 1 (month/pattern/filter) not mirror total | Operator surface + expense-series vs Firefly |

## Acceptance

- **(BW)** Category mirror match after Full sync
- **(BX)** Backdated/bulk ingest or documented remediation
- **(BY)** Full vs exchange-only sync clarity

**Recommended next phase:** `/discovery` (BUG-0025)

---

