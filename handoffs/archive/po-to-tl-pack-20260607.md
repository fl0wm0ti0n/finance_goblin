# PO to TL archive pack (2026-06-07)

- Rollover trigger: `PO_TO_TL_HOT_MAX_LINES=500, PO_TO_TL_HOT_MAX_SECTIONS=40`
- Source: `handoffs/po_to_tl.md`
- Archived units (oldest first, contiguous prefix): 1
- Retained units in hot file: 9
- First archived heading: `## intake-20260607-category-planning-subscriptions — US-0018 / US-0019 / US-0020 (hot pointer)`
- Last archived heading: `## intake-20260607-category-planning-subscriptions — US-0018 / US-0019 / US-0020 (hot pointer)`
- Verification tuple (mandatory):
  - archived_body_lines=49
  - retained_body_lines=494

---

## intake-20260607-category-planning-subscriptions — US-0018 / US-0019 / US-0020 (hot pointer)

**From:** PO  
**To:** Tech Lead  
**Date:** 2026-06-07  
**Stories:** US-0018, US-0019, US-0020  
**Intake run:** `intake-20260607-category-planning-subscriptions`  
**Pack:** `first-intake-pack` (`coverage_complete=true`)  
**Next phase:** `/discovery` (US-0018 first recommended)

### Summary

Operator requested **category-first analytics** across the product, **goal-driven planning** with per-plan statistics and AI savings picks, and **subscription manual control** (search, majority category, custom tags). Intake split into **three vertical stories** per operator request ("more than one US").

### Story map

| ID | Title | Priority | Depends |
|----|-------|----------|---------|
| **US-0018** | Category filters & expense trend analytics | P1 | — (foundation) |
| **US-0019** | Goal-driven planning, per-plan stats, AI savings | P1 | US-0018 (rich UX); MVP w/ category picker OK |
| **US-0020** | Subscription search, majority category, tags | P2 | Independent |

### Risks

- Category filter without mirror quality (NULL `category_id`) → empty charts; gate on BUG-0006 ingest health
- Goal-plan math must pin forecast account (114 vs 116 — BUG-0013 pattern)
- AI savings scope creep vs US-0006 — keep opt-in line materialization only
- Tags in product DB need migration + API before UI

### Spec-pack / user-guide (scratchpad flags)

- `SPEC_PACK_MODE=1` — create/update CRS at architecture: `docs/engineering/spec-pack/US-0018/`, `US-0019/`, `US-0020/`
- `USER_GUIDE_MODE=1` — user guides at `docs/user-guides/US-0018.md` etc. at execute

### Evidence

- `handoffs/intake_evidence/intake-20260607-category-planning-subscriptions.json`
- [R-0080](docs/engineering/research.md#r-0080--category-analytics-goal-planning-subscription-tags-intake)

### Alternatives presented

| Option | Verdict |
|--------|---------|
| Single mega-epic | **Rejected** — operator asked for multiple US |
| Four stories (split filter vs chart) | **Rejected** — shared API contract |
| Firefly tag write-back | **Rejected** — read-only contract |

---

