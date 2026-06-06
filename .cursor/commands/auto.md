---
description: "its-magic auto: deterministic continuation orchestrator."
---

# /auto

## Subagents
- curator
- tech-lead

## Execution model
- `/auto` is a **spawn-only orchestrator**: it schedules materialization, spawns
  fresh **phase-role** subagents, and verifies phase boundaries—it **must not**
  execute lifecycle phase work, perform phase-role duties, or author **phase
  deliverables** in the orchestrator context.
- For each phase, **spawn a fresh subagent** for that phase’s canonical role;
  phase output must arrive only via artifacts and handoff files (no in-turn
  orchestrator execution of that phase).
- Phase context transfer happens only through artifacts and handoff files.
- Scope is process/workflow orchestration only. Do not claim runtime product
  orchestration changes.
- **Bug-queue mode** (**`US-0087`**) uses the same **spawn-only** contract: the
  orchestrator schedules materialization and spawns phase-role subagents per
  bug segment—it **must not** run **`execute`**, **`qa`**, or other lifecycle
  phases in the orchestrator turn. Violations → **`AUTO_ORCHESTRATOR_PHASE_EXECUTION`**
  (**`BUG-0006`**, **`US-0069`**, **`DEC-0051`**).

## Spawn-boundary integrity (BUG-0006)

- **Forbidden**: treating the orchestrator turn as the executor of a lifecycle
  phase (for example running **`architecture`**, **`execute`**, **`qa`**, or any
  other **`phase_id`** in the orchestrator instead of spawning the required
  subagent).
- **Fail fast** with **`AUTO_ORCHESTRATOR_PHASE_EXECUTION`**. **Remediation**:
  stop; spawn a **fresh** subagent for the canonical **`phase_id`** and **`role`**
  per the phase→role matrix (**DEC-0051**); do not merge phase output into
  orchestrator turns. **Distinct from** **`PHASE_CONTEXT_ISOLATION_VIOLATION`**
  (wrong writer / isolation break) and **`RUNTIME_PROOF_*`** / **`PHASE_ROLE_*`**
  families—do not overload those codes for a missing-spawn violation.

## Continuous multi-phase execution (US-0088)

A single `/auto` orchestrated run advances through **all phases** in the
**intersected resolved schedule** (reference **Step 5**) until a
**deterministic stop condition** fires. The orchestrator does **not** stop after
spawning one phase unless the stop matrix requires it.

**Outer-driver equivalence (AC-1, Option B)**: When a single Cursor `/auto`
invocation cannot schedule multiple fresh subagent turns (product/runtime
constraint), a **documented outer driver** (operator script or manual
re-invocation with `start-from` / refreshed `resume_brief`) is
**deterministically equivalent** provided: same intersected phase order, same
isolation + strict-proof attestation per phase (**DEC-0038**), same stop
reasons, and same `resume_brief` + `state.md` refresh at every boundary.
Operators must follow the runbook recipe
(**`docs/engineering/runbook.md`** § Continuous `/auto` + backlog drain).

**Deterministic stop matrix** (see also architecture `# US-0088`):

| Condition | Behavior |
|-----------|----------|
| Next phase exists, no hard stop | **Continue** — preflight US-0069, spawn next phase |
| `decision_gate` | **Stop** (non-suppressible) |
| `error` / missing critical input | **Stop** (non-suppressible) |
| `AUTO_PAUSE_REQUEST` / `pause` | **Stop** at safe boundary (non-suppressible) |
| `AUTO_LOOP_MAX_CYCLES` / `loop_max` | **Stop** (non-suppressible) |
| `blocked` (sync/scope gate) | **Stop** (non-suppressible) |
| US lifecycle DONE / sprint segment complete | **Stop** segment; `AUTO_BACKLOG_DRAIN=1` may advance to next OPEN story (recompute phase plan — **reference Step 5**) |
| `BACKLOG_MAX_STORIES_REACHED` | **Stop** (non-suppressible) |

`stop_reason` vocabulary: `completed`, `decision_gate`, `missing_input`,
`pause_request`, `loop_max`, `error`, `blocked`.

## Full specification (US-0080 / DEC-0062)

Long prose, expanded mode semantics, and **Steps 1–13** detail live in
**`docs/engineering/auto-orchestration-reference.md`** (jointly normative). This file
keeps **contract excerpts** required for regression parity and default `/auto` loads.

## Per-phase isolation enforcement (US-0048 / DEC-0029)

- Orchestrator must not write phase deliverables (`PHASE_CONTEXT_ISOLATION_VIOLATION`).
- Each spawned phase appends isolation evidence to `docs/engineering/state.md`
  with `phase_id`, `role`, `fresh_context_marker`, `timestamp`, `evidence_ref`.
