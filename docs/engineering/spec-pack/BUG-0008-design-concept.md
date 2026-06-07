# Design Concept — BUG-0008

## Summary

BUG-0008 restores operator trust in subscription alerts and improves detection recall from large synced ledgers. Two sub-defects — **W** (alert count vs list mismatch) and **X** (under-detection) — ship as sequenced bundles under **DEC-0071** and **DEC-0072**. The UX fix is reconciled unread semantics on `/subscriptions`; the recall fix is smarter payee grouping without alert spam.

## Goals

- Lifecycle fingerprint dedup for `subscription_alerts` (DEC-0071 W)
- Reconciled `GET /api/v1/subscriptions/alerts/unread-count` consumed by banner and toast
- Orphan alert cleanup on confirm/reject/inactive
- Phase 1 payee normalization + transfer counterparty priority + 730-day window (DEC-0072 X)
- W-before-X sequencing — dedup before recall tuning
- Preserve US-0005 header bell boundary; subscription trust on `/subscriptions` page

## Non-goals

- Combined header badge (subscription + unified alerts)
- AI in-pipeline detection or BUG-0007 orchestrator changes
- `list_patterns` REST filter changes (R-0065 coordinate)
- Firefly write-back or in-app subscription editing
- min_emit_confidence threshold drop before W closed

## Key decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| DEC-0071 | Fingerprint partial unique + upsert | R-0068; mirrors R-0023 pattern |
| DEC-0071 | Unread-count API | Reconciled semantics; not list length |
| DEC-0071 | US-0005-only bell | R-0011/R-0023 boundary |
| DEC-0072 | Phase 1 normalization + window | High recall, low FP (R-0069) |
| DEC-0072 | Phase 2 category grouping gated | Medium FP risk; after Phase 1 probe |
| DEC-0072 | AI deferred | Privacy, latency, coordinate table |

**Spec-pack traceability:** `docs/engineering/spec-pack/BUG-0008-crs.md`, `docs/engineering/spec-pack/BUG-0008-technical-specification.md`
