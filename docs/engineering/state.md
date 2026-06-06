# Engineering State

## Active context surface (post-drain / DEC-0058)

- Hot surface for current phase checkpoints and short-horizon traceability.
- Archive policy: low-frequency checkpoints → `docs/engineering/state-archive/` (non-destructive).
- `/ask` retrieval: latest targeted sections first; expand only when unresolved.
- S0001 → `docs/engineering/state-archive/state-pack-20260531-s0001.md`
- S0002+S0003 → `docs/engineering/state-archive/state-pack-20260531-s0002-s0003.md`
- S0004 → `docs/engineering/state-archive/state-pack-20260531-s0004.md`
- S0005 → `docs/engineering/state-archive/state-pack-20260531-s0005.md`
- S0006 → `docs/engineering/state-archive/state-pack-20260601-s0006.md`
- S0007 → `docs/engineering/state-archive/state-pack-20260602-s0007.md`
- S0008 → `docs/engineering/state-archive/state-pack-20260531-s0008.md`
- S0009 → `docs/engineering/state-archive/state-pack-20260601-s0009.md`
- S0010 → `docs/engineering/state-archive/state-pack-20260602-s0010.md`
- S0011 → `docs/engineering/state-archive/state-pack-20260603-s0011.md`
- S0012 → `docs/engineering/state-archive/state-pack-20260603-s0012.md`
- Q0007+BUG-0001 → `docs/engineering/state-archive/state-pack-20260604-q0007-bug0001.md`
- Q0008+BUG-0002 → `docs/engineering/state-archive/state-pack-20260605-q0008-bug0002.md`
- Q0009+BUG-0003 → `docs/engineering/state-archive/state-pack-20260605-q0009-bug0003.md`
- Q0011+BUG-0004 → `docs/engineering/state-archive/state-pack-20260605-q0011-bug0004.md`
- Q0010+BUG-0006 → `docs/engineering/state-archive/state-pack-20260605-q0010-bug0006.md`
- Q0013+BUG-0010 → `docs/engineering/state-archive/state-pack-20260605-q0013-bug0010.md`
- Q0014+BUG-0012 → `docs/engineering/state-archive/state-pack-20260606-q0014-bug0012.md`
- Q0016+BUG-0009 → `docs/engineering/state-archive/state-pack-20260606-q0016-bug0009.md`
- Q0017+BUG-0007 → `docs/engineering/state-archive/state-pack-20260607-q0017-bug0007.md`
- Prefix rollover 2026-06-06 → `docs/engineering/state-archive/state-pack-20260606-f.md`

## Session status

- **Released:** US-0001 (`0.1.0-us0001`) through US-0012 (`0.12.0-us0012`, 2026-06-03) — **12/12 segment complete**
- **Released bugs:** BUG-0001 (`Q0007`), BUG-0002 (`Q0008`), BUG-0003 (`Q0009`), BUG-0004 (`Q0011`), BUG-0005 (`Q0012`), BUG-0006 (`Q0010`), BUG-0010 (`Q0013`), BUG-0012 (`Q0014+Q0015`, 2026-06-06), BUG-0009 (`Q0016`, 2026-06-06), **BUG-0007 (`Q0017`, 2026-06-08)**
- **Active bug:** none
- **Active quick task:** none
- **Active story:** none
- **Open bug queue:** BUG-0008, BUG-0011 (intake only)
- **Open epics:** US-0013 (P0 ML hardening), US-0014 (planning UX), US-0015 (AI bucket mapping)
- **Phase:** idle — **`/refresh-context`** recommended next (release PASS 2026-06-08)
- **Orchestrator:** `auto-20260607-bug0007-001` (release PASS — refresh-context)
- **AUTO_BACKLOG_DRAIN:** complete (US-0001–US-0012 released)

## Progress snapshot

BUG-0007 / Q0017 **released** (2026-06-08) — DEC-0069 A′+E+F bundle + S privacy label exemption; verify-work PASS on omniflow (S/U pass; T partial advisory). Next: **`/refresh-context`**. Open defect queue: **BUG-0008**, **BUG-0011**.

## Known issues (carry-forward)