- Fail closed on missing/invalid/stale evidence.

Reason codes: `PHASE_CONTEXT_ISOLATION_MISSING`, `PHASE_CONTEXT_ISOLATION_VIOLATION`,
`ISOLATION_EVIDENCE_STALE`, `ISOLATION_EVIDENCE_INVALID`.

## Strict runtime proof enforcement (US-0056 / DEC-0038)

- Each completed phase supplies tuple: `orchestrator_run_id`, `runtime_proof_id`,
  `phase_id`, `role`, `proof_issued_at`, `proof_ttl_seconds`, `proof_hash`
  (hash = SHA-256 sorted-key JSON per **DEC-0038**).
- `runtime_proof_id` unique per phase run; enforce TTL freshness and linkage.

Reason codes: `RUNTIME_PROOF_MISSING`, `RUNTIME_PROOF_INVALID`, `RUNTIME_PROOF_REUSED`,
`RUNTIME_PROOF_STALE`, `RUNTIME_PROOF_AMBIGUOUS_LINK`.

## Strict phase role enforcement (US-0069 / DEC-0051)

Post-hoc markers alone are insufficient: preflight expected role, capability gate,
post-completion match of isolation + strict-proof `role`, execute default deny unless
override governance is satisfied.

### Canonical phase→role matrix (fixed defaults)

| phase_id | Allowed roles | Default when no valid alternate policy |
|----------|-----------------|----------------------------------------|
| `intake` | `po` | `po` |
| `discovery` | `po` | `po` |
| `research` | `po`, `tech-lead` | `tech-lead` |
| `architecture` | `tech-lead` | `tech-lead` |
| `sprint-plan` | `tech-lead` | `tech-lead` |
| `plan-verify` | `qa`, `tech-lead` | `qa` |
| `execute` | `dev` (override path only) | `dev` |
| `qa` | `qa` | `qa` |
| `verify-work` | `qa` | `qa` |
| `release` | `release` | `release` |
| `refresh-context` | `curator`, `po` | `curator` |

Alternate-role keys (merged scratchpad): `AUTO_ROLE_RESEARCH`, `AUTO_ROLE_PLAN_VERIFY`,
`AUTO_ROLE_REFRESH_CONTEXT` — single-valued resolution per **DEC-0051** (see reference).

Execute override: requires `AUTO_EXECUTE_ROLE_OVERRIDE=allowed_non_dev_execute` **and**
parseable `EXECUTE_OVERRIDE_GOVERNANCE_REF`.

Role reason codes: `PHASE_ROLE_CAPABILITY_MISSING`, `PHASE_ROLE_MISMATCH`.

## Configurable phase selection policy (US-0070 / DEC-0052)

Treat **resolved phase plan** as fail-closed schedule from merged scratchpad **before**
resume / `start-from` intersection. Canonical lifecycle:

`intake` → `discovery` → `research` → `architecture` → `sprint-plan` →
`plan-verify` → `execute` → `qa` → `verify-work` → `release` → `refresh-context`

Selectors and reinstatement: see reference. Phase-plan reason codes include
`PHASE_POLICY_CONFLICT`, `PHASE_PLAN_UNKNOWN_PHASE`, `START_FROM_PHASE_PLAN_EMPTY_INTERSECTION`.

Phase boundary visibility (**AC-10**): record `resolved_phase_plan` snapshot,
`skipped_phases`, `phase_boundary`, `next_scheduled_phase` on `state.md`. For
bug-queue segments, also record **`segment_work_item_kind`**, **`active_bug_id`**,
**`bug_queue_position`**, **`bug_queue_remaining`**, **`backlog_drain_active`**,
**`bug_queue_active`** per **`docs/engineering/architecture.md`** **`# US-0087`**
and **`docs/engineering/auto-orchestration-reference.md`**.

## Inputs

Merged scratchpad (**US-0073** / **DEC-0055**), automation flags (`AUTO_*`, `SECURITY_REVIEW`,
`TEAM_*`), phase-plan keys `AUTO_PHASE_PLAN`, `AUTO_PHASE_EXCLUDE`, `AUTO_PHASE_INCLUDE`,
`AUTO_PHASE_PROFILE`, `AUTO_PHASE_HIGH_RISK_ACK`, product/engineering docs,
optional `start-from=<phase>`, optional **`bug-target=BUG-####`** or
**`bug-target=all-open`**, optional `--execute-bulk`, `handoffs/resume_brief.md`,
`docs/engineering/state.md`.

## Automation remote routing contract (US-0086)

- Automation-only gate: `AUTO_REMOTE_AUTOMATION_PROFILE=deterministic_v1` enables
  target routing; `off` keeps manual/local behavior unchanged.
