# Technical Specification — US-0020

## Overview

Implement **DEC-0098** through **DEC-0103**: subscription discover explorer; manual confirm-from-discover; majority display category; operator tag schema and APIs; optional Grafana `$tag`.

## Components

| Layer | Change |
|-------|--------|
| `backend/migrations/` | **New** — `display_category_id`; `operator_tags`; `subscription_pattern_tags` |
| `backend/src/subscriptions/discovery.rs` | **New** — discover service (DEC-0098) |
| `backend/src/subscriptions/repository.rs` | **Extend** — confirm-from-discover, majority compute, tag joins |
| `backend/src/subscriptions/service.rs` | **Extend** — discover + confirm orchestration |
| `backend/src/api/subscriptions.rs` | **Extend** — discover GET, confirm POST, list `?tag=`, PUT tags |
| `backend/src/api/subscription_tags.rs` | **New** — tag CRUD |
| `frontend/src/pages/SubscriptionsPage.tsx` | **Extend** — Discover tab, badges, tag UI |
| `frontend/src/lib/api.ts` | **Extend** — discover, tags types |
| `grafana/.../subscriptions.json` | **Optional** — `$tag` variable (P2) |
| `docs/user-guides/US-0020.md` | **New** — at execute |

## Interfaces

### `GET /api/v1/subscriptions/discover`

**Query:** `account_id` (recommended required), `payee`, `interval_days`, optional `amount_min`/`amount_max` (P2), `limit` (max 50)

**Response:** `candidates[]` + `meta { limit, truncated, window_days }`

### `POST /api/v1/subscriptions/discover/confirm`

**Body:** `payee_key`, `interval_days`, `median_amount`, `transaction_ids[]`, `kind`

**Responses:** 201 create, 200 merge, 409 rejection/conflict

### `GET /api/v1/subscriptions`

**Query extension:** `tag` (slug) AND-composed with `status`, `kind`

**DTO extension:** `display_category_id`, `display_category_name`, `tags[]`

### Tag CRUD

| Method | Path |
|--------|------|
| POST | `/api/v1/subscription-tags` |
| GET | `/api/v1/subscription-tags` |
| PATCH | `/api/v1/subscription-tags/:id` |
| DELETE | `/api/v1/subscription-tags/:id` |

### `PUT /api/v1/subscriptions/:id/tags`

**Body:** `{ "tag_ids": ["…"] }` — replace set

## Majority category (internal)

Compute at confirm and on `merge_confirmed_pattern` — RANK by count DESC, max(date) DESC per DEC-0100.

## Sequencing (sprint-plan input)

1. M1 migration
2. D1 discover API → D2 Discover UI
3. C1 confirm-from-discover → C2 majority compute → C3 badge UI
4. T1 tag CRUD → T2 assign/filter → T3 tag UI
5. R1 docs + R2 regression
6. V1 smoke; G1 optional Grafana

## Verification

- Unit: discover post-filters; confirm merge vs create; majority tie-break; tag slug dedupe
- Integration: rejected payee-interval → 409; list `?tag=`; no `new_detection` on manual confirm
- Regression: `DetectionPipeline` ordering unchanged; pending confirm path intact
- UI: Discover tab; majority tooltip; tag manager delete confirm
