# Progress — Sprint S0019

**Story:** US-0020  
**Sprint:** S0019  
**Phase:** execute complete → qa

| ID | Status | Title |
|----|--------|-------|
| T-0198 | done | Migration `display_category_id` + tag tables |
| T-0199 | done | Discover service + GET `/discover` route |
| T-0200 | done | Discover tab UI |
| T-0201 | done | POST `discover/confirm` + merge |
| T-0202 | done | Majority category compute + merge refresh |
| T-0203 | done | Majority category badge + tooltip UI |
| T-0204 | done | Operator tag CRUD API |
| T-0205 | done | PUT tag assign + list `?tag=` filter |
| T-0206 | done | Tag manager modal + filter chips UI |
| T-0207 | done | User guide US-0020 |
| T-0208 | done | US-0003/US-0008 regression tests |
| T-0209 | done | UAT OIDC smoke + AC-1..AC-6 template |
| T-0210 | done | Optional Grafana `$tag` variable (P2) |

## Milestones

- **2026-06-10:** Sprint planned — 12 tasks; see `handoffs/tl_to_dev.md`
- **2026-06-10:** Execute complete — T-0198..T-0210; `cargo test --lib` 213/213; `npm test` 9/9

## Tests

- `cargo test --lib`: **213/213 PASS**
- `npm test -- --run`: **9/9 PASS**

## Operator gates (verify-work)

- **BACKEND_FRONTEND_DEPLOY**
- **FULL_FIREFLY_SYNC**

## Next

- `/qa` in fresh subagent/chat (role: qa)