- Explicit intent literal is constrained to: `start container <target_id>`.
- Deterministic precedence when profile is enabled:
  1. explicit intent target id resolution,
  2. canonical target validation (`targets[].id` exists and is enabled),
  3. documented heuristic fallback,
  4. local default when no remote target is selected.
- Fail-closed reason codes (do not overload):
  `REMOTE_AUTOMATION_MODE_OFF`, `REMOTE_TARGET_UNKNOWN`,
  `REMOTE_TARGET_DISABLED`, `REMOTE_TARGET_UNROUTABLE`.
- Mode-off guardrail: never silently reroute `TEST_COMMAND` to remote when
  automation profile is disabled.

## Canonical status contract (US-0045)

Story status authority: `docs/product/backlog.md` only; do not infer readiness from
contradictory derived views.

## Outputs (artifacts)

Phase artifacts, `docs/engineering/state.md`, `handoffs/resume_brief.md` when stopped,
QA loop handoffs when applicable, continuation breadcrumbs including `resolution_source`
(`argument|resume_brief|state_fallback`) and related resume metadata per reference.

## Stop conditions

Deterministic stop reasons (see **Stop matrix** in `## Continuous multi-phase
execution (US-0088)` above): `completed`, `decision_gate`, `missing_input`,
`pause_request`, `loop_max`, `error`, `blocked`.

## Optional backlog-drain mode (US-0044 / DEC-0022)

Canonical controls: `AUTO_BACKLOG_DRAIN`, `AUTO_BACKLOG_MAX_STORIES`, `AUTO_BACKLOG_ON_BLOCK`,
`AUTO_STORY_SELECTION`. When `AUTO_BACKLOG_DRAIN=1`, each story advances through
**multiple phases** until its terminal boundary (**reference Step 5**); the
orchestrator **recomputes** the materialized phase plan at each **story boundary**
and selects the **next eligible OPEN story** per `AUTO_STORY_SELECTION`.
Reason codes include `BACKLOG_MAX_STORIES_REACHED`. Full semantics: reference.

## Optional bug-queue mode (US-0087)

Canonical **argv** literals (exact strings; **no aliases** in v1):
- **`bug-target=BUG-####`** (example: **`bug-target=BUG-0007`**) — single defect from
  **`docs/product/backlog.md`** **`## Bug issues (canonical)`** with status **OPEN**.
- **`bug-target=all-open`** — deterministic **OPEN**-only queue, ascending **numeric**
  **`BUG-####`** sort, optional cap **`AUTO_BUG_MAX_ITEMS`** (see reference).

Scratchpad keys (**default-off**): **`AUTO_BUG_QUEUE`**, **`AUTO_BUG_TARGET`**,
**`AUTO_BUG_MAX_ITEMS`**, **`AUTO_BUG_ON_BLOCK`** — full semantics: reference +
**`architecture.md`** **`# US-0087`**.

**Scheduler mutex**: if merged scratchpad has **`AUTO_BACKLOG_DRAIN=1`** **and**
**`AUTO_BUG_QUEUE=1`** **and** this invocation has **no** explicit **`bug-target=`**
argv token → fail closed with **`AUTO_SCHEDULER_CONFLICT`** (use
**`[AUTO_RESUME_ERROR] AUTO_SCHEDULER_CONFLICT: ...`** form per reference). When
**`bug-target=`** argv is present, it **selects** the bug scheduler for this run;
**`AUTO_BACKLOG_DRAIN`** must **not** also drive story selection for that same
materialized run.

Fail-closed codes (orthogonal to existing resume/phase codes; do **not** overload):
- **`AUTO_BUG_QUEUE_EMPTY`** — **`all-open`** (or equivalent) and zero **OPEN** bugs.
- **`AUTO_BUG_TARGET_UNKNOWN`** — malformed id, wrong pattern, or id missing from canonical bug section.
- **`AUTO_BUG_TARGET_NOT_OPEN`** — known id exists but status is not **OPEN** (e.g. **DONE**).

## Optional bulk execute mode (US-0047 / DEC-0024)

Explicit `--execute-bulk` or `AUTO_EXECUTE_BULK=1`. Reason codes include
`EXEC_BULK_MAX_ITEMS_REACHED`, `EXEC_TEAM_SCOPE_BLOCKED`, `EXEC_TEAM_SCOPE_SKIPPED`. Full
semantics: reference.

## Sync policy contract (US-0038 / DEC-0018)

Phase-completion boundary evaluation only. **Guarded auto-push eligibility chain**
(checklist in reference). Reason codes include `BRANCH_NOT_ALLOWLISTED`, `TEST_COMMAND_MISSING`,
`SYNC_PUSHED`. Full list: reference.