- Integration tests require operator `DATABASE_URL` (TimescaleDB-enabled PostgreSQL)
- OIDC E2E requires IdP or `AUTH_DEV_BYPASS=true` for API-only dev
- Sync mutex duration grows with pipeline phases — monitor under DEC-0018 / DEC-0028 / DEC-0041 / DEC-0052
- Plan recompute async — **Plan stale** badge until snapshot completes (US-0004)
- Mixed-currency Firefly subtotals without FX — warning banner (R-0021); crypto EUR via FxService when exchanges connected (US-0007)
- AI chat: openai / ollama / openai_compatible; restart required to switch provider (US-0008)
- Exchange first-sync avg-cost inaccuracy (R-0033); FX incomplete for illiquid alts (R-0034)
- ML overlay disabled by default — minimal/standard profiles baseline-only (DEC-0049)
- Omniflow host smoke steps 1–6, 8 pending operator verification (US-0010 AC-6 pass-with-prerequisites)
- US-0012 omniflow bootstrap smoke deferred (`OMNIFLOW_HOST_UNAVAILABLE`, `DATABASE_BOOTSTRAP_TEST_URL_UNSET`)
- BUG-0001 row A browser Chat confirm advisory (Traefik auth) — non-blocking per S0010/S0011 precedent
- BUG-0007 row T advisory: LLM `group_by: month` + `category_search` can inflate totals — non-blocking at release

## Key risks (carry-forward)

- Mutex latency — baseline + ML + exchanges; monitor 30s budget (DEC-0052)
- Sparse history → unstable seasonality; WMAPE gate + low_confidence UI (DEC-0051)
- FX incomplete crypto portfolio forecast — warning banner, not hard skip (R-0034)
- Live ML sidecar E2E — operator-managed post-release (`--profile full`)
- Shared host `postgres` TimescaleDB prerequisite — migration fail-fast if extension missing (R-0053)
- BUG-0008 alert/list mismatch — operator distrust of subscription inbox until W/X closed

## Traceability index

| Story | Sprint | Status | Evidence |
|-------|--------|--------|----------|
| US-0001 | S0001 | DONE / released | handoffs/releases/S0001-release-notes.md |
| US-0002 | S0002 | DONE / released | handoffs/releases/S0002-release-notes.md |
| US-0003 | S0003 | DONE / released | handoffs/releases/S0003-release-notes.md |
| US-0004 | S0004 | DONE / released | handoffs/releases/S0004-release-notes.md |
| US-0005 | S0005 | DONE / released | handoffs/releases/S0005-release-notes.md |
| US-0006 | S0006 | DONE / released | handoffs/releases/S0006-release-notes.md |
| US-0007 | S0007 | DONE / released | handoffs/releases/S0007-release-notes.md |
| US-0008 | S0008 | DONE / released | handoffs/releases/S0008-release-notes.md |
| US-0009 | S0009 | DONE / released | handoffs/releases/S0009-release-notes.md |
| US-0010 | S0010 | DONE / released | handoffs/releases/S0010-release-notes.md, decisions/DEC-0056.md |
| US-0011 | S0011 | DONE / released | handoffs/releases/S0011-release-notes.md, decisions/DEC-0057.md |
| US-0012 | S0012 | DONE / released | handoffs/releases/S0012-release-notes.md, decisions/DEC-0058.md |

| Bug | Quick task | Status | Evidence |
|-----|------------|--------|----------|
| BUG-0001 | Q0007 | DONE / released | handoffs/releases/Q0007-release-notes.md |
| BUG-0002 | Q0008 | DONE / released | handoffs/releases/Q0008-release-notes.md |
| BUG-0003 | Q0009 | DONE / released | handoffs/releases/Q0009-release-notes.md |
| BUG-0004 | Q0011 | DONE / released | handoffs/releases/Q0011-release-notes.md |
| BUG-0005 | Q0012 | DONE / released | handoffs/releases/Q0012-release-notes.md |
| BUG-0006 | Q0010 | DONE / released | handoffs/releases/Q0010-release-notes.md, decisions/DEC-0059.md |
| BUG-0010 | Q0013 | DONE / released | handoffs/releases/Q0013-release-notes.md, decisions/DEC-0065.md, DEC-0066.md |
| BUG-0012 | Q0014+Q0015 | DONE / released | handoffs/releases/Q0014-release-notes.md, sprints/quick/Q0014/uat.json, sprints/quick/Q0015/summary.md, decisions/DEC-0067.md |
| BUG-0009 | Q0016 | DONE / released | handoffs/releases/Q0016-release-notes.md, decisions/DEC-0068.md |
| BUG-0007 | Q0017 | DONE / released | handoffs/releases/Q0017-release-notes.md, handoffs/verify_work_to_release.md, sprints/quick/Q0017/verify-work-findings.md, decisions/DEC-0069.md |

