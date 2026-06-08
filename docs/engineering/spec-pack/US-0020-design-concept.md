# Design Concept — US-0020

## Summary

US-0020 delivers **subscription-ops manual control**: Discover-tab search for recurring candidates, operator confirm without pending-only path, **majority display category** from linked transaction categories, and **operator-defined tags** for grouping and filtering — all product-DB overlay metadata with no Firefly write-back.

## Goals

- Discover tab search by account, payee, interval with capped results (AC-1)
- Manual confirm-from-search into confirmed patterns with DEC-0085 merge semantics (AC-2)
- Majority category badge with documented tie-break tooltip (AC-3)
- Global operator tag CRUD, multi-assign, list filter (AC-4)
- Tags and display category in product DB only (AC-5)
- US-0003 detection + US-0008 alert dedup regression (AC-6)

## Non-goals

- Firefly tag or category write-back
- Operator override column for display category (stretch)
- Per-account tag namespaces
- Changes to auto-detection pipeline thresholds
- Paginated discover beyond 50 results
- Grafana `$tag` when sprint capacity exceeded (DEC-0103 P2)

## Key decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| DEC-0098 | Reuse `detect_recurrence_groups` + GET `/discover` | DEC-0084/0086 alignment; cap 50 |
| DEC-0099 | POST `/discover/confirm` direct confirmed | AC-2; no spurious pending/alerts |
| DEC-0100 | `display_category_id` + RANK majority | Deterministic tie-break; recompute on merge |
| DEC-0101 | `operator_tags` junction tables | Normalized filter + Grafana path |
| DEC-0102 | PUT replace tags + `?tag=` list filter | Idempotent drawer UX |
| DEC-0103 | Grafana `$tag` optional P2 | AC silent; DEC-0089 precedent |
| DEC-0085 | Merge on payee+interval | Preserved for manual confirm |

## User experience

`/subscriptions` gains a **Discover** tab with account, payee, and interval filters. Results show recurring candidates the operator can confirm — merged toast when matching an existing confirmed pattern. Confirmed rows show a **majority category** badge (tooltip explains tie-break). A **tag manager** creates labels like "luxus" or "important"; the detail drawer assigns multiple tags; All/Standing tabs filter by tag chips. Pending tab and auto-detection behavior are unchanged.