## Canonical `start-from` phase IDs

`intake`, `discovery`, `research`, `architecture`, `sprint-plan`, `plan-verify`,
`execute`, `qa`, `verify-work`, `release`, `refresh-context` — aliases invalid.

## Deterministic resume-source precedence

Resolve nominal start phase and scheduler inputs in strict order (**`US-0087`**
extends scratchpad vs **`resume_brief`** ordering — full matrix: reference):

1. Explicit `/auto start-from=<phase>`
2. Explicit **`bug-target=`** argv token when present (parsed **before** merged
   scratchpad scheduler keys; selects bug scheduler for this run).
3. Merged scratchpad (**`US-0073`** / **`DEC-0055`**) — including **`AUTO_BACKLOG_DRAIN`**,
   **`AUTO_BUG_QUEUE`**, **`AUTO_BUG_TARGET`**, etc.
4. `handoffs/resume_brief.md`
5. Conservative `docs/engineering/state.md` fallback
6. Fail fast on ambiguity/conflict/unrecoverable inputs (including
   **`AUTO_SCHEDULER_CONFLICT`** when both schedulers are enabled in scratchpad
   without **`bug-target=`** argv resolution).

If `resume_brief.md` is present but stale or unparseable, fail fast instead
of silently falling back.

## Fail-fast error code contract

All resume-resolution failures must use:

[AUTO_RESUME_ERROR] <code>: <summary>. Source=<source>. Fix: <action>.

Required codes:
- `INVALID_START_FROM`
- `RESUME_BRIEF_MISSING`
- `RESUME_BRIEF_STALE`
- `RESUME_BRIEF_UNPARSEABLE`
- `RESUME_STATE_CONFLICT`
- `STATE_PHASE_AMBIGUOUS`
- `STATE_PHASE_UNRECOVERABLE`

Bug-queue extensions (**`US-0087`**; same **`[AUTO_RESUME_ERROR]`** envelope when
used for resume/materialization failures):

- `AUTO_SCHEDULER_CONFLICT`
- `AUTO_BUG_QUEUE_EMPTY`
- `AUTO_BUG_TARGET_UNKNOWN`
- `AUTO_BUG_TARGET_NOT_OPEN`

## Steps (compact; full detail in reference)

1. Read automation flags from merged scratchpad and **materialize the resolved
   phase plan** per **Configurable phase selection policy (US-0070 / DEC-0052)**; append
   plan breadcrumbs to `docs/engineering/state.md` **before** first spawn.
2. Parse `start-from` / **`bug-target=`** / `--execute-bulk`; resolve scheduler
   mutex (**`AUTO_SCHEDULER_CONFLICT`** when applicable); resolve nominal start phase;
   intersect with plan.
3. Record continuation metadata (`invocation_mode=auto`, `requested_start_from`,
   `resolved_start_phase`, `resolution_source`, `resolution_status`, `timestamp`).
4. Spawn fresh subagents per intersected schedule; enforce **US-0069** preflight/post checks.
5. **Multi-phase continuation** (normative detail: **reference Step 5** in
   **`docs/engineering/auto-orchestration-reference.md`** `## Steps` item 5):
   advance through **all remaining phases** in the intersected resolved schedule
   order until a **deterministic stop condition** fires (see **Stop matrix** in
   `## Continuous multi-phase execution (US-0088)` above). When
   `AUTO_BACKLOG_DRAIN=1`, repeat the story lifecycle for the next eligible OPEN
   story, **reloading** scratchpad and **recomputing** the materialized phase
   plan at each story boundary. Outer-driver equivalence applies when a single
   invocation cannot schedule multiple subagent turns (**AC-1 Option B**).
   `stop_reason`: `completed|decision_gate|missing_input|pause_request|loop_max|error|blocked`.
6. Isolation evidence verification at each boundary (**reference** step 11a).
7. At each phase boundary, verify strict runtime attestation tuple exists
   and is valid for the completed phase (`orchestrator_run_id`, `runtime_proof_id`,
   `phase_id`, `role`, `proof_issued_at`, `proof_ttl_seconds`, `proof_hash`)
   (**reference** step 11b).
8. Sync verdict recording when eligible — reference step 12.
9. Backlog-drain / bulk per-item summaries when enabled — reference step 13.

## Backward compatibility

Default manual/interactive unchanged; `/resume` remains valid; deterministic precedence
applies for `/auto` continuation.

## Deterministic artifact ordering guard (US-0058 / DEC-0040)

Follow `docs/engineering/artifact-ordering-policy.md` (`state.md` append-bottom, etc.);
`ARTIFACT_ORDERING_ANCHOR_AMBIGUOUS` fail-closed.
