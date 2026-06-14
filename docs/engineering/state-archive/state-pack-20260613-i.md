# State archive pack (2026-06-13)

- Rollover trigger: `STATE_HOT_MAX_LINES=1000, STATE_HOT_MAX_CHECKPOINTS=80`
- Source: `docs/engineering/state.md`
- Archived units (oldest first, contiguous prefix): 1
- Retained units in hot file: 20
- First archived heading: `## Checkpoint: verify-work US-0021 S0020 2026-06-13T08:10:01Z`
- Last archived heading: `## Checkpoint: verify-work US-0021 S0020 2026-06-13T08:10:01Z`
- Verification tuple (mandatory):
  - archived_body_lines=18
  - preamble_lines=397
  - retained_body_lines=984

---

## Checkpoint: verify-work US-0021 S0020 2026-06-13T08:10:01Z

- `orchestrator_run_id`: auto-20260613-us0021
- `phase_id`: verify-work
- `role`: qa
- `fresh_context_marker`: verify-work-20260613-us0021-qa-fresh
- `timestamp`: 2026-06-13T08:10:01Z
- `evidence_ref`: sprints/S0020/verify-work-findings.md; sprints/S0020/uat.json; sprints/S0020/uat.md; sprints/S0020/qa-findings.md; GET :18080/health 200; GET :18080/api/v1/subscriptions/transactions/search 404; GET :18080/api/v1/subscriptions/discover?account_id=114 200; GET :18080/subscriptions 404; cargo test --lib 221/221; cargo test --test us0021_transaction_search 6/6; npm test 17/17; npm run build PASS
- `active_story_id`: US-0021
- `active_sprint_id`: S0020
- `architecture_decisions`: DEC-0112, DEC-0113, DEC-0114
- `verify_work_verdict`: PASS-WITH-PREREQUISITES
- `uat_counts`: 1 pass / 5 pass-with-prerequisites / 0 fail
- `operator_gates_pending`: BACKEND_FRONTEND_DEPLOY
- `next_scheduled_phase`: release
- `next_scheduled_role`: release
- `stop_reason`: completed

