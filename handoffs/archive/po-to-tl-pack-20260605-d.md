# PO to TL archive pack (2026-06-05)

- Rollover trigger: `PO_TO_TL_HOT_MAX_LINES=500, PO_TO_TL_HOT_MAX_SECTIONS=40`
- Source: `handoffs/po_to_tl.md`
- Archived units (oldest first, contiguous prefix): 1
- Retained units in hot file: 7
- First archived heading: `## research-20260605-bug0012 — BUG-0012 forecast monthly bucket attribution research`
- Last archived heading: `## research-20260605-bug0012 — BUG-0012 forecast monthly bucket attribution research`
- Verification tuple (mandatory):
  - archived_body_lines=63
  - retained_body_lines=485

---

## research-20260605-bug0012 — BUG-0012 forecast monthly bucket attribution research

**From:** Tech Lead  
**To:** PO / Architecture  
**Date:** 2026-06-05  
**Bug:** BUG-0012  
**Orchestrator:** `auto-20260605-bug0012-001`  
**Next phase:** `/architecture`

### Summary

Web + code research completed for BUG-0012 AG/AH. Added **[R-0063](docs/engineering/research.md#r-0063--bug-0012-forecast-monthly-bucket-component-attribution)**. Root cause unchanged from discovery: **net-daily bucket assignment** ignores mirror categories. **Recommended fix:** component-level monthly attribution — rolling residual → Variable; each recurring due → `category_id` → `category_names` → `map_category`; extend `RecurringPattern` with `category_id` from `RecurrenceGroup.category_ids`. Daily balance path unchanged. Architecture triad **blocked** — `architecture.md` 4624/3000 lines; research in R-0063 until rollover.

### Key findings by topic

| Topic | Research | Recommendation for architecture |
|-------|----------|--------------------------------|
| **Component attribution** | [R-0063 §DEC-0007 intent](docs/engineering/research.md#r-0063--bug-0012-forecast-monthly-bucket-component-attribution) | Replace `categorize_delta(net_delta)` with per-component monthly accumulation; preserve `balance += delta` |
| **Rolling residual** | R-0006 variable layer | Always **Variable** bucket for `rolling.daily_rate` |
| **Recurring fixed/income** | `RecurrenceGroup.category_ids` already exists | Carry mode `category_id` on `RecurringPattern`; bucket via `map_category` |
| **Category name ↔ TOML** | `map_category` lowercases **name**; TOML English keys | Operator must extend `[forecast.category_buckets]` for non-matching Firefly labels; no fuzzy match in bug scope |
| **Subscription override** | `ConfirmedRecurring` lacks category | Runtime lookup from mirror txs by `payee_key` on override |
| **Same-day mixed flows** | Net delta collapses buckets | Component split avoids double-count; balance nets |
| **Tests** | Unit-first in `project.rs` | Salary+rent scenario; Variable regression; optional integration post-BUG-0006 |

### Discovery open questions — research resolution

| Question | Resolution (per research) |
|----------|---------------------------|
| Per recurring event vs net delta? | **Per component** — aligns DEC-0007 / R-0006 §4 |
| Rolling residual bucket? | **Variable** (architecture may gate positive misc) |
| Firefly name vs TOML keys? | Name map + operator TOML extension; document omniflow checklist |
| Subscription category inherit? | From group mode `category_id` or payee tx lookup on override |
| Same-day salary + rent? | Component attribution — no special case |

### Risks surfaced (carry to architecture)

1. **Operator category labels** — German/custom names miss default TOML → buckets stay Variable until config extended
2. **Variable total shift** — recurring fixed moves out of Variable (intended); needs regression test
3. **Positive rolling rate** — bucket Income vs Variable — architecture decision gate
4. **Architecture oversize** — triad blocked; § BUG-0012 needs rollover before hot-surface write
5. **Plans/alerts consumers** — verify monthly bucket semantics change does not break assumptions

### Recommended next steps

1. `/architecture` — freeze component attribution contract; `RecurringPattern` schema; positive-rolling gate; operator TOML doc; DEC-xxxx if needed
2. `/sprint-plan` — quick sprint 4–6 tasks after architecture
3. Operator re-smoke AG/AH on `financegnome.omniflow.cc` after deploy (verify-work)

### Triad check (research phase)

| Surface | Check | Result |
|---------|-------|--------|
| `docs/product/backlog.md#BUG-0012` | Research notes linked | pass (architecture phase) |
| `docs/product/acceptance.md` | AG/AH unchanged | pass |
| `docs/engineering/research.md` | R-0063 appended | pass |
| `docs/engineering/architecture.md` | § BUG-0012 | **defer** — oversize 4624/3000; triad blocked |
| Code trace | project/recurring/categories/service | pass |

`triad_hot_surface`: research prepended; architecture section deferred pending rollover

---