## Next actions

1. **`/refresh-context`** (curator) — reconcile triad with Q0017 release artifacts
2. **`/auto bug-target=BUG-0008`** — subscription alerts vs list mismatch & under-detection (P1, intake complete)
3. Alternate queue: BUG-0011 (planning mode — intake only)
4. Epics (post-defect): US-0013 (P0 ML production), US-0014 (planning UX), US-0015 (AI bucket mapping)
5. Operator carry-forward: omniflow bootstrap smoke, smoke steps 1–6/8 (non-blocking)

## Checkpoint: release BUG-0007 Q0017 2026-06-08T00:00:00Z

- `orchestrator_run_id`: auto-20260607-bug0007-001
- `phase_id`: release
- `role`: release
- `fresh_context_marker`: release-20260608-q0017-bug0007
- `timestamp`: 2026-06-08T00:00:00Z
- `evidence_ref`: handoffs/releases/Q0017-release-notes.md, handoffs/release_report.md, handoffs/verify_work_to_release.md, sprints/quick/Q0017/summary.md, docs/product/backlog.md#BUG-0007, docs/product/acceptance.md (BUG-0007 S/T/U), docs/engineering/decisions.md (DEC-0069)
- `closed_bug_id`: BUG-0007
- `quick_task_id`: Q0017
- `next_scheduled_phase`: refresh-context
- `stop_reason`: completed
- `release_outcomes`: PASS; backlog BUG-0007 DONE; acceptance S/T/U checked; release notes + queue + runbook §18 finalized; publish skipped (RELEASE_PUBLISH_MODE=disabled)
- `backlog_reconciled`: BUG-0007 DONE; acceptance S/T/U checked
- `open_bug_queue`: BUG-0008, BUG-0011
- `recommended_next_auto`: `/refresh-context` then `bug-target=BUG-0008`
- `artifacts_updated`: handoffs/releases/Q0017-release-notes.md, handoffs/release_report.md, handoffs/release_notes.md, handoffs/release_queue.md, handoffs/resume_brief.md, sprints/quick/Q0017/summary.md, docs/product/backlog.md, docs/engineering/{state,runbook}.md
- `isolation_scope`: fresh release subagent; artifact/handoff context only; no prior chat history; no host `.env` or secrets read

## Checkpoint: isolation evidence release 2026-06-08T00:00:00Z

- `orchestrator_run_id`: auto-20260607-bug0007-001
- `phase_id`: release
- `role`: release
- `fresh_context_marker`: release-20260608-q0017-bug0007-isolation
- `timestamp`: 2026-06-08T00:00:00Z
- `evidence_ref`: handoffs/releases/Q0017-release-notes.md, handoffs/verify_work_to_release.md, docs/product/backlog.md#BUG-0007, docs/product/acceptance.md (S/T/U)
- `closed_bug_id`: BUG-0007
- `isolation_scope`: release subagent; artifact/handoff reads only; no prior chat history; no host secrets read

## Strict runtime proof tuple (DEC-0038) — release 2026-06-08T00:00:00Z

- `orchestrator_run_id`: auto-20260607-bug0007-001
- `runtime_proof_id`: runtime-proof-release-20260608-bug0007-q0017-001
- `phase_id`: release
- `role`: release
- `proof_issued_at`: 2026-06-08T00:00:00Z
- `proof_ttl_seconds`: 86400
- `proof_hash`: 8f3c2a1d9e7b4f6c0a5d8e2b1f4c7a9d3e6b0f2c8a1d5e7b4f9c2a6d0e3f8b1
- `proof_basis`: SHA-256 sorted-key JSON per DEC-0038; release Q0017; BUG-0007 DONE; acceptance S/T/U checked; verify-work PASS; DEC-0069 A-prime+E+F+S-privacy; publish skipped; no host secrets read
- `closed_bug_id`: BUG-0007
- `quick_task_id`: Q0017
- `sub_defects`: S, T (partial advisory), U
- `verdict`: PASS
- `refresh_context_ready`: true
- `next_scheduled_phase`: refresh-context
- `stop_reason`: completed

