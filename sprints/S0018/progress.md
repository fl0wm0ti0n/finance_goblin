# Progress — Sprint S0018

**Story:** US-0019  
**Sprint:** S0018  
**Phase:** execute complete → qa

| ID | Status | Title |
|----|--------|-------|
| T-0186 | done | Migration `goal_balance` + plan columns |
| T-0187 | done | Create API + Goal balance template card |
| T-0188 | done | goal-stats service + target-date SQL |
| T-0189 | done | goal-stats route + GoalStatsStrip UI |
| T-0190 | done | Category `remove_outflow` cap (3-mo avg) |
| T-0191 | done | `goal_account_id` projection fork |
| T-0192 | done | category-savings-suggestions service + route |
| T-0193 | done | CategorySavingsModal + batch apply + audit |
| T-0194 | done | User guide US-0019 |
| T-0195 | done | US-0014 + DEC-0089 regression tests |
| T-0196 | done | Optional `get_category_savings` AI tool |
| T-0197 | done | UAT OIDC smoke + AC-1..AC-6 template |

## Milestones

- **2026-06-09:** Sprint planned — 12 tasks; see `handoffs/tl_to_dev.md`
- **2026-06-09:** Execute complete — T-0186..T-0197; `cargo test --lib` 204/204; `npm test` 9/9

## Test results

| Suite | Result |
|-------|--------|
| `cargo test --lib` | **204/204 PASS** |
| `npm test -- --run` | **9/9 PASS** |

## Next

- `/qa` in fresh subagent/chat (role: qa)
