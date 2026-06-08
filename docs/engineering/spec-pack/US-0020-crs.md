# CRS — US-0020

## Purpose

Enable operators to **manually discover and confirm** recurring subscriptions, see a **majority-based display category** from transaction history, and organize subscriptions with **operator-defined tags** — extending US-0003 detection with manual control and US-0018 category display names.

## Scope

**In scope**

- `GET /api/v1/subscriptions/discover` (DEC-0098)
- `POST /api/v1/subscriptions/discover/confirm` (DEC-0099)
- `display_category_id` on `subscription_patterns` (DEC-0100)
- `operator_tags` + `subscription_pattern_tags` + CRUD (DEC-0101)
- `PUT /api/v1/subscriptions/:id/tags` + list `?tag=` (DEC-0102)
- Discover tab + majority badge + tag manager UI
- `docs/user-guides/US-0020.md`
- US-0003/US-0008 regression tests (AC-6)
- OIDC US-0010 external profile smoke (AC-6)
- Optional Grafana `$tag` (DEC-0103 P2)

**Out of scope**

- Firefly write-back (tags or categories)
- Operator display-category override column
- Detection pipeline changes
- Per-account tags
- Amount band filter (P2 stretch per DEC-0098)

## Acceptance criteria ref

See `docs/product/acceptance.md` § US-0020 — AC-1 through AC-6.

## Dependencies

- US-0003 DONE (detection, pending confirm/reject)
- DEC-0084..DEC-0086 DONE (confirm persistence, merge, tolerance)
- US-0018 DONE (DEC-0087 category catalog for display names)
- US-0008 DONE (alert dedup — AC-6 regression)
- R-0085 research gates resolved