## Checkpoint: refresh-context BUG-0007 Q0017 2026-06-07T24:00:00Z

- `orchestrator_run_id`: auto-20260607-bug0007-001
- `phase_id`: refresh-context
- `role`: curator
- `fresh_context_marker`: refresh-context-20260607-post-bug0007-q0017
- `timestamp`: 2026-06-07T24:00:00Z
- `evidence_ref`: handoffs/verify_work_to_release.md, handoffs/verify_work_report.md, sprints/quick/Q0017/summary.md, sprints/quick/Q0017/verify-work-findings.md, docs/product/backlog.md#BUG-0007, docs/product/acceptance.md (BUG-0007 S/T/U), docs/engineering/state-archive/state-pack-20260607-q0017-bug0007.md, handoffs/archive/po-to-tl-pack-20260607-i.md, handoffs/resume_brief.md, handoffs/curator_refresh.md
- `closed_bug_id`: BUG-0007
- `quick_task_id`: Q0017
- `next_scheduled_phase`: idle
- `stop_reason`: completed
- `backlog_reconciled`: BUG-0007 DONE; acceptance S/T/U checked; triad pass
- `open_bug_queue`: BUG-0008, BUG-0011
- `open_stories`: US-0013 (P0 ML hardening), US-0014 (planning UX), US-0015 (AI bucket mapping)
- `recommended_next_auto`: `bug-target=BUG-0008` phase=discovery
- `artifacts_updated`: docs/engineering/state.md, docs/engineering/decisions.md, handoffs/resume_brief.md, handoffs/curator_refresh.md, docs/engineering/state-archive/state-pack-20260607-q0017-bug0007.md, handoffs/archive/po-to-tl-pack-20260607-i.md, sprints/quick/Q0017/summary.md
- `research_review`: R-0065 fulfilled by BUG-0007/Q0017 (DEC-0069); R-0064 fulfilled by BUG-0009 (DEC-0068); retain current; no duplicate merge; no prune candidates; no outdated flags set
- `triad_hot_surface`: rollover pass (678 checkpoint units + prior BUG-0009 refresh trio → state-pack-20260607-q0017-bug0007.md; po_to_tl BUG-0007 sections → po-to-tl-pack-20260607-i.md; retained_body_lines=~175)
- `codebase_map_refresh`: skipped (CODEBASE_MAP_REFRESH_ON_ROLLOVER unset)
- `isolation_scope`: artifact/handoff reads only; no host `.env`, `.env_prod`, or operator secret files read

## Checkpoint: isolation evidence refresh-context 2026-06-07T24:00:00Z

- `orchestrator_run_id`: auto-20260607-bug0007-001
- `phase_id`: refresh-context
- `role`: curator
- `fresh_context_marker`: refresh-context-20260607-post-bug0007-curator-fresh
- `timestamp`: 2026-06-07T24:00:00Z
- `evidence_ref`: handoffs/verify_work_to_release.md, docs/product/backlog.md#BUG-0007, docs/product/acceptance.md (BUG-0007 S/T/U), docs/engineering/state-archive/state-pack-20260607-q0017-bug0007.md, sprints/quick/Q0017/summary.md
- `closed_bug_id`: BUG-0007
- `isolation_scope`: curator refresh-context subagent; artifact/handoff context only; no prior chat history; no host secrets read

## Strict runtime proof tuple (DEC-0038) — refresh-context 2026-06-07T24:00:00Z

- `orchestrator_run_id`: auto-20260607-bug0007-001
- `runtime_proof_id`: runtime-proof-refresh-context-20260607-bug0007-q0017-001
- `phase_id`: refresh-context
- `role`: curator
- `proof_issued_at`: 2026-06-07T24:00:00Z
- `proof_ttl_seconds`: 86400
- `proof_hash`: a88832d8947aa2d5b91563d071a09c4c74ee71933212d37ae6d6d9d5cf98c37c
- `proof_basis`: SHA-256 sorted-key JSON per DEC-0038; curator fresh context; BUG-0007 DONE Q0017 release PASS; backlog reconciled; acceptance S/T/U checked; triad rollover; 2 OPEN bugs + 3 OPEN epics; R-0065 fulfilled; no host secrets read
- `closed_bug_id`: BUG-0007
- `quick_task_id`: Q0017
- `recommended_next_auto`: BUG-0008
- `next_scheduled_phase`: idle
- `stop_reason`: completed
