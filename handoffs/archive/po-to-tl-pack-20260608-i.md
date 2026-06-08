# PO to TL archive pack (2026-06-08)

- Rollover trigger: `PO_TO_TL_HOT_MAX_LINES=500, PO_TO_TL_HOT_MAX_SECTIONS=40`
- Source: `handoffs/po_to_tl.md`
- Archived units (oldest first, contiguous prefix): 1
- Retained units in hot file: 11
- First archived heading: `## discovery-20260609-us0020 — US-0020 subscription discover, category & tags (hot pointer)`
- Last archived heading: `## discovery-20260609-us0020 — US-0020 subscription discover, category & tags (hot pointer)`
- Verification tuple (mandatory):
  - archived_body_lines=99
  - retained_body_lines=500

---

## discovery-20260609-us0020 — US-0020 subscription discover, category & tags (hot pointer)

**From:** PO  
**To:** Tech Lead  
**Date:** 2026-06-09  
**Story:** US-0020  
**Orchestrator run:** `auto-20260608-us0020-001`  
**Intake bundle:** `intake-20260607-category-planning-subscriptions`  
**Prior releases:** US-0018 `0.18.0-us0018` (S0017); US-0019 `0.19.0-us0019` (S0018)  
**Next phase:** `/research`

### Summary

Discovery refined **US-0020** as the **subscription-ops vertical** for the intake bundle: **manual Discover search** on `/subscriptions`, **operator confirm** without auto-detection-only path, **majority display category** from linked transaction categories, and **operator-defined tags** (CRUD + assign + filter) — all product-DB overlay metadata with **no Firefly write-back**. Builds on US-0003 detection, **DEC-0084**..**DEC-0086** confirm persistence, and US-0018 category catalog. **Single story retained.**

### Frozen dependencies

| Source | Relevance to US-0020 |
|--------|----------------------|
| **US-0003** | `/subscriptions` tabs, confirm/reject API, detection pipeline — preserve; manual path additive |
| **DEC-0084** | Card billing `payee_key` normalization — applies to explorer grouping + manual confirm |
| **DEC-0085** | Payee+interval confirm inheritance — manual confirm must merge, not duplicate |
| **DEC-0086** | ±3d interval tolerance + fingerprint rotation on merge |
| **US-0018 / DEC-0087** | `GET /api/v1/categories` — display names for majority category badge |
| **R-0080** | Explorer SQL, tag schema, majority `MODE()` — extend in research |

### Surface → feature map (canonical)

| Surface | Tab / area | Discovery target |
|---------|------------|------------------|
| Discover search | `/subscriptions` **Discover** | Filters: account, payee/title text, interval (months); capped results (50); excludes confirmed/rejected fingerprints |
| Manual confirm | Discover row action | `POST` → **confirmed** pattern + linked txs; kind override (subscription vs standing_order) |
| Majority category | Confirmed list + detail | `display_category_id` = mode of linked tx categories on confirm; tooltip tie-break |
| Tag manager | Subscriptions page | CRUD operator tags (luxus, important, …) |
| Tag assign | Detail drawer | Multi-tag chips per subscription |
| Tag filter | All / Standing tabs | List API `?tag=` filter |
| Regression | Pending + alerts | US-0003/US-0008 + OIDC smoke (AC-6) |

### Partial implementation review

| Area | Status |
|------|--------|
| `/subscriptions` All/Pending/Standing + confirm/reject | **Done** (US-0003) |
| Detection pipeline (`detection.rs`, `group.rs`) | **Done** |
| DEC-0084..0086 confirm persistence | **Done** (Q0023) |
| `category_id` on mirror txs + categories catalog | **Done** |
| Explorer search API + Discover tab UI | **Missing** |
| Manual confirm-from-search | **Missing** |
| `display_category_id` column + majority compute | **Missing** |
| Tag tables + CRUD + assign + filter | **Missing** |
| Grafana `$tag` on `subscriptions` dashboard | **Missing** (stretch) |

### Discovery decomposition evidence

- Feature/workflow count: discover search + manual confirm + majority category + tag CRUD/assign/filter (moderate-high — **single story retained**)
- Cross-cutting impact: `backend/subscriptions/`, new migration, `frontend/SubscriptionsPage.tsx`, optional Grafana `subscriptions.json`
- Acceptance breadth: 6 AC unchanged (`docs/product/acceptance.md#US-0020`)
- Risk surface: explorer SQL performance on 365d window; manual confirm vs DEC-0085 merge conflicts; majority tie-break UX; tag delete with assignments; detection regression if explorer bypasses rejection maps

### Decision gates (PO recommendation)

| Topic | Recommendation | Alternative |
|-------|--------------|-------------|
| Search home | **Discover** tab on `/subscriptions` | Separate route |
| Manual confirm | Direct **confirmed** insert + link txs | Pending-then-confirm |
| Majority algorithm | `MODE(category_id)`; tie → most recent; NULL excluded | Operator override column (stretch) |
| Amount band | Stretch — not MVP | Required AC-1 filter |
| Tag scope | Global operator tags | Per-account |
| Grafana `$tag` | Stretch if capacity | Defer post-MVP |
| Result cap | 50 per query | Unlimited |

### Open questions (carry to research — extend R-0080)

| Topic | Question |
|-------|----------|
| **Explorer SQL** | Reuse `detect_recurrence_groups` vs ad-hoc `GROUP BY payee_key, account_id` with `HAVING COUNT(*) >= 3` |
| **Manual confirm API** | New `POST /api/v1/subscriptions/discover/confirm` vs extend existing confirm with explorer payload |
| **Majority category** | Compute at confirm only vs recompute on sync when new txs link |
| **Outlier tolerance** | Pure mode sufficient for 1/N miscategorization or exclude singleton categories when N≥6 |
| **Tag schema** | `subscription_tags` + `subscription_pattern_tags` naming; soft vs hard delete |
| **List filters** | Combine `?tag=` + `?account_id=` + existing `status`/`kind` params |
| **DEC-0085 interaction** | Manual confirm when payee+interval already confirmed — merge vs 409 |
| **Grafana** | `$tag` variable SQL join pattern on `subscriptions` dashboard |

### Artifacts updated

- `docs/product/vision.md` — US-0020 discovery section
- `docs/product/backlog.md#US-0020` — discovery refinements
- `docs/engineering/state.md` — isolation + runtime proof
- `handoffs/po_to_tl.md` — this handoff

### Triad check (discovery phase)

`triad_hot_surface`: _(pending rollover + --check)_

**Recommended next phase:** `/research` (tech-lead) — extend [R-0080](docs/engineering/research.md#r-0080--category-analytics-goal-planning-subscription-tags-intake) subscription/tags portion

---

