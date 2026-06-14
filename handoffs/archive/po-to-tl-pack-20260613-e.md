# PO to TL archive pack (2026-06-13)

- Rollover trigger: `PO_TO_TL_HOT_MAX_LINES=500, PO_TO_TL_HOT_MAX_SECTIONS=40`
- Source: `handoffs/po_to_tl.md`
- Archived units (oldest first, contiguous prefix): 3
- Retained units in hot file: 40
- First archived heading: `## Operator report`
- Last archived heading: `## Fix direction (discovery)`
- Verification tuple (mandatory):
  - archived_body_lines=43
  - retained_body_lines=357

---

## Operator report

> Income steht **0.00**, im Chart passt es — Summe abgebildet (~€3000 Balken).

**Screenshot:** `handoffs/evidence/bug0026-forecast-income-card-zero-20260613.png`

## Root cause (intake — high confidence)

| Component | Behavior |
|-----------|----------|
| `ForecastPage.tsx` | `monthlySummary = series[0]` — **no month label** |
| `MonthlyChart` | Renders **full** monthly series |
| Live API | `series[0]` **2026-06** income **0.00**; **2026-07+** income **3266.16** |

**Not BUG-0012** — backend projects income correctly from month 2; cards show wrong/unlabeled slice.

## Fix direction (discovery)

Label summary cards with month; use current/next full month or sync with chart selection — **frontend-only**, **DEC-0089** unchanged.

**Acceptance:** **(BZ)**, **(CA)** — **Recommended next phase:** `/discovery`

---

# BUG-0025 operator evidence update — Category spending trend (2026-06-13)

**Bug:** BUG-0025 **Surface:** `/forecast` → **Category spending trend** → **Wohnen - Stromkosten**

**Screenshot:** `handoffs/evidence/bug0025-category-spending-trend-stromkosten-20260613.png`

| Observation | Detail |
|-------------|--------|
| Visible data | **Only 2026-05** bar (~€465 outflow) |
| Empty months | **2025-07 … 2026-04**, **2026-06** at **€0** |
| Summary cards | Highest/Lowest = **2026-05**; MoM **−100%** |
| Mirror API | Category **146** expense-series: **4** txs, all **2026-05** |

**Verdict:** Chart reflects mirror — missing months = **sync ingest gap** (likely **DEC-0002** backdated window), not UI rendering bug.

**Next:** `/discovery` — compare Firefly Strom tx dates per month vs mirror; test cursor reset / full re-sync.

---

