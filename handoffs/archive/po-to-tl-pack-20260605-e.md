# PO to TL archive pack (2026-06-05)

- Rollover trigger: `PO_TO_TL_HOT_MAX_LINES=500, PO_TO_TL_HOT_MAX_SECTIONS=40`
- Source: `handoffs/po_to_tl.md`
- Archived units (oldest first, contiguous prefix): 1
- Retained units in hot file: 7
- First archived heading: `## architecture-20260605-bug0012 — BUG-0012 component monthly bucket attribution`
- Last archived heading: `## architecture-20260605-bug0012 — BUG-0012 component monthly bucket attribution`
- Verification tuple (mandatory):
  - archived_body_lines=45
  - retained_body_lines=485

---

## architecture-20260605-bug0012 — BUG-0012 component monthly bucket attribution

**From:** Tech Lead  
**To:** Dev / Sprint-plan  
**Date:** 2026-06-05  
**Bug:** BUG-0012  
**Orchestrator:** `auto-20260605-bug0012-001`  
**Next phase:** `/sprint-plan`

### Summary

Architecture freezes **DEC-0067** component-level monthly attribution for AG/AH. Replace net-delta `categorize_delta` with per-component `monthly_map` accumulation; daily `balance += delta` unchanged. `RecurringPattern.category_id` from `RecurrenceGroup.category_ids` mode + subscription override payee lookup. Rolling residual → **Variable** (positive and negative). Unmapped categories → **Variable**. Income only via categorized recurring → TOML `map_category`. Operator TOML checklist for non-English Firefly labels (D1).

**Architecture ref:** `docs/engineering/architecture.md` § **BUG-0012**  
**Decision:** **DEC-0067**  
**Research:** **R-0063**

### Frozen contracts

| Topic | Resolution |
|-------|------------|
| Attribution model | Component-level `monthly_map` — **not** net-delta |
| Rolling `daily_rate` | Variable bucket always |
| Recurring due | `category_id` → `category_names` → `map_category` |
| Positive rolling → Income | **Rejected** |
| Daily balance / horizons | Unchanged |
| US-0015 / US-0013 | Out of scope |

### Sprint-plan input (5 tasks)

AH1 → AG1 → T1 → D1 → V1 (~4–6 tasks; `/quick` candidate)

### Triad check (architecture phase)

| Surface | Check | Result |
|---------|-------|--------|
| `docs/engineering/architecture.md` | § BUG-0012 | pass (rollover US-0001..US-0007 → architecture-archive packs) |
| `decisions/DEC-0067.md` | Component attribution | pass |
| `docs/product/acceptance.md` | AG/AH unchanged | pass |
| `docs/engineering/research.md` | R-0063 linked | pass |

`triad_hot_surface`: architecture rollover 2 packs; § BUG-0012 prepended; retained_lines=2664/3000

---

