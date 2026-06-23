# State archive pack (2026-06-22)

- Rollover trigger: `STATE_HOT_MAX_LINES=1000, STATE_HOT_MAX_CHECKPOINTS=80`
- Source: `docs/engineering/state.md`
- Archived units (oldest first, contiguous prefix): 1
- Retained units in hot file: 11
- First archived heading: `## Checkpoint: /auto drain-advance materialization 2026-06-14T18:20:00Z`
- Last archived heading: `## Checkpoint: /auto drain-advance materialization 2026-06-14T18:20:00Z`
- Verification tuple (mandatory):
  - archived_body_lines=32
  - preamble_lines=449
  - retained_body_lines=991

---

## Checkpoint: /auto drain-advance materialization 2026-06-14T18:20:00Z

- `orchestrator_run_id`: auto-20260613-bug0025
- `invocation_mode`: auto
- `drain_advance_action`: spawned
- `drain_advance_trigger`: refresh-context complete + AUTO_BACKLOG_DRAIN=1 + budget remaining
- `selected_work_item_kind`: story
- `selected_work_item_id`: US-0022
- `selection_policy`: priority_then_backlog_order (P2 US-0022 — only OPEN story)
- `resolved_start_phase`: discovery
- `resolution_source`: backlog_drain
- `resolution_status`: resolved
- `timestamp`: 2026-06-14T18:20:00Z
- `delivery_mode`: standard
- `resolved_phase_plan`: discovery, research, architecture, sprint-plan, plan-verify, execute, qa, verify-work, release, refresh-context
- `skipped_phases`: intake (pre-completed per backlog § US-0022; evidence intake-20260613-deploy-version-stamp)
- `segment_work_item_kind`: story
- `active_story_id`: US-0022
- `story_queue_position`: 1
- `story_queue_remaining`: 0
- `backlog_drain_active`: true
- `AUTO_BACKLOG_DRAIN`: 1
- `AUTO_FLOW_MODE`: full_autonomy
- `native_chain_active`: true
- `native_chain_continuing`: true
- `outer_cycle_index`: 23
- `intake_evidence`: handoffs/intake_evidence/intake-20260613-deploy-version-stamp.json
- `research_ref`: R-0095 (pre-existing)
- `acceptance_rows`: AC-1, AC-2, AC-3, AC-4, AC-5, AC-6
- `next_scheduled_phase`: discovery
- `next_scheduled_role`: po

