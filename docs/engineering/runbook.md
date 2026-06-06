# Runbook

## Commands

TEST_COMMAND: bash tests/run-tests.sh
LINT_COMMAND:
TYPECHECK_COMMAND:
DEPLOY_STAGING_COMMAND: echo "No staging deploy target configured for this repository"
DEPLOY_PROD_COMMAND: echo "No production deploy target configured for this repository"

LINT_FIX_COMMAND:
FORMAT_COMMAND:
CI_AUTO_FIX: false
TEST_TIMEOUT_SECONDS: 120

## Notes
- Leave a command blank to skip that step.
- Use explicit commands, not placeholders.
- `TEST_TIMEOUT_SECONDS` limits how long any subprocess can run during tests.
  Prevents hangs from prompts, network waits, or infinite loops.
- `LINT_FIX_COMMAND` / `FORMAT_COMMAND` are used by CI auto-fix when checks fail
  (e.g. `npx eslint --fix .` or `npx prettier --write .`).
- `CI_AUTO_FIX`: set to `true` to enable the automatic fix-and-retry loop in
  GitHub Actions. When `false` (default), CI reports failures but does not
  attempt auto-fix commits.

## Intentional empty commands (US-0015)

For this template/installer repository, the following command keys may be
intentionally empty in the shipped template; they are not configuration errors:

- `TEST_COMMAND` (blank until installer bootstrap per stack; **DEC-0056**)
- `LINT_COMMAND`
- `FORMAT_COMMAND`
- `TYPECHECK_COMMAND`

Teams may set these keys when needed for their own project stack.

## OS-aware runbook command bootstrap (US-0063 / DEC-0046)

Installer/upgrade flows auto-bootstrap runbook command keys with deterministic
precedence:

- `user override > detected defaults > safe fail-fast`
- user-provided non-empty values are never overwritten
- defaults are inferred from OS + project stack markers
  (`package.json` scripts, `pyproject.toml`, `go.mod`, platform test scripts)

Baseline detection contract:

- `TEST_COMMAND` is mandatory for push-eligible quality gates.
- `LINT_COMMAND` and `TYPECHECK_COMMAND` are optional and only auto-populated
  when confidently detectable.
- if `TEST_COMMAND` remains unresolved/invalid, installer fails fast with:
  - `[RUNBOOK_BOOTSTRAP_ERROR] TEST_COMMAND_UNRESOLVED`, or
  - `[RUNBOOK_BOOTSTRAP_ERROR] TEST_COMMAND_INVALID:<reason>`

Remediation:

- define `TEST_COMMAND` explicitly in `docs/engineering/runbook.md`, or
- add detectable stack markers/scripts then rerun installer upgrade.

## Codebase map bootstrap (US-0082 / DEC-0065)

**Goal:** `docs/engineering/codebase-map.md` exists in fresh repos without ad-hoc
operator memory, while **`/map-codebase`** stays the explicit manual analysis
command.

### Responsibility

| Path | Owner | Mechanism |
|------|-------|-----------|
| Primary | **`/architecture`** (tech-lead) | Before **`/sprint-plan`**, run `python scripts/materialize_codebase_map.py --trigger architecture` from repo root |
| Optional refresh | **`/refresh-context`** (curator) | Same script with `--trigger refresh-context` only when scratchpad sets **`CODEBASE_MAP_REFRESH_ON_ROLLOVER=1`** (default off) |
| Manual / deep pass | Operator | **`/map-codebase`** |

### Write surfaces

Same as **`/map-codebase`**: `docs/engineering/codebase-map.md`,
`docs/engineering/dependencies.json`. The materializer does **not** append
`docs/engineering/state.md`. Non-bootstrap maps (no bootstrap sentinel in the
file) are never replaced silently.

### Deterministic diagnostics

- **`CODEBASE_MAP_MISSING`** — use when a lifecycle checkpoint requires the map
  but it is absent and generation did not run (e.g. custom **`/auto`** profile
  skipped **`architecture`**).
- **`CODEBASE_MAP_BLOCKED:<subreason>`** — materializer or policy blocked
  creation (`policy_skip`, permissions, etc.); stdout includes remediation
  pointing here and to **`/map-codebase`**.

**Command:** `python scripts/materialize_codebase_map.py --repo .`  
**Tests:** `python tests/codebase_map_materialize_test.py`

Normative architecture: `docs/engineering/architecture.md` (**# US-0082**).

## Documentation profile validation (US-0077 / DEC-0059)

**Goal:** keep root `README.md` (user channel) and `docs/developer/README.md`
(developer shard) aligned with merged scratchpad keys `DOC_AUDIENCE_PROFILE` and
`DOC_DETAIL_LEVEL`, with deterministic reason codes and active/`template/` parity.

### Scratchpad keys

- `DOC_AUDIENCE_PROFILE`: `user` \| `developer` \| `both` (empty defaults to `both` during transition).
- `DOC_DETAIL_LEVEL`: `concise` \| `balanced` \| `technical-deep` (empty defaults to `balanced`).
- Invalid values → `DOC_PROFILE_INVALID`. Merge/read failures → `DOC_PROFILE_MERGE_ERROR`.
- Optional modes `SPEC_PACK_MODE` / `USER_GUIDE_MODE` stay additive only: when `0`, this
  validator does not require spec-pack or user-guide files.

### Command

```bash
python scripts/validate_doc_profile.py --repo .
python scripts/validate_doc_profile.py --repo . --no-template-parity   # fixture trees without template/
```

### Installer hook

`installer.py` scratchpad post-install refreshes missing normative `##` sections
(non-destructive append) from the resolved profile, then operators should keep
content accurate. Re-run `python installer.py --scratchpad-postinstall --target <repo> --mode missing`
after template upgrades if needed.

Normative H2 titles and matrix: `docs/engineering/architecture.md` (`# US-0077`).

## User-visible internal metadata guard (US-0071 / DEC-0053)

**Goal:** keep planning-shaped identifiers out of **operator-visible software
channels** (CLI/installer/validate-and-push strings), while they remain valid in
internal documentation trees and in source comments that are not emitted to
users.

### Forbidden tokens (user-visible channels only)

Match planning-shaped tokens:

- `US-[0-9]{4}`
- `DEC-[0-9]{4}`
- `R-[0-9]{4}`

### Inclusive scan roots (deterministic)

From repository root, the checker walks **only**:

- `bin/**` (`*.js`)
- `installer.py`, `installer.ps1`, `installer.sh`
- `packaging/**` (`*.js`, `*.py`, `*.ps1`, `*.sh`)
- `scripts/validate-and-push.ps1`, `scripts/validate-and-push.sh`

Paths outside this set are **not** scanned by this tool (for example `docs/**`,
`.cursor/**`, `sprints/**`, `handoffs/**`, `decisions/**` remain free to use
story/decision/research IDs). If a new operator-visible deliverable is added
outside these roots, extend the scan list or you risk
`METADATA_SANITIZATION_SCOPE_AMBIGUOUS` classification during review.

### Command

```bash
python scripts/check-user-visible-metadata.py
python scripts/check-user-visible-metadata.py --json
```

### Reason codes (minimum)

- `USER_VISIBLE_INTERNAL_METADATA_DETECTED` — forbidden token matched inside a
  scanned user-visible string/literal.
- `METADATA_SANITIZATION_POLICY_MISSING` — checker entrypoint missing or
  unusable.
- `METADATA_SANITIZATION_SCOPE_AMBIGUOUS` — cannot classify whether a path
  belongs in inclusive scan roots; treat as fail-closed until the runbook table
  is updated.

### Findings / remediation (contract)

On failure, diagnostics must cite **evidence_ref** (`path:line:column` when
available), **token class** (`US` \| `DEC` \| `R`), and remediation: remove the
token from operator-visible strings; keep traceability in allowlisted internal
artifacts or non-emitting comments per `DEC-0053`.

## Guided intake mode (US-0033)

Intake interaction behavior is controlled by one switch in
`.cursor/scratchpad.md`:

- `INTAKE_GUIDED_MODE=1` (default): guided PO behavior
  - targeted follow-up questions only when acceptance is ambiguous
  - at least one viable option/alternative before recommendation
  - explicit user decision authority
  - intake-time research persisted in `docs/engineering/research.md`
- `INTAKE_GUIDED_MODE=0`: low-touch intake
  - no proactive follow-up/options/research overhead unless user asks
  - duplicate/overlap backlog check remains mandatory baseline safety

## Intake decomposition and risk-aware questioning (US-0051)

When guided mode is enabled (`INTAKE_GUIDED_MODE=1`), intake adds bounded
decomposition and adaptive questioning behavior:

- Run deterministic breadth/risk heuristics before persisting a story:
  - feature/workflow-step count
  - cross-cutting impact surface
  - acceptance breadth
  - risk/unknown dependency surface
- If heuristics indicate broad/high-risk intake:
  - propose bounded multi-story decomposition (typically 2-5 stories)
  - prefer vertical-slice/workflow-step stories with independent user value
  - avoid technical-layer-only splits unless user explicitly requests
- Preserve user authority explicitly before persistence:
  - user can accept, merge, or adjust the proposed split
- Keep adaptive questioning concise and bounded:
  - ask ambiguity-driven questions plus risk-triggered questions
  - stop after bounded rounds or when acceptance confidence is sufficient
- Low-touch compatibility (`INTAKE_GUIDED_MODE=0`):
  - no forced decomposition
  - single-story default unless user explicitly asks for decomposition
  - duplicate/overlap safety remains mandatory
- Traceability requirement:
  - intake output must capture decomposition/questioning evidence in
    `docs/product/backlog.md`, `docs/product/acceptance.md`, and
    `handoffs/po_to_tl.md`.

## Mandatory intake question packs and persistence coverage gate (US-0068 / DEC-0050)

Intake persistence is fail-closed unless required topic coverage is complete (or
bounded assumptions are explicitly confirmed).

Deterministic pack contract:

- `first-intake-pack` (first/new/broad intake)
  - required topics:
    - `users_problem`
    - `runtime_target_environment`
    - `language_framework_runtime`
    - `architecture_preference`
    - `ui_design_expectations`
    - `security_compliance`
    - `non_functional_priorities`
    - `scope_timeline`
- `small-intake-pack` (small follow-up intake)
  - required topics:
    - `outcome_success_criteria`
    - `impacted_components`
    - `constraints_compatibility_risks`
    - `required_tests_acceptance_checks`
    - `done_definition`

Pack selection and coverage behavior:

- Select exactly one pack per intake write path.
- Unknown/ambiguous stack or project cues must fail closed to
  `first-intake-pack`.
- Required coverage must be evaluated before writing
  `docs/product/backlog.md` or `docs/product/acceptance.md`.
- Incomplete required coverage blocks persistence unless assumptions are
  explicitly confirmed.

Deterministic fail-closed reason codes:

- `INTAKE_REQUIRED_TOPIC_MISSING`
- `INTAKE_REQUIRED_PACK_INCOMPLETE`
- `INTAKE_ASSUMPTION_CONFIRMATION_REQUIRED`
- `INTAKE_PERSISTENCE_BLOCKED`

Required remediation output on block:

- include `missing_topics`
- provide targeted follow-up prompts for missing required topics
- request explicit assumption confirmation when assumptions are used

Required persisted intake evidence fields:

- `asked_topics`
- `missing_topics`
- `assumptions_confirmed`

## First-intake full-plan coverage gate (US-0081 / DEC-0064)

For first/new/broad intake (`selected_pack=first-intake-pack`), persistence is
additionally blocked unless complete-plan coverage is machine-verifiable.

Required coverage contract fields:

- `plan_area_inventory[]` with unique stable `plan_area_id` values
- `plan_area_coverage[]` with exactly one row per `plan_area_id`
- xor mapping per row: `story_ids[]` or `deferred_ref` + `deferred_reason`
- `coverage_complete=true` only when derived validation succeeds

Coverage diagnostics (under umbrella `INTAKE_PERSISTENCE_BLOCKED`):

- `INTAKE_PLAN_COVERAGE_MISSING`
- `INTAKE_PLAN_AREA_ID_INVALID`
- `INTAKE_PLAN_COVERAGE_CONTRACT_INVALID`
- `INTAKE_PLAN_DEFERRED_REF_MISSING`

Guided and low-touch parity:

- `INTAKE_GUIDED_MODE=1` and `INTAKE_GUIDED_MODE=0` must run the same
  first-intake complete-plan validator path.
- Low-touch may reduce optional prompts but cannot bypass complete-plan coverage
  validation.

## Interactive intake evidence validation (US-0078 / DEC-0060 / US-0083 / DEC-0067)

- Interactive intake evidence validation (US-0078 / DEC-0060) — automation/harness anchor; extended rules for **US-0083** / **DEC-0067** follow in this section.

**US-0078** adds machine-verifiable **`topic_coverage`** rows, canonical **`ie:`** refs
(**DEC-0060**), asked-vs-covered enforcement, and **`assumption_confirmation_ref`**
binding before backlog/acceptance writes.

- Validator entrypoints: `python scripts/intake_evidence_validate.py --self-test`;
  `python scripts/intake_evidence_validate.py --file <bundle.json>` or `--stdin`.
- Library: `scripts/intake_evidence_lib.py` (shared rules for tests and tooling).
- Regression: `tests/intake_evidence_fixtures_test.py` (R-0055 **AC-8** matrix tiers A/B),
  invoked from `tests/run-tests.ps1` / `tests/run-tests.sh` §26k.
- **Packaged installs (BUG-0001 / DEC-0063)**: `intake_evidence_validate.py`, `intake_evidence_lib.py`, and `intake_bug_routing_guard.py` are mirrored under `template/scripts/` and listed in `docs/engineering/context/installer-owned-paths.manifest` so fresh install and `upgrade` copy them to the consumer’s `scripts/`. Drift guard: `python scripts/check_intake_template_parity.py --repo .` (also §26N in `tests/run-tests.*`). **Release (S0060)**: operator notes `handoffs/releases/S0060-release-notes.md` (gate summary + verify steps).
- **US-0084**: `remote_config_summary.py` and `guard_installer_publish.py` use the same **`template/scripts/`** mirror + manifest rows; npm **`package.json` `files`** also lists the active copies for publish.
- **Installer completeness gate (BUG-0003 / DEC-0066)**: post-install invariant checks every path in `[required_install_script_paths]` from `docs/engineering/context/installer-owned-paths.manifest`. Missing paths fail closed with `INSTALL_COMPLETENESS_FAILED` and `INSTALL_REQUIRED_SCRIPT_MISSING:<path>`. Remediation: update manifest parity (active + `template/`), ensure required script exists under `template/scripts/`, keep install/clean ownership paired, then rerun `its-magic --mode missing|upgrade` (or `python installer.py --validate-install-completeness --target <repo>` for direct diagnostics).
- **Guided** and **low-touch** (`INTAKE_GUIDED_MODE=0`) share the **same** pre-persistence
  validation pipeline; mandatory pack evidence is never skipped.
- Legacy intake evidence without **`ie:`** refs remains **grandfathered** for display until the
  next intake-driven mutation, which must supply full evidence (**DEC-0060** §5).
- **Delegated required-topic path (US-0083 / DEC-0067)**:
  - Allowed: `topic_coverage[].satisfied_by=delegation_ref` with required
    `delegation_scope`, `delegation_rationale`, `delegation_confidence` (`low|medium|high`).
  - Missing delegation fields fail closed with `INTAKE_DELEGATION_EVIDENCE_MISSING`.
  - Malformed delegation values or invalid `ie:` binding fail closed with
    `INTAKE_DELEGATION_EVIDENCE_INVALID`.
  - Non-delegated unresolved required topics remain unchanged fail-closed
    (`INTAKE_REQUIRED_TOPIC_MISSING` path).
- **Repetitive-ask suppression with accounting (US-0083 AC-1)**:
  - When equivalent evidence already exists, avoid re-asking by recording row-level
    `evidence_source=equivalent_evidence_ref` plus `equivalent_evidence_ref`.
  - Required-topic accounting remains explicit through `topic_coverage` rows.

## Bug issues (US-0079 / DEC-0061)

- **Canonical ids**: **`BUG-####`** in **`docs/product/backlog.md`** **`## Bug issues (canonical)`**; status literals **`OPEN`** | **`DONE`** only — illegal values fail **`BUG_VALIDATION_STATUS_INVALID`**.
- **Minimum fields** (non-empty): **`environment`**, **`steps_to_reproduce`**, **`expected`**, **`actual`**, **`evidence_refs`** — missing/empty → **`BUG_VALIDATION_FIELD_EMPTY`** (or **`BUG_VALIDATION_SECTION_MISSING`** when the region is absent).
- **Ordering**: bug blocks sorted by id ascending — violation → **`BUG_VALIDATION_ORDER_INVERSION`**.
- **Intake routing**: merged **`INTAKE_WORK_ITEM_KIND=story|bug`** and/or explicit **`/intake bug`**; defect-shaped prose with **`story`** kind → **`INTAKE_BUG_ROUTING_REQUIRED`** via **`python scripts/intake_bug_routing_guard.py`** (**DEC-0061** §5). Mismatch/conflict → **`INTAKE_WORK_ITEM_KIND_MISMATCH`** family (documented in command surfaces).
- **Acceptance reconciliation**: **`docs/product/acceptance.md`** **`## Bug acceptance (canonical)`** checkbox rows must match backlog bug status — drift codes **`BUG_RECONCILE_ACCEPTANCE_*`**.
- **Commands**:
  - `python scripts/bug_issue_validate.py --self-test`
  - `python scripts/bug_issue_validate.py --backlog docs/product/backlog.md [--check-acceptance] [--print-next-id]`
  - `python scripts/intake_bug_routing_guard.py --kind story|bug --file <path>` (or **`--stdin`**)
- **Regression**: `tests/bug_issue_fixtures_test.py` (R-0056 Tier A/B), invoked from **`tests/run-tests.ps1` / `tests/run-tests.sh`** §26L.

## Optional ID namespace bootstrap (US-0052)

Fresh-project ID bootstrap is optional and default-off in
`.cursor/scratchpad.md`:

- `ID_NAMESPACE_BOOTSTRAP=0|1` (default `0`)

Deterministic behavior:

- If `ID_NAMESPACE_BOOTSTRAP=1`, evaluate freshness eligibility before creating
  new IDs:
  - no `US-` IDs in `docs/product/backlog.md`
  - no `DEC-` IDs in `docs/engineering/decisions.md` (and no existing
    `decisions/DEC-*.md`)
  - no `R-` IDs in `docs/engineering/research.md`
- If eligible, first created IDs start at:
  - `US-0001` for intake stories
  - `DEC-0001` for architecture decisions
  - `R-0001` for research entries
- If not eligible (or mode is off), continue from highest existing ID in each
  namespace.
- Never rewrite/renumber historical IDs.
- If bootstrap is requested but ineligible, emit deterministic diagnostic
  `ID_BOOTSTRAP_NOT_FRESH` and continue with highest-existing continuation.

## Context compaction and token profile mode (US-0053 / DEC-0035)

Tiered token-cost control is explicit and defaulted in `.cursor/scratchpad.md`:

- `TOKEN_PROFILE=lean|balanced|full` (default `balanced`)

Deterministic profile semantics:

- `lean`: reduce non-critical overhead defaults (for example aggressive research,
  autonomous loops, broad-context retrieval), while preserving mandatory
  quality/release gates.
- `balanced`: preserve current capability profile with moderate overhead.
- `full`: maximize context breadth and autonomy for complex/high-uncertainty work.

Manual override precedence:

- Explicit flag values remain authoritative for that flag.
- If a flag is explicitly set, it overrides profile defaults.
- Profile changes must not disable mandatory gate contracts
  (`/qa`, `/verify-work`, `/release`).

### Token-cost evidence + comparability (US-0080 / DEC-0062)

- **Fresh context**: spawn **new** subagents per `/auto` phase; avoid carrying prior chat
  reasoning as phase input.
- **`start-from`**: use **`/auto start-from=<canonical_phase_id>`** when resuming so the
  schedule intersection matches materialized **`resolved_phase_plan`** (**`DEC-0052`**).
- **`TOKEN_PROFILE`**: `lean` lowers default automation breadth; does **not** remove
  isolation, strict-proof, role, or release gates.
- **Metrics**: append-only **`handoffs/token_cost_runs/<orchestrator_run_id>.md`** (or
  **`.jsonl`**); copy path into **`token_cost_evidence_ref`** on **`state.md`** checkpoints.
- **AC-2**: compare **`cache_read_tokens`** only when **`run_class_hash`** matches; else
  **`TOKEN_COST_RUN_CLASS_MISMATCH`**.
- **CI/repo checks**: `python scripts/check_token_cost_parity.py --repo .` (manifest-listed
  active/`template/` pairs); **`tests/run-tests.ps1`** / **`tests/run-tests.sh`** §26M.

Context compaction policy:

- `docs/engineering/state.md` is a compact hot surface for current execution
  context and recent checkpoints.
- Historical state packs belong in `docs/engineering/state-archive/` and are
  append-only/non-destructive.
- `docs/engineering/decisions.md` is a compact index with bounded summaries and
  canonical links to full records in `decisions/DEC-xxxx.md`.
- Enforced rollover thresholds:
  - `STATE_HOT_MAX_LINES` (default `1200`)
  - `STATE_HOT_MAX_CHECKPOINTS` (default `80`)
  - `PO_TO_TL_HOT_MAX_LINES` (default `800`)
  - `PO_TO_TL_HOT_MAX_SECTIONS` (default `60`)
  - `ARCH_HOT_MAX_LINES` (default `3500`)
  - `ARCH_HOT_MAX_STORY_SECTIONS` (default `120`)
  Thresholds resolve from merged `.cursor/scratchpad.md` +
  `.cursor/scratchpad.local.md` (DEC-0054 triad contract).
  When a cap is exceeded, the mutating phase must run rollover **before**
  completion or fail closed (no successful completion with an oversize hot
  surface).

### Triad hot-surface enforcement (DEC-0054)

Canonical hot/archive surfaces:

- `docs/engineering/state.md` → `docs/engineering/state-archive/state-pack-*.md`
- `handoffs/po_to_tl.md` → `handoffs/archive/po-to-tl-pack-*.md`
- `docs/engineering/architecture.md` →
  `docs/engineering/architecture-archive/architecture-pack-*.md`

Operator commands:

```bash
python scripts/enforce-triad-hot-surface.py --check
python scripts/enforce-triad-hot-surface.py --rollover
```

- `--check` verifies all three surfaces are within policy (CI-safe).
- `--rollover` archives oldest contiguous units into the next deterministic pack
  name; reruns are idempotent when already within caps.
- Successful rollover records a verification tuple:
  `boundary`, `moved`, `retained` (counts / lines), `pack_ref`.

Rollover fail-safe reason codes:

- `STATE_ARCHIVE_BOUNDARY_AMBIGUOUS`
- `STATE_ARCHIVE_WRITE_FAILED`
- `STATE_ARCHIVE_VERIFICATION_FAILED`
- `STATE_ARCHIVE_REQUIRED`
- `ARTIFACT_HOT_SURFACE_OVERSIZE`
- `CONTEXT_BUDGET_EXCEEDED`

### Minimal-read defaults by phase (bounded escalation)

Read `docs/engineering/phase-context.md` first, then the **required** paths for
your phase. If unresolved, expand once to the **single** archive pack named in
the latest verification tuple for that surface. Do not load entire archive
directories by default.

| Phase | Required reads (default) | Combined line budget (guidance) |
|-------|--------------------------|----------------------------------|
| `/intake` | `phase-context.md`, target story in `docs/product/backlog.md`, `handoffs/po_to_tl.md` (tail) | ≤ 900 lines |
| `/discovery` | `phase-context.md`, `docs/product/vision.md` (story notes), `handoffs/po_to_tl.md` (tail) | ≤ 900 lines |
| `/research` | `phase-context.md`, `docs/engineering/research.md` (target entry), `docs/product/backlog.md` (target story) | ≤ 800 lines |
| `/architecture` | `phase-context.md`, `docs/engineering/architecture.md` (target story section), `docs/engineering/research.md` | ≤ 1200 lines |
| `/sprint-plan` | `phase-context.md`, `docs/engineering/architecture.md` (target story), `handoffs/tl_to_dev.md` | ≤ 1000 lines |
| `/plan-verify` | `phase-context.md`, `sprints/Sxxxx/tasks.md`, `docs/product/backlog.md` (ACs) | ≤ 900 lines |
| `/execute` | `phase-context.md`, `sprints/Sxxxx/tasks.md`, `handoffs/tl_to_dev.md` | ≤ 800 lines |
| `/qa` | `phase-context.md`, `sprints/Sxxxx/`, `tests/report.md` | ≤ 900 lines |
| `/verify-work` | `phase-context.md`, `sprints/Sxxxx/uat.json`, QA findings | ≤ 600 lines |
| `/release` | `phase-context.md`, release queue + sprint release findings | ≤ 700 lines |
| `/refresh-context` | `phase-context.md`, `docs/engineering/state.md` (tail), `docs/product/backlog.md` (status) | ≤ 900 lines |
| `/auto` (resolver) | `phase-context.md`, `handoffs/resume_brief.md`, `docs/engineering/state.md` (tail) | ≤ 700 lines |

If the default set is insufficient, escalate with an explicit note citing
`pack_ref`. Unbounded broad reads fail closed with `CONTEXT_BUDGET_EXCEEDED`.

`/ask` retrieval policy:

- Use question-scoped narrow reads first.
- Expand context in bounded steps only when unresolved.
- If unresolved after bounded expansion, answer with explicit "not found in
  current artifacts" rather than broad speculative reads.

## Configurable multi-target publish mode (US-0054 / DEC-0036)

Post-release publish orchestration is configurable and default-safe:

- `RELEASE_PUBLISH_MODE=disabled|confirm|auto` (default `confirm`)
- `RELEASE_TARGETS_FILE=docs/engineering/release-targets.json`
- `RELEASE_TARGETS_DEFAULT=` optional comma-separated default target IDs

Target schema contract:

- Canonical target config file: `docs/engineering/release-targets.json`
- Supported target types:
  - `npm`, `choco`, `brew`, `git`, `docker`, `cloud`
  - `custom` (generic command target)
  - `ssh` (host/user/port/auth reference + remote command)
- Connectivity metadata (for operator-safe remote/local context):
  - `runtime.mode` (`local|remote`)
  - endpoint fields (`domainEnv|ipEnv|hostEnv`, `port`, `protocol`)
  - optional ingress metadata (`traefik.enabled`, `router`, `entrypoint`, `tls`)
  - optional `dockerOverSsh` object for ssh/dockerd remote execution context
- Each target entry must define deterministic fields:
  - `id` (stable unique target ID)
  - `type`
  - `enabled` (`true|false`)
  - `order` (deterministic execution ordering)
  - execution details (`command` for non-ssh, `remoteCommand` + host/user/auth refs for `ssh`)

Safety contract:

- Mandatory release gates remain unchanged and must pass before any publish
  target execution.
- `confirm` mode requires explicit operator approval before publish execution.
- Sensitive fields must be env-referenced (`*Env` keys); inline secret literals
  are not allowed.
- Invalid target config must fail fast with deterministic diagnostics and no
  partial side effects.
- Invalid remote connectivity metadata must fail fast with
  `REMOTE_CONNECTIVITY_CONFIG_INVALID`.
- Canonical operator endpoint summary is written to
  `docs/engineering/runtime-connectivity.md` with sanitized values only.

## Release operator hints contract (US-0067 / DEC-0049)

Release outputs must include deterministic operator-ready hints with mandatory
section order:

`Run -> Connect -> Verify -> Credentials -> Known Issues`

Required fields for canonical sprint notes
(`handoffs/releases/Sxxxx-release-notes.md`):

- `Run`: `start_command`, `runtime_mode`, `runtime_context_ref`
- `Connect`: `service_url`, `service_port`, `health_endpoint`
- `Verify`: deterministic `verification_steps`, `expected_health_signal`
- `Credentials`: env-reference-only source refs and expected value-source
  location guidance (never inline secrets)
- `Known Issues`: concise issue list or explicit `None`

Legacy pointer contract (`handoffs/release_notes.md`):

- keep concise latest run/connect/verify summary only
- always link to canonical sprint-scoped release notes for full details

Fail-closed reason codes:

- `RELEASE_OPERATOR_HINTS_MISSING`
- `RELEASE_OPERATOR_HINTS_AMBIGUOUS`
- `RELEASE_OPERATOR_HINTS_SECRET_EXPOSURE`

## Deterministic status reconciliation mode (US-0055 / DEC-0037)

Use the dedicated reconciliation command to normalize status drift across
canonical and derived artifacts:

- Command: `/status-reconcile`
- Canonical source: `docs/product/backlog.md` (story `Status`)
- Derived surfaces: `docs/product/acceptance.md`, `docs/engineering/state.md`,
  `handoffs/resume_brief.md`

Deterministic behavior:

- Detects mismatches (for example DONE + unchecked ACs, acceptance drift, resume drift).
- Applies target-scoped reconciliation only to mismatched story blocks/rows.
- Preserves canonical ownership; derived artifacts reconcile to backlog status.
- Updates `handoffs/resume_brief.md` to next OPEN story and intended phase.
- Writes auditable rows to `docs/engineering/status-normalization-report.md`.

Reason-code baseline:

- `STATUS_RECONCILE_APPLIED`
- `STATUS_RECONCILE_NOOP`
- `STATUS_RECONCILE_MISSING_INPUT`
- `STATUS_RECONCILE_CANONICAL_CONFLICT`
- `STATUS_RECONCILE_PHASE_AMBIGUOUS`
- `STATUS_RECONCILE_EVIDENCE_MISSING`

## Optional cross-repo observability mode (US-0034)

Compatibility visibility is optional and default-off in `.cursor/scratchpad.md`:

- `CROSS_REPO_OBSERVABILITY=0|1` (default `0`)
- `COMPATIBILITY_GATE_ON_CRITICAL=0|1` (default `1`)
- `COMPATIBILITY_SOURCES=` monitored source declarations

Default-off behavior:
- With `CROSS_REPO_OBSERVABILITY=0`, `/intake`, `/architecture`, `/execute`,
  and `/qa` add zero required compatibility overhead.

Enabled behavior (`CROSS_REPO_OBSERVABILITY=1`):
- Use canonical artifacts:
  - `docs/engineering/compatibility-report.md`
  - `docs/engineering/compatibility-signals.md`
  - `docs/engineering/manifests/registry.manifest.yaml`
  - `docs/engineering/manifests/repo.manifest.yaml`
- Record findings with severity, affected modules, evidence refs, and
  recommended actions.
- If unresolved critical findings exist and
  `COMPATIBILITY_GATE_ON_CRITICAL=1`, trigger decision gate before release
  progression (`COMPATIBILITY_CRITICAL_OPEN`).

## Optional component-scoped execution mode (US-0035)

Component-scoped execution is optional and default-off:

- `COMPONENT_SCOPE_MODE=0|1` (default `0`)
- `TARGET_COMPONENTS=` comma-separated scoped component IDs

Default-off behavior:
- With `COMPONENT_SCOPE_MODE=0`, workflow phases add zero required scope
  overhead.

Enabled behavior (`COMPONENT_SCOPE_MODE=1`):
- Declare scope in `docs/engineering/component-scope.md`:
  - `target_components[]`
  - `non_target_components[]`
  - `allowed_interface_touch[]`
- `/sprint-plan` tasks declare `target_component_ids` and
  `expected_impacted_interfaces`.
- `/execute` enforces scope-first behavior.
- `/qa` verifies unaffected-component checks and records evidence in
  `docs/engineering/component-scope-report.md`.
- If unapproved out-of-scope impact remains open, release must stop at decision
  gate (`COMPONENT_SCOPE_VIOLATION_UNAPPROVED`).

## Optional spec-pack documentation mode (US-0031)

Spec-pack mode is optional and default-off in `.cursor/scratchpad.md`:

- `SPEC_PACK_MODE=0|1` (default `0`)

Default-off behavior:
- With `SPEC_PACK_MODE=0`, `/intake`, `/architecture`, `/execute`, `/qa`, and
  `/release` add no required spec-pack steps (zero overhead).

Enabled behavior (`SPEC_PACK_MODE=1`):

**Canonical names and locations** (per story):
- Design Concept: `docs/engineering/spec-pack/<story_id>-design-concept.md`
- CRS (Customer/Product Requirements Summary): `docs/engineering/spec-pack/<story_id>-crs.md`
- Technical Specification: `docs/engineering/spec-pack/<story_id>-technical-specification.md`

**Traceability**: Backlog story ID (e.g. `US-0031`) maps 1:1 to the three
artifacts above. Handoffs and state should reference these paths when
spec-pack mode is enabled.

**Minimum required sections** (completeness is testable; validation blocks
only when enabled and a required section is missing or empty):

- Design Concept: `# Summary`, `# Goals`, `# Non-goals`, `# Key decisions`
- CRS: `# Purpose`, `# Scope`, `# Acceptance criteria ref`
- Technical Specification: `# Overview`, `# Components`, `# Interfaces`, `# Non-functional`

**Validation**: When `SPEC_PACK_MODE=1`, release gate checks that for the
target sprint story, all three artifacts exist and each required section
above is present and non-empty. If not, release is blocked with reason code
`SPEC_PACK_INCOMPLETE` and remediation guidance.

**Ownership (role/phase)**:
- Design Concept: Tech Lead, `/architecture` (create/update).
- CRS: PO, `/intake` (create/update for new story); Tech Lead may extend in
  architecture.
- Technical Specification: Tech Lead, `/architecture` (create); Dev, `/execute`
  (update when implementation details change).

## Optional user-guide documentation mode (US-0032)

User-guide mode is optional and default-off in `.cursor/scratchpad.md`:

- `USER_GUIDE_MODE=0|1` (default `0`)

Default-off behavior:
- With `USER_GUIDE_MODE=0`, `/intake`, `/architecture`, `/sprint-plan`, `/execute`,
  `/qa`, and `/release` add no required user-guide steps or blocking checks (zero overhead).

Enabled behavior (`USER_GUIDE_MODE=1`):

**Canonical location and naming** (per feature story):
- One guide per feature story: `docs/user-guides/US-xxxx.md` (e.g. `docs/user-guides/US-0032.md`).
- Story ID `US-xxxx` is the stable identifier; create/update the guide when the story is in scope.

**Minimum required schema** (structural validation only; completeness is testable):
- `# Purpose`
- `# Prerequisites`
- `# Usage steps`
- `# Example`
- `# Limitations`
- `# Troubleshooting`

**Traceability**: Story ID maps 1:1 to the user-guide artifact. Handoffs and release
context should reference `docs/user-guides/US-xxxx.md` for the target story when
user-guide mode is enabled.

**Validation**: When `USER_GUIDE_MODE=1`, release gate checks that for the target
sprint story, the guide file exists at the canonical path and each required section
above is present and non-empty. If not, release is blocked with reason code
`USER_GUIDE_INCOMPLETE` and remediation guidance (create or complete the guide).

**Boundary with spec-pack (US-0031)**: User guides are end-user facing how-to
documentation only. They do not duplicate Design Concept, CRS, or Technical
Specification content; user guides may reference spec-pack artifacts but must not
replicate their ownership or technical scope. See runbook/README separation guidance.

## Legacy DONE-story drift detection and guard (US-0049)

Stories that are DONE in backlog but lack aligned acceptance/traceability or
release representation are in **legacy drift**. US-0049 adds detection, bounded
repair, and an ongoing guard at release/reconciliation (DEC-0031).

**Detection rule** — A story is in legacy drift when:
- Backlog status is **DONE**, and
- At least one of:
  - Acceptance checklist item for that story is **unchecked**
  - Traceability index or `docs/engineering/state.md` **lacks an entry** for that story
  - Release artifacts (e.g. `handoffs/releases/Sxxxx-release-notes.md`, queue row)
    **lack clear representation** for that story

**Bounded repair**: Only stories matching the rule above may be mutated; no broad
rewrite of unrelated backlog/acceptance/state/release artifacts.

**Canonical audit artifact**: `docs/engineering/legacy-drift-audit.md`
- Required fields per entry: story ID, prior acceptance state, prior traceability
  state, resolved state(s), reason code, evidence reference.
- Append-only; one-time backfill and ongoing guard append entries when drift is
  detected and repaired (or when guard blocks and reports).

**Reason-code vocabulary** (with remediation):
- `BACKLOG_DONE_ACCEPTANCE_UNCHECKED` — Backlog DONE but acceptance item unchecked.
  Remediation: set acceptance checkbox from canonical release/state evidence or run one-time backfill.
- `BACKLOG_DONE_TRACEABILITY_MISSING` — Backlog DONE but traceability/state lacks entry.
  Remediation: add traceability row in `docs/engineering/state.md` from backlog/release evidence or run backfill.
- `BACKLOG_DONE_RELEASE_ARTIFACT_MISSING` — Backlog DONE but release artifacts lack representation.
  Remediation: ensure release notes or queue row exists for the story’s sprint or run backfill.

**One-time backfill mode**: Explicit trigger (e.g. dedicated check or `/memory-audit`-related path).
- Run detection once over all DONE stories; for each legacy-drift story, perform
  target-scoped repair and append an entry to `docs/engineering/legacy-drift-audit.md`.
- Idempotent when no drift: no mutations; report empty or "no drift".
- Only stories matching the detection rule are mutated.

**Ongoing guard**: At release or reconciliation boundary (or dedicated check).
- When legacy drift is detected, either **block** with explicit reason code and
  remediation, or **repair** target-scoped and append audit entry (policy documented).
- Behavior is deterministic; operators get explicit diagnostics.

## Memory drift auditing

Run `/memory-audit` at key workflow checkpoints to verify artifact consistency:

- **Pre-handoff**: before writing `handoffs/dev_to_qa.md` or any role handoff.
- **Pre-QA**: before running `/qa` or `/verify-work`.
- **Pre-release**: before running `/release`.
- **Ad-hoc**: after external code changes, long pauses, or whenever artifacts
  feel stale.

Output: `docs/engineering/memory-drift-report.md` — an advisory report with
severity-classified findings. The command is read-only and non-blocking.

Interpreting results:
- **high**: artifact contradicts repository state — fix before next handoff/release.
- **medium**: artifact is likely stale — fix before release.
- **low**: minor inconsistency — fix during `/refresh-context` or next sprint.

Template drift findings (active vs `template/`) are listed for reference only
and belong to US-0017 scope.

Follow-up commands: `/refresh-context`, `/sprint-plan`, `/verify-work`, `/intake`.

## Remote execution validation contract

Remote execution is mode-aware and default-off:

- `REMOTE_EXECUTION=0`: skip remote-config validation entirely (zero overhead).
- `REMOTE_EXECUTION=1`: validate `.cursor/remote.json` before remote activities;
  fail fast on first blocking issue.

Validation classes (remote-enabled mode):

1. Presence: config file exists at `REMOTE_CONFIG` (default `.cursor/remote.json`)
2. Syntax: JSON parses cleanly
3. Contract: required fields/types/enums
4. Semantics: `defaultTarget` points to an existing enabled target; target ids
   are unique
5. Security: no inline secret-like literals; env-var refs only for sensitive values

Required contract summary:

- Root: `version` (integer), `defaultTarget` (string), `targets` (array)
- Target: `id` (string), `type` (`docker|ssh|vm`), `enabled` (boolean),
  `host` (string), `port` (integer `1..65535`), `workspaceRoot` (string)
- Optional auth: `auth.mode` (`none|env`); if `env`, use `*Env` references

Error message format (actionable, fail-fast):

- `[REMOTE_CONFIG_ERROR] <path>: expected <rule>, got <actual>. Fix: <hint>.`

Operator troubleshooting:

- Missing config file:
  - Copy from `template/.cursor/remote.json`, or disable remote mode.
- Malformed JSON:
  - Fix syntax (commas/brackets/quotes), then retry.
- Invalid value or enum:
  - Correct field value to the documented contract.
- Security violation (inline secret-like literal):
  - Replace with env-var reference fields (`tokenEnv`, `passwordEnv`,
    `privateKeyPathEnv`, ...).

### Manual vs automation routing (US-0086)

Manual and automation modes are intentionally separate:

- Manual mode (`AUTO_REMOTE_AUTOMATION_PROFILE=off`) keeps local-first behavior.
  No automatic remote routing is allowed, and `TEST_COMMAND` is never silently
  rerouted to remote targets.
- Automation mode (`AUTO_REMOTE_AUTOMATION_PROFILE=deterministic_v1`) may route
  to Docker/SSH/local targets using deterministic precedence.
- Explicit NL intent literal is constrained to `start container <target_id>`.
  Unknown or disabled targets fail closed.

Deterministic fail-closed reason codes:

- `REMOTE_AUTOMATION_MODE_OFF`
- `REMOTE_TARGET_UNKNOWN`
- `REMOTE_TARGET_DISABLED`
- `REMOTE_TARGET_UNROUTABLE`

Security continuity (`US-0085` / `DEC-0071`) remains mandatory in all modes:

- Never read `.env` from agent automation.
- Never print secret values in command output, logs, handoffs, or state.
- Names-only evidence format is required (`secret_surface=names_only`).

### Remote-routing evidence tuple (execute/qa/release)

When automation routing is used, include this tuple in handoffs/state artifacts:

- `target_id`
- `environment_label`
- `automation_profile`
- `routing_source` (`explicit_intent|heuristic_fallback|local_default`)
- `secret_surface=names_only`

If routing is not used (mode off/local default), still record:

- `target_id=local-default`
- `environment_label=local`
- `automation_profile=off`
- `routing_source=local_default`
- `secret_surface=names_only`

### Published npm `installer.sh` / POSIX dash (US-0084)

- **Symptom**: `set: Illegal option -` on an early line when running `its-magic` or
  `sh installer.sh` on Debian/Ubuntu (**`/bin/sh`** → **dash**).
- **Common causes**: bash-only `set` options (`pipefail`, `-o errexit`, `-u` bundles)
  on the **unconditional** startup path, or **CRLF** line endings in the file that
  ships from npm.
- **`sh` vs `bash`**: the Unix CLI path uses **`sh` + `installer.sh`** (**BUG-0004** /
  **DEC-0068**). Do not assume bash for the first lines of **`installer.sh`**.
- **Remediation**:
  - Upgrade to an **its-magic** build that includes **US-0084** (LF + POSIX guards).
  - Normalize to **LF** only (e.g. `dos2unix installer.sh`, or fix checkout —
    root **`.gitattributes`** uses `*.sh text eol=lf`).
  - Reinstall from npm after verifying maintainer gates:
    `python scripts/guard_installer_publish.py` (also **`npm run guard:installer`**
    / **`prepublishOnly`**).
- **Normative**: **`docs/engineering/architecture.md`** **`# US-0084`**.

### Automated checks (US-0084)

- `python tests/installer_shell_bug0004_test.py` — CR/LF rejection, forbidden
  `set` tokens, optional **`dash -n`** when **`dash`** is on **`PATH`**.
- `python scripts/guard_installer_publish.py` — same checks for publish/CI
  (**`prepublishOnly`**).
- `python scripts/remote_config_summary.py` — with **`REMOTE_EXECUTION=1`**,
  read-only summary of **`REMOTE_CONFIG`** (default **`.cursor/remote.json`**);
  stdout is **names-only** (no secret values). **`DEC-0070`**: when
  **`REMOTE_EXECUTION=0`**, the helper exits **0** and skips validation
  (stderr skip reason).

### Optional deterministic CI routing recipe (US-0086)

Use this only when CI needs explicit remote-target hints; keep it opt-in.

1. Define explicit path filters:
   - container surfaces: `Dockerfile*`, `docker-compose*.yml`, container scripts
   - ssh/runtime infra surfaces: deployment ssh scripts, host runtime scripts
2. Route using explicit matrix labels (`local`, `docker`, `ssh`) with no
   implicit fallback logic outside documented defaults.
3. Keep manual mode unchanged: if `AUTO_REMOTE_AUTOMATION_PROFILE=off`, run
   local path and do not apply remote routing.
4. Emit names-only evidence (`target_id`, `environment_label`,
   `automation_profile`, `routing_source`, `secret_surface=names_only`) into
   CI logs/artifacts.

## Runtime QA autopilot contract (US-0065 / DEC-0047)

Generated-project validation requires runtime proof, not static checks alone.

Mandatory runtime stage order:

`startup -> readiness/connectivity -> log scan -> bounded retry -> verdict`

Deterministic runtime failure reason codes:

- `RUNTIME_STARTUP_FAILED`
- `RUNTIME_ENDPOINT_UNREACHABLE`
- `RUNTIME_LOG_CRITICAL_DETECTED`
- `RUNTIME_RETRY_BUDGET_EXHAUSTED`
- `RUNTIME_STACK_PROFILE_UNRESOLVED`

Runtime evidence schema (record in QA findings):

- `runtime_startup_command`
- `runtime_stack_profile` (`node|python|go|java|dotnet`)
- `runtime_mode` (`local|remote`)
- `runtime_health_target`
- `runtime_health_result`
- `runtime_log_summary` (severity counts and key error signals)
- `runtime_retry_count`
- `runtime_retry_ledger` (`attempt`, `delay_ms`, `outcome`)
- `runtime_final_verdict`
- `runtime_reason_code`
- `runtime_evidence_refs`

Bounded retry policy:

- retry only transient startup/connectivity failures
- enforce configured max-attempt cap (`attempt <= max`)
- fail fast on non-transient critical runtime log signals

Stack/profile resolution:

- Minimum supported runtime profiles: Node, Python, Go, Java, .NET.
- Unknown or ambiguous profile must fail closed with
  `RUNTIME_STACK_PROFILE_UNRESOLVED`.

Webapp verification path (when applicable):

- include browser-surface load validation
- capture console error summary and failed network request summary
- add these signals to `runtime_log_summary` and evidence refs

Optional debug escalation (bounded):

- use for reproducible runtime failures only
- keep instrumentation bounded and reversible
- record applied debug steps and explicit cleanup confirmation

## Generated test scaffolding + auto-run contract (US-0066 / DEC-0048)

Generated app projects require deterministic baseline test scaffolding and
automatic QA test execution evidence.

Detection/profile contract:

- Resolve one deterministic stack profile from:
  `node|python|go|java|dotnet` (minimum supported).
- If profile cannot be resolved, fail closed with
  `TEST_SCAFFOLD_STACK_UNRESOLVED`.
- If detected stack is outside supported baseline set, fail closed with
  `TEST_SCAFFOLD_UNSUPPORTED_STACK`.

Generation contract (`/execute`):

- Generate only missing baseline assets for:
  - unit tests
  - integration tests
  - acceptance tests
- Use stable scaffold paths so reruns are idempotent (no duplicate file churn).
- Record generated paths and actions in execution evidence.
- If generation fails, fail closed with `TEST_SCAFFOLD_GENERATION_FAILED`.

Runbook command wiring:

- `TEST_COMMAND` baseline is stack-aware and deterministic.
- Non-destructive precedence is mandatory:
  - preserve user-authored non-empty `TEST_COMMAND`,
  - write baseline command only when `TEST_COMMAND` is missing/unset.

QA auto-run evidence contract (`/qa`):

- Execute generated baseline tests automatically.
- Record evidence fields:
  - `generated_test_stack_profile`
  - `generated_test_command`
  - `generated_test_result`
  - `generated_test_output_ref`
  - `generated_test_paths_ref`
  - `generated_test_reason_code`

Runtime boundary with US-0065:

- Generated static test PASS is required but never sufficient for QA PASS.
- Runtime-autopilot verdict remains mandatory; non-starting apps cannot PASS QA.

## Auto continuation resume contract

`/auto` continuation uses deterministic phase resolution (DEC-0017):

1. explicit `/auto start-from=<phase>`
2. `handoffs/resume_brief.md`
3. conservative `docs/engineering/state.md` fallback
4. fail-fast

Canonical `start-from` phase IDs:
`intake`, `discovery`, `research`, `architecture`, `sprint-plan`,
`plan-verify`, `execute`, `qa`, `verify-work`, `release`, `refresh-context`.

Conflict and stale-source policy:
- Explicit valid override wins.
- If no override and `resume_brief` conflicts with `state`, fail fast.
- If `resume_brief` exists but is stale/unparseable, fail fast.
- Use state fallback only when `resume_brief` is absent.

Fail-fast error format:
- `[AUTO_RESUME_ERROR] <code>: <summary>. Source=<source>. Fix: <action>.`

Required error codes:
- `INVALID_START_FROM`
- `RESUME_BRIEF_MISSING`
- `RESUME_BRIEF_STALE`
- `RESUME_BRIEF_UNPARSEABLE`
- `RESUME_STATE_CONFLICT`
- `STATE_PHASE_AMBIGUOUS`
- `STATE_PHASE_UNRECOVERABLE`

Breadcrumbs required for inspectability:
- `resolution_source`, `resolved_start_phase`, `stop_reason`, `stop_phase`,
  `timestamp`.
- Record in `docs/engineering/state.md`; update `handoffs/resume_brief.md` when
  auto stops before completion.

Stop-condition preservation:
- continuation does not bypass decision gates, missing-input blockers,
  pause requests, or loop max cycle limits.

## Per-phase subagent isolation evidence (US-0048 / DEC-0029)

Per-phase fresh-context isolation is enforced with auditable, fail-closed
evidence.

### Canonical evidence store and locations

- Canonical evidence store: `docs/engineering/state.md` (append-only checkpoints).
- Cross-references are allowed in phase artifacts and handoffs:
  - `handoffs/dev_to_qa.md`, `handoffs/qa_to_dev.md`
  - `handoffs/resume_brief.md` (pause/resume provenance)
  - `sprints/Sxxxx/summary.md`, `sprints/Sxxxx/qa-findings.md`, `sprints/Sxxxx/uat.*`,
    `sprints/Sxxxx/release-findings.md`

### Required schema (one entry per phase run)

Each phase run must append an isolation evidence entry containing:

- `phase_id`: canonical phase id (`intake|discovery|research|architecture|sprint-plan|plan-verify|execute|qa|verify-work|release|refresh-context|pause|resume`)
- `role`: subagent role executing the phase (`po|curator|tech-lead|dev|qa|release|security`)
- `fresh_context_marker`: a marker unique to the fresh subagent context for this phase run
- `timestamp`: ISO UTC timestamp
- `evidence_ref`: canonical path to the primary artifact written/validated for the phase run

### Gate behavior (fail closed)

- Missing evidence blocks progression with `PHASE_CONTEXT_ISOLATION_MISSING`.
- Invalid schema/fields blocks progression with `ISOLATION_EVIDENCE_INVALID`.
- Stale evidence (reused marker across runs or older than the resumed boundary)
  blocks progression with `ISOLATION_EVIDENCE_STALE`.
- Orchestrator executing phase work without spawning a fresh subagent context is
  a hard violation: `PHASE_CONTEXT_ISOLATION_VIOLATION`.

Remediation (all cases): re-run the affected phase in a fresh subagent context
and write new isolation evidence before proceeding.

### Reason codes and remediation (US-0048)

- `PHASE_CONTEXT_ISOLATION_MISSING`: no isolation evidence entry found for a
  required phase run. Fix: rerun the phase in a fresh subagent and append the
  required evidence fields.
- `ISOLATION_EVIDENCE_INVALID`: evidence entry present but missing required
  fields or contains invalid `phase_id`/`role`. Fix: rerun the phase and write a
  corrected entry.
- `ISOLATION_EVIDENCE_STALE`: evidence is reused across runs/cycles or predates
  the latest resume boundary. Fix: rerun the phase and write a new
  `fresh_context_marker`.
- `PHASE_CONTEXT_ISOLATION_VIOLATION`: phase work was performed without a fresh
  subagent context (for example orchestrator performed phase writes). Fix: stop,
  revert unsafe artifacts if needed, rerun the phase correctly, and ensure
  orchestration-only behavior.

## Strict runtime proof contract (US-0056 / DEC-0038)

Strict runtime proof augments artifact-level isolation evidence. `/auto`,
`/verify-work`, and `/release` must validate runtime attestation tuples at phase
boundaries before continuation/finalization.

Required runtime attestation tuple fields:

- `orchestrator_run_id`
- `runtime_proof_id` (unique per phase run)
- `phase_id`
- `role`
- `proof_issued_at` (ISO UTC / RFC3339)
- `proof_ttl_seconds`
- `proof_hash`

Deterministic fail-closed reason codes:

- `RUNTIME_PROOF_MISSING`
- `RUNTIME_PROOF_INVALID`
- `RUNTIME_PROOF_REUSED`
- `RUNTIME_PROOF_STALE`
- `RUNTIME_PROOF_AMBIGUOUS_LINK`

Boundary behavior:

- Missing/invalid/reused/stale/ambiguous runtime proof blocks progression.
- Release finalization must consume strict runtime proof in addition to existing
  isolation evidence checks.
- Pause/resume provenance must reference latest valid strict-proof boundary.

## Strict `/auto` phase→role enforcement (US-0069 / DEC-0051)

`/auto` must treat phase roles as a **fail-closed admission and checkpoint
contract** (see `decisions/DEC-0051.md` and `/auto` command text).

### Canonical matrix and scratchpad alternates

- Fixed phase→role defaults are documented in `/auto` (for example `execute` →
  `dev`, `release` → `release`).
- Alternate phases resolve **one** expected role via scratchpad:
  - `AUTO_ROLE_RESEARCH`: `po` \| `tech-lead` (empty → default `tech-lead`)
  - `AUTO_ROLE_PLAN_VERIFY`: `qa` \| `tech-lead` (empty → default `qa`)
  - `AUTO_ROLE_REFRESH_CONTEXT`: `curator` \| `po` (empty → default `curator`)
- Non-empty values outside the allowed set fail closed (no unrelated-role
  substitution).

### Preflight and checkpoints

- **Preflight (before spawn)**: resolve expected role; verify the required
  subagent capability exists. Missing capability → `PHASE_ROLE_CAPABILITY_MISSING`
  with `phase_id`, expected role, observed result, remediation. Do not spawn a
  substitute role.
- **Post-completion**: isolation evidence `role` and strict-proof `role` must
  both match the same preflight-resolved role; else `PHASE_ROLE_MISMATCH`.
- **`proof_hash`**: SHA-256 over sorted-key JSON of the strict-proof tuple fields
  (`orchestrator_run_id`, `runtime_proof_id`, `phase_id`, `role`,
  `proof_issued_at`, `proof_ttl_seconds`).

### Execute default deny and rare override

- Default: `execute` requires `dev`.
- Override allowed only when **both** hold:
  `AUTO_EXECUTE_ROLE_OVERRIDE=allowed_non_dev_execute` and
  `EXECUTE_OVERRIDE_GOVERNANCE_REF` references a parseable approved exception (for
  example `DEC-xxxx` or a documented state anchor).

### Continuation parity

- Every `/auto` run recomputes role policy and preflight; `start-from`, fresh
  `resume_brief`, and `state.md` fallback cannot bypass the gate with stale role
  intent alone.

## Configurable `/auto` phase plan (US-0070 / DEC-0052)

`/auto` schedules a **resolved ordered phase plan** from merged scratchpad
before any spawn. See `decisions/DEC-0052.md` and `/auto` command text.

### Selectors (exactly one active mode)

- `AUTO_PHASE_PLAN=full` (default when unset and no other selector is set)
- `AUTO_PHASE_EXCLUDE=<csv>` — remove listed canonical phase ids from `full`
- `AUTO_PHASE_INCLUDE=<csv>` — only listed ids, re-sorted into canonical lifecycle order
- `AUTO_PHASE_PROFILE=<name>` — expand a named profile (unknown → fail closed)
- `AUTO_PHASE_HIGH_RISK_ACK=<token>` — required when a documented high-risk profile demands acknowledgment

Conflicting selectors → `PHASE_POLICY_CONFLICT` (no plan materialization).

### Materialization and gates

- Expand → apply **non-skippable reinstatement** (`qa`, `verify-work`, `release`,
  plus evidence-chain closure per `/auto`) → intersect **`start-from` / resume
  anchor** with the plan → **empty intersection** →
  `START_FROM_PHASE_PLAN_EMPTY_INTERSECTION`.
- Record `resolved_phase_plan`, `skipped_phases` (+ reasons such as
  `policy_exclude`, `non_skippable_gate`), and **phase boundary status** entries
  in continuation breadcrumbs (`docs/engineering/state.md`).
- **Backlog-drain**, **bulk execute**, and **team scope** paths must **reload**
  scratchpad phase-selection inputs and **recompute** the plan at each bounded
  boundary (no silent revival of omitted phases).

### Failure codes (deterministic)

- `PHASE_POLICY_CONFLICT`
- `PHASE_PLAN_UNKNOWN_PHASE`
- `PHASE_PLAN_EMPTY_INCLUDE`
- `PHASE_PLAN_UNKNOWN_PROFILE`
- `PHASE_PLAN_INVALID_AUTO_PHASE_PLAN`
- `PHASE_PLAN_HIGH_RISK_ACK_REQUIRED`
- `START_FROM_PHASE_PLAN_EMPTY_INTERSECTION`

## Optional backlog-drain auto mode (US-0044)

`/auto` can optionally continue across multiple planned stories when explicitly
enabled in scratchpad.

Controls:
- `AUTO_BACKLOG_DRAIN=0|1` (default `0`)
- `AUTO_BACKLOG_MAX_STORIES=<n>` (default `1`)
- `AUTO_BACKLOG_ON_BLOCK=stop|skip` (default `stop`)
- `AUTO_STORY_SELECTION=priority_then_backlog_order` (default)

Semantics:
- With `AUTO_BACKLOG_DRAIN=0`, keep current single-segment continuation behavior.
- With `AUTO_BACKLOG_DRAIN=1`, select next eligible OPEN story
  deterministically and run full lifecycle story-by-story until bounded limit,
  no eligible stories, or a mandatory stop condition.
- Decision gates remain mandatory and pause progression until user decision.

## Targeted bug auto drain (US-0087)

Use **`/auto`** with an explicit **OPEN** bug binding when you want defect-scoped
continuation instead of story **`AUTO_BACKLOG_DRAIN`**.

**Canonical argv** (exact literals; no aliases in v1):

- **`bug-target=BUG-####`** — single **OPEN** bug from **`docs/product/backlog.md`**
  **`## Bug issues (canonical)`** (example: **`bug-target=BUG-0007`**).
- **`bug-target=all-open`** — walk all **OPEN** bugs in ascending **numeric**
  **`BUG-####`** order (optional per-run cap: **`AUTO_BUG_MAX_ITEMS`**).

**Scratchpad** (merged; default-off — see **`.cursor/scratchpad.md`** and
**`template/.cursor/scratchpad.local.example.md`**):

- **`AUTO_BUG_QUEUE`**, **`AUTO_BUG_TARGET`**, **`AUTO_BUG_MAX_ITEMS`**, **`AUTO_BUG_ON_BLOCK`**

**Mutex**: do **not** enable **`AUTO_BACKLOG_DRAIN=1`** and **`AUTO_BUG_QUEUE=1`**
together **without** an explicit **`bug-target=`** argv on that invocation — fail
closed **`AUTO_SCHEDULER_CONFLICT`**. Supply **`bug-target=`** to select the bug
scheduler for that run (normative detail: **`docs/engineering/auto-orchestration-reference.md`**
**Optional bug-queue mode (US-0087)** and **`docs/engineering/architecture.md`**
**`# US-0087`**).

**Fail-closed codes**: **`AUTO_BUG_QUEUE_EMPTY`**, **`AUTO_BUG_TARGET_UNKNOWN`**,
**`AUTO_BUG_TARGET_NOT_OPEN`**, **`AUTO_SCHEDULER_CONFLICT`** — plus spawn-only
orchestrator rules (**`BUG-0006`**, **`US-0069`**, **`AUTO_ORCHESTRATOR_PHASE_EXECUTION`**;
see **`.cursor/commands/auto.md`**).

## Continuous `/auto` + backlog drain (US-0088)

**Goal:** a single `/auto` run (or documented equivalent outer driver) advances
through all intersected lifecycle phases until a deterministic stop, and
`AUTO_BACKLOG_DRAIN=1` can continue across multiple OPEN stories without routine
operator chatter.

### Quick start

```
/auto                                       # full lifecycle, single story
/auto start-from=execute                    # resume from execute phase
```

With backlog drain enabled (`.cursor/scratchpad.md`):

```
AUTO_BACKLOG_DRAIN=1
AUTO_BACKLOG_MAX_STORIES=5
AUTO_BACKLOG_ON_BLOCK=stop
```

### Normative reference

Multi-phase iteration lives in
**`docs/engineering/auto-orchestration-reference.md`** **`## Steps`** item 5
(cross-anchor: **"reference Step 5"**). The compact steps in
**`.cursor/commands/auto.md`** point to that block unambiguously.

### Caps and safety guards

| Control | Default | Purpose |
|---------|---------|---------|
| `AUTO_BACKLOG_MAX_STORIES` | `1` | Max stories per drain run |
| `AUTO_LOOP_MAX_CYCLES` | `5` | Max execute-QA cycles per story |
| `AUTO_PAUSE_REQUEST` | `0` | Set to `1` to request graceful stop at next safe boundary |
| `AUTO_PAUSE_POLICY` | `after_phase` | Stop boundary granularity |

### Decision gates

Decision gates are **never** suppressed — even when `AUTO_QUIET=1`. When a gate
fires, the run stops and waits for operator resolution before continuing.

### `AUTO_QUIET` (default off)

Set `AUTO_QUIET=1` in `.cursor/scratchpad.md` to suppress **routine** per-phase
success chatter. Non-suppressible notifications:

- `decision_gate`
- Errors / `missing_input`
- `pause_request`
- `loop_max`
- `blocked`
- Segment handoff / drain advance

`AUTO_QUIET` is **orthogonal** to `TOKEN_PROFILE` (**DEC-0035** / **US-0080**):
`TOKEN_PROFILE` controls context breadth and token cost, not notification policy.

### Caveman mode (US-0089)

Optional response-side terse / imperative assistant voice. **Default off.**
When `CAVEMAN_MODE=0` (or absent), this mode adds **zero** behavioral change
and the assistant responds exactly as it did pre-US-0089. Full contract:
**DEC-0072** + `docs/engineering/architecture.md` `# US-0089` +
`.cursor/rules/caveman.mdc`.

Non-substitution with `TOKEN_PROFILE`:

`TOKEN_PROFILE` controls context breadth. `CAVEMAN_MODE` controls reply voice. Neither substitutes for the other; setting one does not change the other. Combine freely.

#### Scratchpad keys

| Key | Values | Default | Notes |
|-----|--------|---------|-------|
| `CAVEMAN_MODE` | `0` \| `1` | `0` | `0` = pre-US-0089 behavior. `1` = voice rule active. Absence = `0`. |
| `CAVEMAN_LEVEL` | `lite` \| `full` \| `ultra` or empty | empty | With `MODE=0`: inert. With `MODE=1` and empty: treat as `full`. Unknown value → `CAVEMAN_LEVEL_UNKNOWN` (fail closed; fall back to pre-US-0089 voice while continuing the turn). |
| `CAVEMAN_COMPRESS_INPUT` | `0` \| `1` | `0` | **Reserved for US-0090.** Documented no-op in US-0089. |
| `CAVEMAN_FILE_SCOPE` | string | empty | **Reserved for US-0090.** Documented no-op in US-0089. |

#### Canonical operator toggle phrases

| Phrase | Effect |
|--------|--------|
| `caveman on` | Enable Caveman voice for the session (overlay). Effective from the next assistant turn. |
| `caveman off` | Disable Caveman voice for the session (overlay). Effective from the next assistant turn. |
| `stop caveman` | Alias for `caveman off`. |
| `normal mode` | Alias for `caveman off`. |
| `caveman: lite|full|ultra` | Set level for the session (implies `caveman on`). Effective from the next assistant turn. Accepts the three literal tokens `caveman: lite`, `caveman: full`, `caveman: ultra`. |

#### Determinism semantics

- Scratchpad `CAVEMAN_MODE` / `CAVEMAN_LEVEL` are **authoritative across
  subagent spawns**; session toggle phrases are overlays for the current
  conversation only and do NOT persist across a fresh subagent context.
- Session toggle phrases apply **as an overlay for the next assistant
  turn**; they never rewrite the current turn's machine-verifiable
  artifacts (gate messages, reason codes, strict-proof tuples, isolation
  evidence fields).
- Within a session, the **last explicit toggle wins**. Ambiguous phrases
  are **not** recognized — only the literal matches in the table above.

#### Literal-region invariant (rule-enforced)

Under `CAVEMAN_MODE=1`, the 9 literal regions enumerated in
`.cursor/rules/caveman.mdc` (fenced code, paths, AC checklists, reason
codes, IDs, contract markers, strict-proof tuple fields, isolation
evidence fields, git refs) render byte-literal. The non-suppressible gate
vocabulary inherited from **US-0088** (`decision_gate`, `error`, `pause`,
`loop_max`, `blocked`, `missing input`, `[BUG_VALIDATION_OK]`,
`[INTAKE_EVIDENCE_VALIDATION_OK]`, `[SCRATCHPAD_PAIR_OK]`) also renders
byte-literal even at `CAVEMAN_LEVEL=ultra`.

### Caveman input compression (US-0090)

Optional **input-side** file compression. **Default off.** Operator-initiated,
script-invoked only. Never fires autonomously. Full contract:
**DEC-0073** + `docs/engineering/architecture.md` `# US-0090` +
`scripts/caveman_compress_input.py`.

Non-substitution with `TOKEN_PROFILE` and `CAVEMAN_MODE`:

`TOKEN_PROFILE` controls context breadth. `CAVEMAN_MODE` controls reply voice. `CAVEMAN_COMPRESS_INPUT` controls input-side file compression. All three axes are orthogonal: setting one does not change the others, and none substitutes for another.

#### Activation gate (DEC-0073 §2)

All three conditions must hold before any mutation occurs:

1. `CAVEMAN_COMPRESS_INPUT=1` in `.cursor/scratchpad.md`.
2. `CAVEMAN_FILE_SCOPE` non-empty.
3. CLI invoked with `--write`.

Any failing condition short-circuits with a reason code from §7 and exit `2`.
Default / unset / partial state = no-op.

#### Sidecar originals (DEC-0073 §3)

Before mutating any file, the script writes the pre-mutation bytes to
`docs/.caveman-originals/<relative/path>/<filename>`. Atomic order: sidecar
first (temp + replace), then target (temp + replace). The tree is anchored
by `docs/.caveman-originals/.gitkeep` and excluded from VCS by the repo-root
`.gitignore` anchor for US-0090.

#### Deny-list policy (DEC-0073 §4)

Layered, read in this order (**deny always wins**):

1. Hard-coded baseline in `scripts/caveman_compress_input.py` (`DENY_BASELINE`).
2. Merged secret-like patterns from repo-root `.gitignore` (`.env*`, `*secret*`,
   `*credential*`, `*token*`, `*private*`).
3. Optional `.cursorignore` overlay when
   `CAVEMAN_COMPRESS_INGEST_CURSORIGNORE=1` in scratchpad.

Deny-list baseline is versioned via `deny_list_version` (SHA-256 of sorted
canonical JSON) and reported by `--report`.

#### Allow-list grammar (DEC-0073 §5)

Three forms in `CAVEMAN_FILE_SCOPE`:

| Form | Example | Notes |
|------|---------|-------|
| Named profile | `docs-prose-only` | Frozen v1 table; new profiles require subsequent DEC. |
| Raw CSV globs | `docs/user-guides/**/*.md,handoffs/archive/*.md` | Forward slashes only. |
| Hybrid | `profile:docs-prose-only;globs:handoffs/archive/*.md` | One profile per scope; unknown tokens fail closed. |

#### Safe-mode minifier (DEC-0073 §6)

Four-step, strictly idempotent pipeline:

1. Collapse two-or-more consecutive blank lines into a single blank line
   (outside fenced code).
2. Trim trailing whitespace on non-fence lines.
3. Normalize `CRLF` / `CR` → `LF`.
4. Preserve the source file's EOF-newline status.

Aggressive mode is **deferred**; v1 ships safe-mode only. All safe-mode
transformations keep the 9 DEC-0072 §4 literal regions byte-identical; any
drift is fail-closed with `CAVEMAN_COMPRESS_LITERAL_REGION_DAMAGED`.

#### Reason-code vocabulary (DEC-0073 §7)

Nine codes in three families. No post-write codes.

| Family | Code |
|--------|------|
| Gating | `CAVEMAN_COMPRESS_MODE_DISABLED` |
| Gating | `CAVEMAN_COMPRESS_FLAG_CONFLICT` |
| Scope | `CAVEMAN_COMPRESS_SCOPE_EMPTY` |
| Scope | `CAVEMAN_COMPRESS_SCOPE_UNKNOWN_PROFILE` |
| Scope | `CAVEMAN_COMPRESS_SCOPE_VIOLATION` |
| Integrity | `CAVEMAN_COMPRESS_DENY_HIT` |
| Integrity | `CAVEMAN_COMPRESS_NOT_IDEMPOTENT` |
| Integrity | `CAVEMAN_COMPRESS_LITERAL_REGION_DAMAGED` |
| Integrity | `CAVEMAN_COMPRESS_ORIGINAL_MISSING` |

Additions require a subsequent DEC amending §7.

#### CLI contract (DEC-0073 §8)

| Flag | Semantics |
|------|-----------|
| `--dry-run` | (default) inventory + diff summary to stdout; no mutation. |
| `--write` | Perform sidecar + target mutation on eligible files (sidecar first). |
| `--verify-originals` | Walk sidecar tree; verify bidirectional presence; fail closed with `CAVEMAN_COMPRESS_ORIGINAL_MISSING` on orphan. |
| `--report` | Emit canonical JSON report on stdout (incompatible with `--write`). |

#### Scratchpad keys (US-0090 additions)

| Key | Values | Default | Notes |
|-----|--------|---------|-------|
| `CAVEMAN_COMPRESS_INPUT` | `0` \| `1` | `0` | Activation gate bit (DEC-0073 §2). |
| `CAVEMAN_FILE_SCOPE` | string | empty | Profile name, CSV globs, or hybrid (§5). |
| `CAVEMAN_COMPRESS_INGEST_CURSORIGNORE` | `0` \| `1` | `0` | Optional overlay (§4). |

#### Template parity (DEC-0073 §10)

The following pairs are byte-identical between active and template copies and
installer-owned (BUG-0003 / DEC-0066): `scripts/caveman_compress_input.py`,
`docs/engineering/context/installer-owned-paths.manifest`,
`docs/engineering/runbook.md`, `docs/engineering/auto-orchestration-reference.md`.
Verify with `python scripts/check_intake_template_parity.py --scope=caveman-compress`.
Negative parity (must NOT track): `.cursor/rules/caveman.mdc` (US-0089
rule-set; US-0090 adds no new Cursor rule).

### Outer-driver equivalence (AC-1, Option B)

When a single Cursor `/auto` invocation cannot schedule multiple subagent turns,
operators may use an outer driver (script or manual re-invocation with
`start-from` / refreshed `resume_brief`). This is deterministically equivalent
when: same phase order, same isolation + strict-proof per phase, same stop
reasons, same `resume_brief` + `state.md` refresh at every boundary.

### Drain advance behavior

When `AUTO_BACKLOG_DRAIN=1` and a story reaches its terminal boundary:

1. Orchestrator reloads merged scratchpad phase-selection inputs.
2. Orchestrator recomputes the materialized phase plan for the next story.
3. Selects the next eligible OPEN story per `AUTO_STORY_SELECTION`.
4. Runs the full resolved lifecycle for that story until stop or cap.

Notify operator on segment handoff (non-routine, non-suppressible).

### Stop reasons

`completed`, `decision_gate`, `missing_input`, `pause_request`, `loop_max`,
`error`, `blocked`. See the **Deterministic stop matrix** in
**`docs/engineering/auto-orchestration-reference.md`** §
**Continuous multi-phase execution (US-0088)**.

### Troubleshooting

| Symptom | Likely cause | Fix |
|---------|-------------|-----|
| Run stops after one phase | Older `/auto` text without continuous semantics | Update to latest; verify **reference Step 5** anchor exists |
| `RESUME_BRIEF_STALE` mid-run | Brief not refreshed at phase boundary | Ensure paired `resume_brief` + `state.md` refresh per DEC-0069 |
| `AUTO_SCHEDULER_CONFLICT` | Both `AUTO_BACKLOG_DRAIN=1` and `AUTO_BUG_QUEUE=1` without `bug-target=` argv | Supply explicit `bug-target=` or disable one scheduler |
| `BACKLOG_MAX_STORIES_REACHED` | Drain cap hit | Increase `AUTO_BACKLOG_MAX_STORIES` or run another `/auto` |

## Explicit bulk sprint planning mode (US-0046)

`/sprint-plan` stays single-scope by default. Bulk planning is opt-in via
explicit argument:

- `/sprint-plan --bulk`

Deterministic controls from `.cursor/scratchpad.md`:
- `SPRINT_BULK_MAX_STORIES` (candidate OPEN stories per run)
- `SPRINT_BULK_MAX_SPRINTS` (max generated sprints per run)
- `SPRINT_BULK_SELECTION=priority_then_backlog_order`

Deterministic behavior:
- Select eligible OPEN stories by configured selection order.
- Generate one or more bounded sprint plans while preserving per-sprint sizing
  guardrails (`SPRINT_MAX_TASKS`, `SPRINT_AUTO_SPLIT`).
- Stop with explicit reason codes when bounded or blocked:
  - `SPRINT_BULK_MAX_STORIES_REACHED`
  - `SPRINT_BULK_MAX_SPRINTS_REACHED`
  - `SPRINT_BULK_NO_ELIGIBLE_STORIES`
  - `SPRINT_BULK_MISSING_ACCEPTANCE`

## Explicit bulk execute mode (US-0047)

`/auto` remains non-bulk by default. Bulk execution is explicit and can be
enabled per run (`/auto --execute-bulk`) or by scratchpad switch.

Deterministic controls:
- `AUTO_EXECUTE_BULK=0|1` (default `0`)
- `AUTO_EXECUTE_MAX_ITEMS=<n>` (default `1`)
- `AUTO_EXECUTE_ON_BLOCK=stop|skip` (default `stop`)
- `AUTO_EXECUTE_SELECTION=planned_then_priority` (default)
- `AUTO_TEAM_SCOPE_ENFORCE=0|1` (default `1`)

Execution semantics:
- Select eligible planned items deterministically.
- Preserve strict isolation:
  - fresh subagent per phase
  - fresh subagent per execute<->QA loop cycle
- Enforce bounded stop behavior:
  - `EXEC_BULK_MAX_ITEMS_REACHED`
  - `EXEC_BULK_NO_ELIGIBLE_ITEMS`
  - `EXEC_BULK_ITEM_BLOCKED_STOP`
  - `EXEC_BULK_ITEM_BLOCKED_SKIPPED`

Team mode guardrails (`TEAM_MODE=1`):
- Capture team context snapshot in breadcrumbs:
  - `TEAM_MODE`, `TEAM_MEMBER`, `ACTIVE_TASK_IDS`
- With enforcement enabled, out-of-scope tasks are never mutated and must emit:
  - `EXEC_TEAM_SCOPE_BLOCKED` (stop policy)
  - `EXEC_TEAM_SCOPE_SKIPPED` (skip policy)

## Sync policy and guarded auto-push contract (US-0038 / DEC-0018)

Sync policy controls (from `.cursor/scratchpad.md`):
- `SYNC_POLICY_MODE`: `disabled|manual|by_phase|by_milestone|custom_phase_list`
- `SYNC_CUSTOM_PHASES`: comma-separated canonical phase IDs for custom mode
- `ALLOW_AUTO_PUSH`: `0|1`
- `AUTO_PUSH_BRANCH_ALLOWLIST`: comma-separated branches/patterns

Default-safe behavior:
- Default mode is `manual` (non-auto).
- `disabled` and `manual` are near-zero-overhead modes (no auto-push attempts).
- Unset/invalid mode fails closed to `manual`.

Phase-boundary-only evaluation:
- Evaluate sync eligibility only at completed phase boundaries.
- Never evaluate during partial or in-progress work units.

Guarded auto-push eligibility (all required):
1. Boundary trigger is eligible for current mode.
2. `ALLOW_AUTO_PUSH=1`.
3. QA-first restriction passes (feature work cannot auto-push before QA pass).
4. No unresolved blocking QA findings / unresolved critical issues.
5. Branch safety passes:
   - protected/default branches denied by default,
   - allow only explicitly allowlisted branches.
6. Mandatory check chain passes.

Mandatory pre-push check chain:
1. `TEST_COMMAND` (mandatory baseline)
2. `LINT_COMMAND` (only if configured)
3. `TYPECHECK_COMMAND` (only if configured)

Rules:
- Missing `TEST_COMMAND` blocks push (`TEST_COMMAND_MISSING`).
- Failing `TEST_COMMAND` blocks push (`TEST_FAILED`).
- Timed-out `TEST_COMMAND` blocks push (`TEST_TIMEOUT`).
- Optional check failures block push when configured (`OPTIONAL_CHECK_FAILED`).
- Optional checks that are not configured must be reported as `skipped`.

Deterministic reason-code baseline:
- `SYNC_DISABLED`
- `MANUAL_MODE_NO_AUTO`
- `SYNC_TRIGGER_NOT_ELIGIBLE`
- `AUTO_PUSH_NOT_ENABLED`
- `PRE_QA_AUTOPUSH_FORBIDDEN`
- `BLOCKING_QA_FINDINGS`
- `BRANCH_NOT_ALLOWLISTED`
- `TEST_COMMAND_MISSING`
- `TEST_FAILED`
- `TEST_TIMEOUT`
- `OPTIONAL_CHECK_FAILED`
- `SYNC_PUSHED`

## Executable validate-and-push wiring (DEC-0058)

Scratchpad **`SYNC_*` / `ALLOW_AUTO_PUSH` / `AUTO_PUSH_BRANCH_ALLOWLIST`** are read from the
**merged** scratchpad only (installer merge: local → materialized baseline → example; same
precedence as installer post-install validation). **`scripts/validate-and-push.ps1`** and
**`scripts/validate-and-push.sh`** call **`python scripts/sync_push_gates.py`** for policy;
**`docs/engineering/runbook.md`** remains the sole source for **`TEST_COMMAND`** and optional
lint/typecheck commands.

**Operator rule:** changing scratchpad alone does **not** run **`git push`**. Run
**`validate-and-push`** (or CI) after an eligible boundary. For **`by_phase`**, **`by_milestone`**,
and **`custom_phase_list`**, scheduling is **operator or CI** responsibility.

**`SYNC_PHASE_BOUNDARY`:** optional environment variable (canonical phase id, case-insensitive).
When **`SYNC_POLICY_MODE=custom_phase_list`**, the variable must be set and must appear in
**`SYNC_CUSTOM_PHASES`** (comma-separated) or the script exits **`SYNC_TRIGGER_NOT_ELIGIBLE`**.

**Dry-run:** **`powershell .../validate-and-push.ps1 -DryRun`** or
**`bash scripts/validate-and-push.sh --dry-run ...`** — runs merge/policy and the runbook check
chain, then prints **`SYNC_PUSHED`** without **`git push`**.

**Branch allowlist matching (`AUTO_PUSH_BRANCH_ALLOWLIST`):** comma-separated entries; each entry
is either an exact branch name or a **`fnmatch`** pattern (for example `release/*`). An empty
allowlist denies every branch (**`BRANCH_NOT_ALLOWLISTED`**).

**QA scan (bounded):** files under **`sprints/S####/qa-findings.md`** (four digits). Blocking
rules match **`DEC-0058`** §6. **`PRE_QA_AUTOPUSH_FORBIDDEN`** applies on branches other than
**`main`** / **`master`** when **no** such **`qa-findings.md`** file exists yet (feature-line
signal; see architecture **US-0076**).

**Python:** merged policy evaluation requires **Python 3** on **`PATH`** (**`PYTHON_NOT_ON_PATH`**
if missing).

Required sync evidence fields:
- `phase_boundary`
- `policy_mode`
- `trigger_source` (`manual|auto`)
- `branch`
- `checks` (`test|lint|typecheck`: `pass|fail|skipped`)
- `qa_status_snapshot`
- `push_decision` (`pushed|blocked|not_eligible`)
- `reason_code`
- `evidence_refs`

## Release gate chain (US-0039 / DEC-0019)

Deterministic mandatory gate order; no step may be skipped or reordered:

1. **Check-in test gate** — Latest `TEST_COMMAND` evidence must be present and passing.
2. **QA completion gate** — No unresolved blocking findings in sprint QA context.
3. **UAT completion gate** — UAT artifacts populated and verified; no placeholder or unresolved-fail state.
4. **Isolation compliance gate** — Per-phase isolation evidence present and valid (US-0048 / DEC-0029).
5. **Release finalization** — Notes, queue, backlog/runbook/state updates only after gates 1–4 pass.

Default: no bypass. Override only via explicit decision gate with rationale and evidence (DEC-0019).

**Optional-command compatibility (US-0039 / AC-10)**: Blank optional runbook keys (`LINT_COMMAND`, `TYPECHECK_COMMAND`) must not cause release to fail. Mandatory gates are check-in test + QA + UAT + isolation only; optional checks run only when configured and are reported as `skipped` when not configured. Release does not require lint/typecheck evidence when those keys are blank.

**Per-gate audit verdict schema (US-0039)** — For TL/QA auditability, record per gate:

- `gate` (check-in_test | qa | uat | isolation | finalization)
- `verdict` (pass | fail | override)
- `reason_code` (e.g. RELEASE_TEST_FAILED, RELEASE_QA_BLOCKERS_OPEN, RELEASE_UAT_INCOMPLETE, RELEASE_GATE_OVERRIDE_APPROVED)
- `remediation` (short remediation steps when fail/override)
- `evidence_refs` (paths to tests/report.md, qa-findings.md, uat.json, release-findings.md, DEC-xxxx as applicable)

Record in `sprints/Sxxxx/release-findings.md` and/or `handoffs/release_queue.md` `gate_snapshot`; state checkpoint in `docs/engineering/state.md` may reference the same.

## Release queue and sprint notes contract (US-0040 / DEC-0020)

Canonical release artifacts:
- `handoffs/releases/Sxxxx-release-notes.md` (canonical per-sprint notes)
- `handoffs/release_queue.md` (canonical queue tracker)
- `handoffs/release_notes.md` (legacy-compatible latest pointer/summary)

Queue row required fields:
- `sprint_id`
- `story_refs`
- `status` (`planned|ready|unreleased|released|blocked`)
- `last_updated`
- `release_notes_ref`
- `gate_snapshot`
- `release_version` (optional before finalization)

Deterministic transition semantics:
- target sprint only may change during one `/release` run
- entering release flow sets target row to `unreleased`
- successful finalization transitions same row to `released`
- no non-target sprint row mutation

Fail-safe reason codes:
- `RELEASE_SPRINT_UNRESOLVED`
- `LEGACY_NOTES_SPRINT_UNRESOLVED`
- `QUEUE_ENTRY_MISSING`
- `NOTES_REF_MISSING`
- `STATUS_TRANSITION_INVALID`

Mismatch and unresolved-sprint policy:
- fail closed for finalization when sprint identity or queue/notes metadata is
  inconsistent
- preserve existing notes artifacts by default (non-destructive)
- do not auto-reconcile by deleting/rebuilding unrelated sprint history
- include remediation steps in queue/state and rerun `/release` after correction

## Post-QA release issue workflow (US-0042)

When `/release` finds a blocker after QA has passed, document it in a dedicated
release findings artifact (separate from QA findings):

- Canonical artifact: `sprints/Sxxxx/release-findings.md`
- Canonical handoff back to implementation: `handoffs/release_to_dev.md`

Required release-findings content:
- gate status (`PASS|BLOCKED`)
- blocking and non-blocking findings
- deterministic reason code(s)
- evidence refs
- remediation steps and rerun criteria

Boundary rule:
- QA-phase defects remain in `sprints/Sxxxx/qa-findings.md`.
- Post-QA release-gate defects must be recorded in
  `sprints/Sxxxx/release-findings.md`.

## Backlog reconciliation invariant (US-0043)

At release finalization boundary, target sprint stories must be synchronized in
`docs/product/backlog.md` using canonical release evidence precedence.

Contract:
- Scope is target sprint stories only (no global backlog mutation).
- If release evidence is PASS, set story status to `DONE` and reconcile
  acceptance checkboxes to checked state.
- If sprint is `released` but backlog story state remains contradictory
  (`OPEN`/unchecked), fail safe with reason code `BACKLOG_STATUS_DRIFT`.
- Record remediation guidance and evidence refs in release artifacts before rerun.

## Canonical status ownership and normalization guard (US-0045)

Canonical owner:
- `docs/product/backlog.md` is the authority for story status (`OPEN|DONE`).
- `docs/product/acceptance.md` and `docs/engineering/state.md` are derived views.

Deterministic reconciliation rules:
1. Read canonical story status from backlog.
2. Validate target sprint release evidence for status transitions.
3. Reconcile derived acceptance/state views from canonical backlog status.
4. Keep mutation scope target-scoped only; never broad-rewrite unrelated stories.

One-time normalization procedure:
- Run an initial normalization pass for historically drifted stories.
- Write all changed rows to `docs/engineering/status-normalization-report.md`
  including prior values, resolved values, evidence references, and timestamp.
- On future runs, append only delta entries; do not rewrite historical report rows.

Fail-safe reason codes:
- `BACKLOG_STATUS_DRIFT`: release evidence contradicts backlog/AC state.
- `CANONICAL_STATUS_CONFLICT`: canonical backlog state conflicts with derived
  status resolution at reconciliation boundary.

## Lifecycle QA matrix (US-0041)

Use this matrix to validate end-to-end installer/CLI lifecycle behavior:

| Scenario | Primary command path | Coverage location | Required evidence |
|---|---|---|---|
| Fresh install (`missing`) | `its-magic --mode missing --create` and direct installer | `tests/run-tests.ps1`, `tests/run-tests.sh` | Required files exist + `its_magic/.its-magic-version` exists |
| Overwrite + backup | `its-magic --mode overwrite --backup` and direct installer | `tests/run-tests.ps1`, `tests/run-tests.sh` | Backup snapshot contains overwritten framework file |
| Upgrade lifecycle | `its-magic --mode upgrade` and direct installer | `tests/run-tests.ps1`, `tests/run-tests.sh`, npm local tests | Framework file restored, scratchpad example refreshed, user local scratchpad preserved |
| Clean-repo safety | `its-magic --clean-repo --yes` and direct installer clean path | `tests/run-tests.ps1`, `tests/run-tests.sh`, CI lifecycle subset | Framework artifacts removed, non-framework marker preserved |
| Negative path | invalid mode/args | `tests/run-tests.ps1`, `tests/run-tests.sh` | Deterministic non-zero fail-fast behavior |
| Platform parity subset | npm/brew/choco CI jobs | `.github/workflows/ci.yml` | Lifecycle subset passes on all three runners |

## Scratchpad example upgrade contract (US-0057 / DEC-0039 / DEC-0057)

`its-magic --mode upgrade` treats `.cursor/scratchpad.local.example.md` as
framework-owned and `.cursor/scratchpad.local.md` as user-owned.

Expected deterministic outcome:
- Framework-owned example is refreshed to latest release contract **before** baseline
  materialization runs in `installer.py --scratchpad-postinstall` (**DEC-0057** ordering).
- User local scratchpad remains preserved without overwrite.
- Installer output reports manifest copy status for the example file where applicable
  (`added|updated|unchanged`) **and** `[SCRATCHPAD_LAYER]` diagnostics from post-install
  (`example_refresh`, `baseline_materialize` / `baseline_skip`, `user_local` preserved).
- CI regression: `python scripts/check-scratchpad-pair-parity.py --repo <root>` exit `0`
  when active and `template/` baseline/example pairs share the same automation `KEY=`
  set and catalog `#` headers from `# Core behavior` (**US-0075** **AC-11**).

## Scratchpad delivery Model B (US-0073 / DEC-0055)

- Install manifest ships `.cursor/scratchpad.local.example.md` (framework catalog)
  but **does not** list `.cursor/scratchpad.md` as a copied file. The installer
  **materializes** `.cursor/scratchpad.md` from the packaged template when absent
  (`missing`, `interactive`, `upgrade`) or refreshes it on `overwrite`.
- Merge precedence for automation readers: **local > materialized baseline > example**
  (same invariant as `DEC-0055`).
- Post-install validation fails closed with `[SCRATCHPAD_MERGE_ERROR]` /
  `[SCRATCHPAD_MATERIALIZE_ERROR]` when layers are missing or required keys are
  empty after merge (`US-0073` `AC-4`).
- `installer.ps1` / `installer.sh` delegate materialize+validate to
  `python installer.py --scratchpad-postinstall` (Python 3 required on PATH).
- Recovery: `python installer.py --scratchpad-postinstall --target <repo> --mode missing`
  (or re-run a full install).

## Deterministic artifact ordering and write discipline (US-0058 / DEC-0040)

Canonical policy source:
- `docs/engineering/artifact-ordering-policy.md`

Required write discipline:
- `docs/engineering/state.md`: append-bottom checkpoint writes only.
- `docs/product/backlog.md`: sorted-canonical story ordering by numeric `US-xxxx`.
- `docs/product/acceptance.md`: sorted-canonical row ordering aligned to backlog.
- Handoff surfaces use explicit policy (`prepend-top` or `append-bottom`) per
  matrix and command contract.

Fail-safe contract:
- Missing/ambiguous placement anchors fail closed with
  `ARTIFACT_ORDERING_ANCHOR_AMBIGUOUS`.
- Non-monotonic `state.md` checkpoint timestamps fail closed with
  `STATE_TIMESTAMP_NON_MONOTONIC`.
- No partial mutation on fail-safe path.
- Re-run without semantic changes must be ordering-idempotent.

## Cross-phase artifact ownership guard (US-0061 / DEC-0043)

Canonical policy source:
- `docs/engineering/artifact-ownership-policy.md`

Required ownership discipline:
- Each phase may mutate only its declared owned scopes for target context.
- Cross-phase non-owned section rewrite/deletion is forbidden by default.
- `docs/engineering/architecture.md` is history-preserving: append new story
  sections or mutate target section only; unrelated story-section deletion is
  prohibited.

Fail-safe contract:
- Ownership violations fail closed with `PHASE_OWNERSHIP_VIOLATION`.
- Missing evidence on override-authorized mutation path fails closed with
  `PHASE_OVERRIDE_EVIDENCE_MISSING`.
- Architecture history deletion detection fails with
  `ARCH_HISTORY_DELETION_DETECTED`.
- No partial mutation on fail-safe path.

Execution guidance:
- Local baseline: run `sh tests/run-tests.sh` (or `powershell -ExecutionPolicy Bypass -File tests/run-tests.ps1`).
- Packaging smoke: run npm local tests in `packaging/npm/`.
- CI evidence: inspect `npm-test`, `brew-test`, and `choco-test` job logs.

## Intake runtime capability and single-writer safety (US-0059 / DEC-0041)

`/intake` enforces deterministic runtime preflight and drift safety before
artifact mutation.

Capability preflight:
- Required role capability: `po` subagent.
- Default policy: fail fast when unavailable with
  `SUBAGENT_CAPABILITY_UNAVAILABLE`.
- Fallback policy is explicit only:
  - `INTAKE_SUBAGENT_FALLBACK=deny` (default): no silent fallback.
  - `INTAKE_SUBAGENT_FALLBACK=allow`: explicit operator opt-in for fallback path.

Single-writer drift safety:
- Intake run binds a deterministic writer/run identity (`writer_id`,
  `intake_run_id`) to target artifacts.
- Self-write updates for the active writer/run are valid and must not trigger
  concurrent drift blockers.
- External concurrent conflicting writes fail safe with
  `INTAKE_CONCURRENT_WRITER_DETECTED`.
- Fail-safe path performs no partial overwrite.

## Post-release operator commands (S0070 / BUG-0008 — released `2026-04-05`)

**S0070** **`released`**; **`BUG-0008`** **DONE** in canonical backlog. In-repo version **`its-magic@0.1.2-41`**. **`/release`** skipped registry publish while **`RELEASE_PUBLISH_MODE=disabled`** — operators still run the steps below when pushing to npm or validating on Debian.

- **Tests (canonical):** `powershell -ExecutionPolicy Bypass -File "tests/run-tests.ps1"` — refresh **`tests/report.md`**; release gate used **793**/0 @ **2026-04-05T20:21:40Z** with **US-0071** harness rows **PASS**.
- **Prepublish:** `npm run prepublishOnly` (runs **`guard:installer`**).
- **Publish:** `npm publish` — set **`RELEASE_PUBLISH_MODE`** to **`confirm`** or **`auto`** when ready; no inline registry secrets in docs.
- **Debian global E2E (optional follow-up):** **`DEFERRED_DEBIAN_E2E_NO_RUNTIME`** was waived for the release cycle — when a Debian/SSH target exists (**US-0086**), run `npm install -g its-magic@0.1.2-41` (or equivalent), `cat -A` on installed `template/docs/engineering/context/installer-owned-paths.manifest` (no `^M$`), then `its-magic --target <repo> --mode missing` without `[INSTALL_MANIFEST_ERROR]`.

## Operator `.env` setup (US-0085 / DEC-0071)

### Quick start

1. Copy the committed template: `cp .env.example .env`
2. Fill in values for each variable relevant to your environment.
3. Source before remote, SSH, or release operations:
   - **Bash/Zsh**: `source .env` or `set -a; source .env; set +a`
   - **PowerShell**: `Get-Content .env | ForEach-Object { if ($_ -match '^([^#]\S+?)=(.*)$') { [Environment]::SetEnvironmentVariable($Matches[1], $Matches[2], 'Process') } }`
4. Run `python scripts/print_remote_env_hint.py` to verify parity between
   `.env.example` and the `*Env` fields in JSON configs.

### Forbidden

- **Committing `.env`**: `.env` is gitignored; never add it to version control.
- **Agents reading `.env`**: AI agents must not open, attach, read, search
  inside, or index `.env` or `.env.*` files (enforced via `.cursorignore` and
  Cursor rules). Use env var **names** in prose only.

### Allowed

- Running `ssh`, `docker`, `python scripts/remote_config_summary.py` after
  sourcing `.env` — the process inherits normal environment variables.
- Referencing env var **names** (not values) in documentation and handoffs.

## Project run steps

### Prerequisites

- Docker Compose v2
- External PostgreSQL 16+ with TimescaleDB extension
- Databases: `flow_finance_ai` (Flow) and `firefly` (Firefly III)
- Copy `.env.example` to `.env` and set `DATABASE_*`, `FIREFLY_*`, `FIREFLY_PERSONAL_ACCESS_TOKEN`
- Optional: OIDC IdP or `AUTH_DEV_BYPASS=true` for local dev

### Local run

```bash
docker compose --profile minimal --profile bundled-firefly up --build
```

- Flow Finance AI: `http://localhost:8080` (`FLOW_PORT`)
- Firefly III: `http://localhost:8081` (`FIREFLY_PORT`)
- Grafana: `http://localhost:3000` (`GRAFANA_PORT`)

> **Note:** After US-0010 (`DEC-0056`), bundled Firefly is on profile `bundled-firefly`. For app + Grafana only (external DB + external Firefly), use `--profile minimal` without `bundled-firefly`.

### Omniflow external deploy (US-0010)

**Release (S0010):** `0.10.0-us0010` — operator notes `handoffs/releases/S0010-release-notes.md` (gate summary + verify steps). Publish skipped (`RELEASE_PUBLISH_MODE=disabled`).

Attach Flow Finance AI to an existing Debian host where Firefly (`firefly`), PostgreSQL (`postgres`), and Traefik already share Docker network **`traefik`**. Host stack paths (read-only alignment — do not edit from this repo):

- Firefly: `/workdir/firefly/docker-compose.yml` — container `firefly`, public UI `https://finance.omniflow.cc`
- Postgres: `/workdir/services/docker-compose.yml` — container `postgres`
- Traefik: `/workdir/networking/docker-compose.yml` — middleware `auth`, cert resolver `myresolver`

**Requirements:** Docker Compose **≥2.24** (merge `!reset` support). Never read or commit operator host `.env` or Traefik credential files.

#### 1. Preflight — shared Postgres / TimescaleDB (US-0012 auto-provision)

**Release (S0012):** `0.12.0-us0012` — operator notes `handoffs/releases/S0012-release-notes.md` (gate summary + bootstrap verify steps). Publish skipped (`RELEASE_PUBLISH_MODE=disabled`).

**Database create (automated):** On first backend start, Flow Finance AI runs **`ensure_database`** (DEC-0058) before migrations: idempotent `CREATE DATABASE … OWNER` for `DATABASE_NAME` when absent, then `CREATE EXTENSION IF NOT EXISTS timescaledb` on the app DB using maintenance credentials. Existing databases are never dropped or recreated.

| Scenario | Configuration |
|----------|----------------|
| App role has `CREATEDB` (greenfield dev, CI superuser) | Omit `DATABASE_BOOTSTRAP_URL` — maintenance URL derived from `DATABASE_*` → `…/postgres` |
| App role lacks `CREATEDB` (typical shared homelab `postgres`) | Set **`DATABASE_BOOTSTRAP_URL`** to admin/superuser URL (`postgres://…@postgres:5432/postgres`); never commit |

**TimescaleDB host install (operator prerequisite — unchanged):** Server packages + `shared_preload_libraries = 'timescaledb'` on host Postgres must be installed before first start. Bootstrap fails closed with log `bootstrap_reason=database_bootstrap_failed_timescaledb` when extension files are missing.

On database **`flow_finance_ai`** (after first successful start or manual troubleshooting):

```sql
SELECT extname, extversion FROM pg_extension WHERE extname = 'timescaledb';
```

- **Pass:** non-null `extversion`
- **Fail:** install TimescaleDB packages on host Postgres, set `shared_preload_libraries = 'timescaledb'`, restart Postgres, redeploy backend (bootstrap retries extension) or run manually: `CREATE EXTENSION IF NOT EXISTS timescaledb CASCADE;` on `flow_finance_ai`

**Bootstrap log triage (`bootstrap_reason`):** `database_bootstrap_started`, `database_bootstrap_created`, `database_bootstrap_skipped_exists`, `database_bootstrap_extension_ok` (success path); `database_bootstrap_failed_privilege` (set `DATABASE_BOOTSTRAP_URL`), `database_bootstrap_failed_timescaledb` (host packages), `database_bootstrap_failed_connect` (network/credentials). Passwords are never logged.

**Security:** Prefer a dedicated bootstrap role with `CREATEDB` only; rotate bootstrap URL after first deploy if policy requires.

See also: `docs/engineering/architecture.md` (US-0012).

Firefly's database on the same `postgres` container does **not** imply TimescaleDB on `flow_finance_ai`.

#### 2. Operator `.env` (names only — set values on host)

Copy `.env.example` → `.env` on the deploy host. Minimum for external profile:

| Variable | External value |
|----------|----------------|
| `COMPOSE_FILE` | `docker-compose.yml:docker-compose.external.yml` |
| `COMPOSE_PROFILES` | `external` |
| `DATABASE_HOST` | `postgres` (overlay default `${DATABASE_HOST:-postgres}`; **required** explicit value on omniflow — see mis-host table below) |
| `DATABASE_PASSWORD` | operator secret |
| `FIREFLY_BASE_URL` | `http://firefly:8080` |
| `FIREFLY_PERSONAL_ACCESS_TOKEN` | Firefly PAT (server-side) — **must be non-empty** in operator `.env`; whitespace-only is treated as unset (sync fail-fast `firefly_personal_access_token_missing`; see BUG-0002 / Q0008) |
| `GRAFANA_ADMIN_PASSWORD` | replace default `admin` |

Optional overrides: `TRAEFIK_HOST` (default `financegnome.omniflow.cc`), `TRAEFIK_MIDDLEWARE` (default `auth`), `GRAFANA_TRAEFIK_HOST` (empty = Grafana internal-only).

**Profile rule:** use **`external` only**. Do **not** combine with `minimal`, `standard`, `full`, or `bundled-firefly` (starts duplicate Firefly).

If using `full` profile on the same host, set `STATS_FORECAST_PORT=8091` (host port 8090 is used by `firefly_product_manager`).

**Omniflow mis-host (`DATABASE_HOST`) — BUG-0003 / DEC-0056 / R-0052:**

Copying greenfield `.env.example` defaults into an external deploy without changing `DATABASE_HOST` leaves `host.docker.internal` in operator `.env`. That overrides `docker-compose.external.yml` `${DATABASE_HOST:-postgres}`. On the shared `traefik` network, `host.docker.internal` is unreachable from `flow-finance-ai` and `grafana` → SQLx pool timeouts (~30s) → widespread product API **500**. `GET /api/v1/settings` may still return **200** (config read without DB round-trip) while showing `database_host: host.docker.internal`.

| Symptom | Typical cause | Remediation (F1) |
|---------|---------------|------------------|
| Many `GET /api/v1/*` return **500** after ~30s | Wrong `DATABASE_HOST` in operator `.env` | Set `DATABASE_HOST=postgres`; recreate `flow-finance-ai` + `grafana` |
| Settings **200** but `database_host` ≠ `postgres` | Same mis-host override | Same as above; verify `GET /api/v1/settings` → `database_host: postgres` |
| Grafana `POST …/analytics/grafana/api/ds/query` **400** db error | Grafana datasource uses same bad host | Same recreate; confirm grafana container env `DATABASE_HOST=postgres` |
| Bitunix test **400** `unknown exchange` (fast, &lt;1s) | Connector registry gap (separate **G** slice) | Deploy G1 code; not fixed by F1 alone |

**F1 operator steps (acceptance F/H — not committed):**

1. On deploy host `.env`: `DATABASE_HOST=postgres` (names only in docs — set real value on host).
2. Recreate backend and Grafana: `docker compose -f docker-compose.yml -f docker-compose.external.yml --profile external up -d --force-recreate flow-finance-ai grafana`
3. Verify: `GET /api/v1/settings` → `database_host: postgres`; sample product GETs **200** in normal latency; Grafana `ds/query` **200**.

#### 3. Deploy

```bash
docker compose -f docker-compose.yml -f docker-compose.external.yml --profile external up -d --build
```

Expected project services: **`flow-finance-ai`**, **`grafana`** only — no `firefly-iii`, no `postgres` containers from this repo.

#### 4. OIDC (when auth enabled — documentation only)

Register with your IdP (not automated):

- Redirect URI: `https://financegnome.omniflow.cc/callback`
- Post-logout: `https://financegnome.omniflow.cc/`
- Origin: `https://financegnome.omniflow.cc`

Set `VITE_OIDC_*` and `OIDC_*` per `.env.example`. AC-6 smoke may use `AUTH_DEV_BYPASS=true`; production auth-on deployments must complete IdP registration. Traefik basic-auth (`auth` middleware) is separate from app OIDC.

#### 5. Grafana access (internal default)

External overlay does **not** publish Grafana on a public Traefik host by default. Access options:

- SSH tunnel to host, then `curl http://127.0.0.1:3000` only if port published locally (overlay uses `!reset` — prefer network access)
- From a container on `traefik` network: `curl http://grafana:3000/login`
- Optional public host: set `GRAFANA_TRAEFIK_HOST` (e.g. `grafana-financegnome.omniflow.cc`) before deploy

#### 6. Operator smoke test (AC-6)

Record results on the Debian host. Use placeholder secrets in docs only — never commit credentials.

| Step | Check | Pass |
|------|-------|------|
| TimescaleDB | `psql` against `flow_finance_ai`: `SELECT extversion FROM pg_extension WHERE extname='timescaledb';` | Non-null version |
| Firefly DNS | From `traefik` network: `curl -sf http://firefly:8080/api/v1/about` | HTTP 200 |
| PAT set | `docker compose ... exec flow-finance-ai printenv FIREFLY_PERSONAL_ACCESS_TOKEN` | Non-empty (verify length only — **do not log or commit the value**) |
| PAT guard | After deploy with empty/missing PAT, manual sync `last_run.error_message` | Contains `firefly_personal_access_token_missing` (no Firefly 401 from blank Bearer) |
| Readiness hint | `curl -sf http://flow-finance-ai:8080/health/ready` (from `traefik` network) | JSON includes `firefly_pat_configured: true` when PAT loaded |
| Backend health | From `traefik` network: `curl -sf http://flow-finance-ai:8080/health` | OK JSON |
| TLS route | `curl -sfI https://financegnome.omniflow.cc/health -u '<basic-auth-user>:<pass>'` | HTTP 200, valid cert |
| Auth middleware | `curl -sfI https://financegnome.omniflow.cc/` without credentials | HTTP 401 |
| No duplicate Firefly | `docker compose ... ps --services` with external profile | No `firefly-iii` |
| Migrations | Backend logs after fresh DB | No migration panic; health OK |

#### 7. Compose config regression (CI / local)

```bash
export DATABASE_PASSWORD=ci FIREFLY_APP_KEY=base64:32RandomCharactersMinimumRequired== \
       FIREFLY_DB_PASSWORD=ci AUTHENTIK_SECRET_KEY=ci
docker compose -f docker-compose.yml -f docker-compose.external.yml \
  --profile external config --services | sort
# expect: flow-finance-ai, grafana
```

See `scripts/compose-config-check.sh` (invoked by `bash tests/run-tests.sh`).

**Optional bootstrap integration CI (US-0012 / AC-6):** Set `DATABASE_BOOTSTRAP_TEST_URL` to a superuser maintenance URL (`postgres://…/postgres`) and run `cargo test --test database_bootstrap_integration`. Suggested fixtures: `postgres:16` for create-if-missing path; `timescale/timescaledb` image for extension path. Skipped gracefully when env unset.

#### 8. Unified analytics (US-0011 / DEC-0057)

**Release (S0011):** `0.11.0-us0011` — operator notes `handoffs/releases/S0011-release-notes.md` and `docs/user-guides/US-0011.md`. Requires US-0010 external deploy (Grafana on `traefik` network, internal-only by default).

**Deploy:** same as § Omniflow external deploy — `docker compose -f docker-compose.yml -f docker-compose.external.yml --profile external up -d --build`. Grafana ships with anonymous Viewer + embedding enabled in compose (DEC-0057).

**Env (names only):**

| Variable | Purpose |
|----------|---------|
| `GRAFANA_UPSTREAM` | Backend proxy target (default `http://grafana:3000`) |
| `VITE_GRAFANA_EMBED_BASE` | SPA iframe base (default `/analytics/grafana`) |
| `GRAFANA_TRAEFIK_HOST` | empty = no public Grafana host (default UX) |

**Routes:** SPA `/analytics/{slug}` (six dashboards); reverse proxy `/analytics/grafana/` (outside `/api/v1` JWT). Traefik `auth` protects edge; browser reuses session for same-origin iframe requests.

**Operator smoke (post-deploy):**

| Step | Check | Pass |
|------|-------|------|
| App health | `curl -sfI https://financegnome.omniflow.cc/health` (with basic-auth) | HTTP 200 |
| Proxy health | `curl -s -o /dev/null -w "%{http_code}" https://financegnome.omniflow.cc/analytics/grafana/api/health` | 200 |
| Iframe routes | Load each `/analytics/{slug}` under Traefik session | Dashboard renders |
| Live refresh | One panel WebSocket via `/analytics/grafana/api/live/` | WS connects |
| Regression | Forecast, Wealth, Planning, Subscriptions, Alerts | Pages load |

**Automated regression:**

```bash
cd backend && cargo test --test analytics_proxy_integration --test product_routes_regression
cd frontend && npm run build
```

**Boundaries:** no default public `GRAFANA_TRAEFIK_HOST`; no `GF_SERVER_SERVE_FROM_SUB_PATH`; no Grafana JSON/SQL uid changes (DEC-0012). If anonymous Grafana is insufficient, open auth-proxy DEC — do not expose Grafana publicly without decision.

#### 9. BUG-0001 hotfix — auth stub + Grafana root URL (Q0007 / released 2026-06-04)

**Release:** BUG-0001 **DONE** — operator notes `handoffs/releases/Q0007-release-notes.md`. Fixes omniflow production regressions on US-0010 external profile: **(A)** `DevBypassAuthProvider` when OIDC unset; **(B)** `GF_SERVER_ROOT_URL` so Grafana assets resolve under `/analytics/grafana/public/...` (not site-root `/public/...` **404**). **B2** proxy HTML rewrite not implemented — escalate only if B1 smoke fails.

**Deploy (explicit — both services must pick up A1 + B1):**

```bash
AUTHENTIK_SECRET_KEY=unused-external-profile docker compose \
  -f docker-compose.yml -f docker-compose.external.yml \
  --profile external up -d --build flow-finance-ai grafana
```

Full external stack if needed:

```bash
docker compose -f docker-compose.yml -f docker-compose.external.yml \
  --profile external up -d --build
```

**Env (names only — add to operator `.env` when overriding host):**

| Variable | Purpose |
|----------|---------|
| `GF_SERVER_ROOT_URL` | Grafana public root with **trailing slash** (compose default: `https://financegnome.omniflow.cc/analytics/grafana/`) |
| `AUTH_DEV_BYPASS` | `true` on external profile when OIDC unset — API dev bypass (Q0006); pairs with A1 frontend stub |

**Operator smoke (post-deploy):**

| Step | Check | Pass |
|------|-------|------|
| Frontend unit | `cd frontend && npm test` | 2/2 |
| Frontend build | `cd frontend && npm run build` | exit 0 |
| App health | `curl -sfI https://financegnome.omniflow.cc/health` (with basic-auth) | HTTP 200 |
| Proxy health | `curl -s -o /dev/null -w "%{http_code}" https://financegnome.omniflow.cc/analytics/grafana/api/health` | 200 |
| Six embeds | `GET /analytics/grafana/d/{uid}/{slug}?kiosk=tv` for platform-health, cashflow, subscriptions, budgets, portfolio, forecast-horizons | 200 each |
| Prefixed assets | Sample `/analytics/grafana/public/build/*.css` and `public/img/fav32.png` from live Grafana HTML | 200 |
| Site-root assets | `GET /public/build/...` at site origin | **401** (Traefik auth), not **404** |
| Chat (A) | Hard refresh SPA; open **AI Chat** | No `useAuth` / `user` TypeError (Traefik auth required for shell) |

**Automated regression:**

```bash
cd frontend && npm test && npm run build
```

**Boundaries:** no `GF_SERVER_SERVE_FROM_SUB_PATH`; no B2 `rewrite_grafana_html` unless operator reports persistent site-root `/public/` **404** after B1 redeploy.

#### 10. BUG-0002 hotfix — Firefly PAT guard, risk-score 200, exchange effective enabled (Q0008 / released 2026-06-05)

**Release:** BUG-0002 **DONE** — operator notes `handoffs/releases/Q0008-release-notes.md`. Fixes omniflow production integration on US-0010 external profile: **(C)** Firefly sync with non-empty PAT + fail-fast guard; **(D)** `GET /api/v1/plans/risk-score` always **200** (`ok` \| `no_score`); **(E)** exchange settings effective enabled when credentials present; Binance default `enabled=false`.

**Deploy (explicit — backend + embedded frontend):**

```bash
AUTHENTIK_SECRET_KEY=unused-external-profile docker compose \
  -f docker-compose.yml -f docker-compose.external.yml \
  --profile external up -d --build flow-finance-ai
```

Full external stack if needed:

```bash
docker compose -f docker-compose.yml -f docker-compose.external.yml \
  --profile external up -d --build
```

**Env (names only — operator `.env`):**

| Variable | Purpose |
|----------|---------|
| `FIREFLY_BASE_URL` | In-network Firefly (default `http://firefly:8080`) |
| `FIREFLY_PERSONAL_ACCESS_TOKEN` | **Non-empty** PAT; whitespace-only treated as unset (sync fail-fast) |
| `BITUNIX_API_KEY` / `BITUNIX_API_SECRET` | Optional — E row verification |

**Operator smoke (post-deploy):**

| Step | Check | Pass |
|------|-------|------|
| Backend unit | `cd backend && cargo test --lib` | 103/103 |
| Frontend unit | `cd frontend && npm test` | 2/2 |
| Frontend build | `cd frontend && npm run build` | exit 0 |
| App health | `curl -sf https://financegnome.omniflow.cc/health` | HTTP 200 |
| Row C | `GET /api/v1/sync/status`, `/api/v1/sync/entities` | 200; success; non-zero counts |
| Row D | `GET /api/v1/plans/risk-score` | **200** (not 404) |
| Row E | `GET /api/v1/settings` | Bitunix enabled+configured when env set; Binance disabled when unset |
| Optional | `POST /api/v1/exchanges/bitunix/test` | 200 |

**Automated regression:**

```bash
cd backend && cargo test --lib
cd frontend && npm test && npm run build
```

**Boundaries:** no Traefik router changes; no JWT/auth stack changes; OIDC regression deferred to operator browser smoke.

#### 11. BUG-0003 hotfix — DATABASE_HOST mis-host, Bitunix registry, Grafana SQL (Q0009 / released 2026-06-05)

**Release:** BUG-0003 **DONE** — operator notes `handoffs/releases/Q0009-release-notes.md`. Fixes omniflow production API cascade on US-0010 external profile: **(F)** product APIs return **200** with `DATABASE_HOST=postgres`; **(G)** Bitunix test registers connector via `effective_enabled()`; **(H)** Grafana SQL executes via in-network `postgres`.

**Deploy (explicit — backend + Grafana):**

```bash
AUTHENTIK_SECRET_KEY=unused-external-profile docker compose \
  -f docker-compose.yml -f docker-compose.external.yml \
  --profile external up -d --build flow-finance-ai grafana
```

**Operator prerequisite (F1 — required for F/H acceptance):**

1. On deploy host `.env`: `DATABASE_HOST=postgres` (not `host.docker.internal`).
2. Recreate services:

```bash
docker compose -f docker-compose.yml -f docker-compose.external.yml \
  --profile external up -d --force-recreate flow-finance-ai grafana
```

**Env (names only — operator `.env`):**

| Variable | Purpose |
|----------|---------|
| `DATABASE_HOST` | **`postgres`** on external profile (F1 / H) |
| `BITUNIX_API_KEY` / `BITUNIX_API_SECRET` | Optional — G row verification |

**Operator smoke (post-deploy):**

| Step | Check | Pass |
|------|-------|------|
| Backend unit | `cd backend && cargo test --lib` | 103/103 |
| Frontend unit | `cd frontend && npm test` | 2/2 |
| Frontend build | `cd frontend && npm run build` | exit 0 |
| App health | `curl -sf https://financegnome.omniflow.cc/health` | HTTP 200 |
| Row F | `GET /api/v1/settings` | `database_host: postgres` |
| Row F | `GET /api/v1/alerts/unread-count`, `/sync/entities`, `/exchanges` | **200** &lt;0.1s |
| Row G | `POST /api/v1/exchanges/bitunix/test` | **200** (not 400 unknown exchange) |
| Row H | `POST /analytics/grafana/api/ds/query` | **200**; SQL executes |

**Automated regression:**

```bash
cd backend && cargo test --lib
cd frontend && npm test && npm run build
```

**Boundaries:** no Traefik router changes; G2 futures auth spike skipped (gated); OIDC regression deferred to operator browser smoke.

#### 12. BUG-0004 hotfix — post-sync pipeline empty analytics (Q0011 / released 2026-06-05)

**Release:** BUG-0004 **DONE** — operator notes `handoffs/releases/Q0011-release-notes.md`. Fixes omniflow post-sync analytics on US-0010 external profile: **(I)** exchange sync terminal status via `finish_sync_run` on `RunMode::ExchangesOnly`; **(J)** subscription detection with DEC-0061 payee key fallbacks + empty-state UX; **(K)** portfolio Grafana UNION SQL fix; **(L)** Firefly account balance parse (DEC-0060) + wealth NULL handling + forecast series after recompute.

**Deploy (explicit — backend + Grafana):**

```bash
AUTHENTIK_SECRET_KEY=unused-external-profile docker compose \
  -f docker-compose.yml -f docker-compose.external.yml \
  --profile external up -d --build flow-finance-ai grafana
```

**Operator prerequisite (L3 — required for I/J/K/L acceptance):**

1. Deploy Q0011 image to omniflow.
2. Trigger **Manual Full Firefly sync** — account balance backfill via DEC-0060 upsert.
3. Trigger **Manual exchange sync** — verify I1 terminal status path.
4. Recreate services if needed:

```bash
docker compose -f docker-compose.yml -f docker-compose.external.yml \
  --profile external up -d --force-recreate flow-finance-ai grafana
```

**Env (names only — operator `.env`):**

| Variable | Purpose |
|----------|---------|
| `DATABASE_HOST` | **`postgres`** on external profile (prerequisite from §11) |
| `FIREFLY_BASE_URL` / `FIREFLY_PERSONAL_ACCESS_TOKEN` | Full Firefly sync gate (L3) |

**Operator smoke (post-deploy + sync):**

| Step | Check | Pass |
|------|-------|------|
| Backend unit | `cd backend && cargo test --lib` | 110/110 |
| Frontend unit | `cd frontend && npm test` | 2/2 |
| Frontend build | `cd frontend && npm run build` | exit 0 |
| App health | `curl -sf https://financegnome.omniflow.cc/health` | HTTP 200 |
| Row I | `GET /api/v1/sync/status` after manual exchange sync | `state: success`; `finished_at` set |
| Row J | `GET /api/v1/subscriptions` | Non-empty pending patterns with `payee_key` |
| Row K | `POST /analytics/grafana/api/ds/query` (portfolio pie) | **200**; no UNION syntax error |
| Row L | `GET /api/v1/wealth` | Firefly asset accounts populated |
| Row L | `GET /api/v1/forecast/daily?account_id=<id>` | **200** with populated series |

**Automated regression:**

```bash
cd backend && cargo test --lib
cd frontend && npm test && npm run build
```

**Boundaries:** no Traefik router changes; 10 historical pre-I1 stuck `scheduled_exchanges` rows out of scope; OIDC/J2 UI regression deferred to operator browser smoke.

#### 13. BUG-0005 hotfix — Bitunix futures multi-product sync (Q0012 / released 2026-06-05)

**Release:** BUG-0005 **DONE** — operator notes `handoffs/releases/Q0012-release-notes.md`. Fixes omniflow Bitunix exchange sync on US-0010 external profile: **(M)** futures wallet + linear position ingestion (not spot-only); **(N)** `fapi.bitunix.com` header-auth client (DEC-0062), `effective_enabled_futures()` (DEC-0063), dual-path test (N4); **(O)** wealth crypto aggregates combined spot + futures holdings (DEC-0064).

**Deploy (explicit — backend only):**

```bash
AUTHENTIK_SECRET_KEY=unused-external-profile docker compose \
  -f docker-compose.yml -f docker-compose.external.yml \
  --profile external up -d --build flow-finance-ai
```

**Operator prerequisite (O1 — required for M/N/O acceptance):**

1. Deploy Q0012 backend image to omniflow.
2. Ensure `BITUNIX_API_KEY` / `BITUNIX_API_SECRET` configured (names only).
3. Trigger **Manual exchange sync** — verify futures-enabled ingestion path.
4. Recreate service if needed:

```bash
docker compose -f docker-compose.yml -f docker-compose.external.yml \
  --profile external up -d --force-recreate flow-finance-ai
```

**Env (names only — operator `.env`):**

| Variable | Purpose |
|----------|---------|
| `BITUNIX_API_KEY` / `BITUNIX_API_SECRET` | Bitunix read-only credentials |
| `DATABASE_HOST` | **`postgres`** on external profile (prerequisite from §11) |

**Operator smoke (post-deploy + exchange sync):**

| Step | Check | Pass |
|------|-------|------|
| Backend unit | `cd backend && cargo test --lib` | 123/123 |
| Frontend unit | `cd frontend && npm test` | 2/2 |
| Frontend build | `cd frontend && npm run build` | exit 0 |
| App health | `curl -sf https://financegnome.omniflow.cc/health` | HTTP 200 |
| Row M | `GET /api/v1/exchanges` after manual exchange sync | Bitunix `holdings` > 0 |
| Row N | `GET /api/v1/settings` | `futures_base_url: https://fapi.bitunix.com`; `enabled_futures: true` |
| Row N | `POST /api/v1/exchanges/bitunix/test` | **200** — `Spot: OK; Futures: OK` |
| Row O | `GET /api/v1/wealth` | `crypto.holdings_count` > 0; `crypto_placeholder: false` |

**Automated regression:**

```bash
cd backend && cargo test --lib
cd frontend && npm test && npm run build
```

**Boundaries:** read-only key constraint preserved; no Traefik router changes; `crypto.subtotal_eur: 0.0` with unpriced linear positions acceptable per DEC-0064; OIDC/browser regression deferred to operator smoke.

#### 14. BUG-0006 hotfix — AI get_transactions empty aggregates (Q0010 / released 2026-06-05)

**Release:** BUG-0006 **DONE** — operator notes `handoffs/releases/Q0010-release-notes.md`. Fixes omniflow AI Chat on US-0010 external profile when Firefly mirror has synced transactions: **(P)** category/spending answers use `get_transactions` aggregates (not false "no expenses"); **(Q)** Firefly sync persists `category_id`, parsed `date`, and DEC-0059 signed `amount` on mirror rows; **(R)** aggregate JSON includes period totals/counts + `period_status` under `allow_raw_transactions=false`.

**Deploy (explicit — backend only):**

```bash
AUTHENTIK_SECRET_KEY=unused-external-profile docker compose \
  -f docker-compose.yml -f docker-compose.external.yml \
  --profile external up -d --build flow-finance-ai
```

**Operator prerequisite (P1 — required for P/Q/R acceptance):**

1. Deploy Q0010 backend image to omniflow.
2. Ensure `FIREFLY_BASE_URL` / `FIREFLY_PERSONAL_ACCESS_TOKEN` configured (names only).
3. Trigger **Manual Firefly sync** — upsert backfill for category/date/amount (no SQL migration).
4. Recreate service if needed:

```bash
docker compose -f docker-compose.yml -f docker-compose.external.yml \
  --profile external up -d --force-recreate flow-finance-ai
```

**Env (names only — operator `.env`):**

| Variable | Purpose |
|----------|---------|
| `FIREFLY_BASE_URL` / `FIREFLY_PERSONAL_ACCESS_TOKEN` | Firefly sync gate (P1) |
| `DATABASE_HOST` | **`postgres`** on external profile (prerequisite from §11) |

**Operator smoke (post-deploy + Firefly sync):**

| Step | Check | Pass |
|------|-------|------|
| Backend unit | `cd backend && cargo test --lib` | 123/123 |
| Frontend unit | `cd frontend && npm test` | 2/2 |
| Frontend build | `cd frontend && npm run build` | exit 0 |
| App health | `curl -sf https://financegnome.omniflow.cc/health` | HTTP 200 |
| Row Q | Operator SQL post-sync | ≥917/922 `category_id`; ≥919/922 `date`; ≥865/922 `amount < 0` |
| Row R | `GET /api/v1/settings` | `allow_raw_transactions: false` |
| Row P | `POST /api/v1/chat/completions` (populated month) | Non-zero tx count + outflow via `get_transactions` |
| Row P/R | `POST /api/v1/chat/completions` (pre-ledger month) | Correct `no_rows` — not false "no expenses" |
| Sync | `GET /api/v1/sync/entities` | `transactions.count` ≥922 |

**Automated regression:**

```bash
cd backend && cargo test --lib
cd frontend && npm test && npm run build
```

**Boundaries:** six-tool registry + DEC-0032 privacy redaction unchanged; no Traefik router changes; OIDC/browser regression deferred to operator smoke. **Queue closure:** last OPEN bug in defect queue (BUG-0001–BUG-0006 all DONE).

#### 15. BUG-0010 hotfix — Forecast wrong numbers, empty wealth, ML posture (Q0013 / released 2026-06-05)

**Release:** BUG-0010 **DONE** — operator notes `handoffs/releases/Q0013-release-notes.md`. Fixes omniflow forecast/wealth on US-0010 external profile after Full Firefly sync: **(AA)** signed starting balances match mirror with `balance_warnings` for overdrawn accounts (not silent -25365.78); **(AB)** wealth includes overdrawn asset accounts with honest signed `total_eur`; **(AC)** meta `ml_skipped_reason: sidecar_disabled` and UI "not enabled" copy when ML off (AC3 ML production → **US-0013** epic).

**Deploy (explicit — backend + frontend):**

```bash
AUTHENTIK_SECRET_KEY=unused-external-profile docker compose \
  -f docker-compose.yml -f docker-compose.external.yml \
  --profile external up -d --build flow-finance-ai
```

**Operator prerequisite (V1 — required for AA/AB/AC acceptance):**

1. Deploy Q0013 backend + frontend image to omniflow.
2. Ensure `FIREFLY_BASE_URL` / `FIREFLY_PERSONAL_ACCESS_TOKEN` configured (names only).
3. Trigger **Manual Full Firefly sync** — mirror balance backfill (no SQL migration).
4. Recreate service if needed:

```bash
docker compose -f docker-compose.yml -f docker-compose.external.yml \
  --profile external up -d --force-recreate flow-finance-ai
```

**Env (names only — operator `.env`):**

| Variable | Purpose |
|----------|---------|
| `FIREFLY_BASE_URL` / `FIREFLY_PERSONAL_ACCESS_TOKEN` | Full Firefly sync gate |
| `DATABASE_HOST` | **`postgres`** on external profile (prerequisite from §11) |

**Operator smoke (post-deploy + Full Firefly sync):**

| Step | Check | Pass |
|------|-------|------|
| Backend unit | `cd backend && cargo test --lib` | 131/131 |
| Frontend unit | `cd frontend && npm test` | 2/2 |
| Frontend build | `cd frontend && npm run build` | exit 0 |
| App health | `curl -sf https://financegnome.omniflow.cc/health` | HTTP 200 |
| Row AA | `GET /api/v1/forecast/long-term?account_id=114&horizon=3` | Start matches mirror; series populated |
| Row AA | `GET /api/v1/forecast/meta` | `balance_warnings` for overdrawn acct when applicable |
| Row AB | `GET /api/v1/wealth` | ≥3 accounts; Giro 114 `is_overdrawn: true`; signed `total_eur` |
| Row AB | `GET /api/v1/wealth/history?days=30` | Post-sync snapshot matches |
| Row AC | `GET /api/v1/forecast/meta` | `ml_skipped_reason: sidecar_disabled` |
| Sync | `GET /api/v1/sync/status` | Last manual success run (e.g. `3e44fbfb`) |

**Automated regression:**

```bash
cd backend && cargo test --lib
cd frontend && npm test && npm run build
```

**Boundaries:** DEC-0007 baseline math unchanged; DEC-0049 ML disabled on external profile preserved; AC3 ML sidecar production deferred **US-0013**; no Traefik router changes; OIDC/browser regression deferred to operator smoke.

#### 16. BUG-0012 hotfix — Forecast monthly Income/Fixed buckets always zero (Q0014+Q0015)

**Release:** BUG-0012 **DONE** (2026-06-06). Fixes omniflow monthly forecast decomposition on US-0010 external profile: **(AG)** Income bucket populated from categorized recurring inflows and household revenue-account salary merge (not net-delta sign); **(AH)** Fixed bucket populated from categorized recurring outflows via `category_id` → TOML map, payee-key due matching, and standing-order forecast scope (**DEC-0067** + Q0015 follow-up).

**Production proof (account 114, post-deploy):** Jun `fixed_costs: 2073.85`; Jul+ `income: 3266.16`. Release notes: `handoffs/releases/Q0014-release-notes.md`.

**Deploy (backend only):**

```bash
AUTHENTIK_SECRET_KEY=unused-external-profile docker compose \
  -f docker-compose.yml -f docker-compose.external.yml \
  --profile external up -d --build flow-finance-ai
```

**Operator prerequisite (V1 — required for AG/AH acceptance):**

1. Deploy Q0014 backend image to omniflow.
2. Ensure `FIREFLY_BASE_URL` / `FIREFLY_PERSONAL_ACCESS_TOKEN` configured (names only).
3. Trigger **Manual Full Firefly sync** + forecast recompute.
4. Recreate service if needed:

```bash
docker compose -f docker-compose.yml -f docker-compose.external.yml \
  --profile external up -d --force-recreate flow-finance-ai
```

**TOML category bucket checklist (conditional — if AG/AH still zero after sync):**

German/custom Firefly category names may not match `default.toml` keys. Resolution path uses **lowercased category name** as TOML key:

1. List mirror category **names** for income/fixed rows in the acceptance month (Firefly UI or `SELECT name FROM categories JOIN transactions ON …`).
2. Add matching keys under `[forecast.category_buckets]` in operator TOML:

```toml
[forecast.category_buckets]
# defaults (already in default.toml)
salary = "income"
rent = "fixed"
utilities = "fixed"
# operator extensions — lowercase name = key
gehalt = "income"
lohn = "income"
miete = "fixed"
"miete nebenkosten" = "fixed"
strom = "fixed"
versicherung = "fixed"
```

3. Recreate service after TOML change; trigger forecast recompute (Full sync or manual recompute).
4. Re-smoke AG/AH probes below.

**Operator smoke (post-deploy + Full Firefly sync):**

| Step | Check | Pass |
|------|-------|------|
| Backend unit | `cd backend && cargo test --lib` | 142/142 |
| Row AG | `GET /api/v1/forecast/monthly?account_id=<funded>` | First month `income > 0` |
| Row AG | `/forecast` Monthly tab | Income card non-zero |
| Row AH | `GET /api/v1/forecast/monthly?account_id=<funded>` | `fixed_costs > 0` |
| Row AH | `/forecast` Monthly tab | Fixed card non-zero |
| App health | `curl -sf https://financegnome.omniflow.cc/health` | HTTP 200 |

**Boundaries:** Daily balance / milestones / horizons unchanged; rolling residual → Variable only; unmapped categories → Variable; US-0015 AI buckets and US-0013 ML out of scope; no frontend code changes required; OIDC/browser regression deferred to operator smoke.

#### 17. BUG-0009 hotfix — Grafana empty panels & account overview (Q0016)

**Status:** released (2026-06-06) — Q0016 verify-work + release PASS. Provisioning-only fix per **DEC-0068**: portfolio breakdown LATERAL SQL, cross-account overview table, `$account_id` ABS(balance) default, ML honest empty-state banner.

**Deploy (Grafana provisioning — no backend rebuild required unless image bundles JSON):**

```bash
AUTHENTIK_SECRET_KEY=unused-external-profile docker compose \
  -f docker-compose.yml -f docker-compose.external.yml \
  --profile external up -d --build grafana
```

**Operator gate — GRAFANA_PROVISIONING_RELOAD (required before V1 smoke):**

```bash
docker compose -f docker-compose.yml -f docker-compose.external.yml \
  --profile external up -d --force-recreate grafana
```

**Grafana UI warning:** Do **not** click **Save** on analytics dashboards after changing `$account_id` or other variables — Grafana persists a `current` block to its DB that overrides provisioning JSON and breaks other deployments. If baked, reset dashboard from repo provisioning or delete dashboard DB row and reload provisioning.

**Operator smoke (post-deploy + provisioning reload):**

| Step | Check | Pass |
|------|-------|------|
| T1 local | `cd backend && cargo test --test grafana_provisioning_bug0009` | 6/6 |
| Row Y | `/analytics/cashflow` default load (no manual account pick) | Non-flat series (funded account, not zero wallet) |
| Row Y | `/analytics/forecast-horizons` | ML status banner visible; ML panels show `ML unavailable` |
| Row Z | `/analytics/portfolio` | Overview table 3 account rows; `total_eur` stat above fold |
| Regression | Six `/analytics/{slug}` routes + `POST …/ds/query` | **200** (BUG-0003 H, BUG-0004 K) |
| Z3 supplementary | `/wealth` React page | Per-account detail — not AC Z substitute |

**Boundaries:** US-0013 ML enablement out of scope; no backend/React code; seventh overview dashboard rejected; Grafana dynamic hide rules rejected.

#### 18. BUG-0007 hotfix — AI merchant/category discovery (Q0017 / released 2026-06-08)

**Release:** BUG-0007 **DONE** — operator notes `handoffs/releases/Q0017-release-notes.md`. Fixes omniflow AI Chat on US-0010 external profile when Firefly mirror has synced transactions: **(S)** subscription merchant enumeration with named `display_name` / `merchant_names[]` (privacy label exemption); **(T)** `category_search` ILIKE resolution + mirror date bounds in empty-state; **(U)** cross-signal fusion rules + audit `result_rows`. Six-tool registry + `allow_raw_transactions=false` preserved per **DEC-0069**.

**Deploy (explicit — backend only):**

```bash
AUTHENTIK_SECRET_KEY=unused-external-profile docker compose \
  -f docker-compose.yml -f docker-compose.external.yml \
  --profile external up -d --build flow-finance-ai
```

**Operator gate — BACKEND_DEPLOY (required before V1 smoke):**

```bash
docker compose -f docker-compose.yml -f docker-compose.external.yml \
  --profile external up -d --force-recreate flow-finance-ai
```

**Env (names only — operator `.env`):**

| Variable | Purpose |
|----------|---------|
| `DATABASE_HOST` | **`postgres`** on external profile (prerequisite from §11) |
| `FIREFLY_BASE_URL` / `FIREFLY_PERSONAL_ACCESS_TOKEN` | Mirror data gate (922+ txs) |

**Operator smoke (post-deploy):**

| Step | Check | Pass |
|------|-------|------|
| Backend unit | `cd backend && cargo test --lib` | 150/150 |
| BUG-0007 suite | `cd backend && cargo test --test bug0007_ai_discovery` | 8/8 |
| App health | `curl -sf https://financegnome.omniflow.cc/health` | HTTP 200 |
| Row S | AI Chat streaming list follow-up | Named merchants (not Counterparty-* only) |
| Row T | Strom / electricity query | Data-backed amount or explicit empty-state with bounds |
| Row U | Multi-tool fusion prompt | Both tools invoked; named merchants in response |
| Regression | `GET /api/v1/settings` | `allow_raw_transactions: false`; six tools |

**Automated regression:**

```bash
cd backend && cargo test --lib
cd backend && cargo test --test bug0007_ai_discovery
```

**Advisory (non-blocking):** LLM may pass `group_by: month` inflating category totals — operator note only.

**Boundaries:** RAG deferred (V); payee aggregates deferred (B); BUG-0008 coordinate-only; no Traefik router changes; OIDC/browser regression deferred to operator smoke.

### Tests

```bash
bash tests/run-tests.sh
```

Optional integration (requires external DB with TimescaleDB extension):

```bash
export DATABASE_URL=postgres://user:pass@host:5432/flow_finance_ai
cd backend && cargo test --test firefly_integration
cd backend && cargo test --test forecast_integration
```

### Forecast verification (US-0002)

After sync with asset account data:

1. Open `http://localhost:8080/forecast` — Daily | Monthly | Long-term tabs with ECharts.
2. Grafana dashboards: **Cashflow** (`uid=cashflow`), **Forecast Horizons** (`uid=forecast-horizons`).
3. Confirm `/api/v1/forecast/meta` shows `last_computed_at` after sync-triggered recompute.

### Subscription verification (US-0003)

After sync with recurring expense transactions (≥3 occurrences per pattern):

1. Open `http://localhost:8080/subscriptions` — All | Pending review | Standing orders tabs.
2. Confirm pending cards show confidence badges; test confirm/reject and kind override dialog.
3. Open detail drawer on a confirmed subscription; verify lazy-loaded price history chart.
4. Grafana dashboard: **Subscriptions** (`uid=subscriptions`).
5. Confirm sync status shows `subscriptions` phase before `forecast`; rejected patterns excluded from forecast projection.
6. Optional: `DATABASE_URL=... cargo test --test subscriptions_integration` for pending persistence and alert proof.

### Planning verification (US-0004)

After sync with asset transactions and successful forecast recompute:

1. Open `http://localhost:8080/planning` — Scenarios | Compare | Plan vs Actual tabs.
2. Create a named plan; apply built-in templates (Current, Leasing, Savings mode, House purchase); add custom adjustments.
3. Create v2/v3 versions; verify Compare tab side-by-side metrics; confirm v4 attempt returns HTTP 409.
4. Set active plan; verify Plan vs Actual tab shows planned, Ist, and deviation (= actual − planned).
5. Grafana dashboard: **Budgets** (`uid=budgets`) — Plan, Ist, Abweichung panels for active plan.
6. Confirm plan changes do not modify Firefly data (read-only Ist from mirrored transactions).
7. Optional: `DATABASE_URL=... cargo test --test plans_integration` for plan CRUD/recompute/plan-vs-actual and Firefly write audit proof.

### Crypto exchange portfolio verification (US-0007)

After US-0001–US-0006 surfaces are operational, migration `007_exchanges_portfolio.sql` is applied, and read-only exchange keys are set:

1. Enable exchanges in `backend/config/default.toml` `[exchanges]`; set `BINANCE_*` / `BYBIT_*` / `BITUNIX_*` in `.env`; restart backend.
2. Settings → **Crypto exchanges** — **Test connection** per enabled exchange.
3. Sync Status → **Sync exchanges now** — confirm `"exchanges"` phase before `"alerts"` and per-exchange rows.
4. Open `http://localhost:8080/wealth` — Overview combined net worth; **Crypto** tab holdings and PnL (realized / unrealized / total return).
5. Create `allocation_target` plan on `/planning` (e.g. 50/50 template); confirm allocation gap card on `/wealth`.
6. Grafana dashboard **Portfolio** (`uid=portfolio`) — crypto stat, allocation pie, total return series.
7. Optional: `DATABASE_URL=... cargo test --test exchanges_portfolio_integration` for migration-007 persistence proof.
8. AI chat `get_portfolio` includes crypto totals and top holdings when exchanges connected.

### AI assistant verification (US-0006)

After US-0001–US-0005 surfaces are operational, migration `006_ai_audit.sql` is applied, and `OPENAI_API_KEY` is set:

1. Open `http://localhost:8080/chat` or header **AI** Sheet drawer — submit a natural-language question; confirm SSE stream and **Tools used** transparency row.
2. Confirm Settings **AI & Privacy** shows OpenAI configured badge and read-only `[privacy]` defaults (`allow_raw_transactions=false` by default).
3. Use suggested prompts (affordability, subscription prices, budget overrun, cancel savings, top categories) — verify tool-backed answers when mirror + active plan exist.
4. Open Settings tool audit table — confirm redacted invocation rows after chat (`GET /api/v1/ai/audit`).
5. With `allow_raw_transactions=false`, confirm aggregate-only transaction tool output and privacy badge in chat header.
6. Optional: `DATABASE_URL=... cargo test --test ai_assistant_integration` for audit persistence + static tools-only boundary proof.

### Local AI provider (US-0008)

Operators can run OpenAI cloud (default), Ollama in Compose **full** profile, or any OpenAI-compatible endpoint (LM Studio, LocalAI, vLLM) via `[ai] base_url`.

**Compose full + Ollama**

```bash
docker compose --profile full up -d
docker compose --profile full exec ollama ollama pull qwen2.5:14b
```

`backend/config/default.toml`:

```toml
[ai]
provider = "ollama"
model = "qwen2.5:14b"
# base_url defaults to http://ollama:11434/v1 inside Compose network
```

**Recommended Ollama tags (VRAM approximate)**

| Tag | Use | VRAM |
|-----|-----|------|
| `llama3.1:8b` | Dev / fast | ~5.5 GB |
| `qwen2.5:14b` | Prod default | ~9.5 GB |
| `qwen2.5:7b` | Minimum GPU | ~5 GB |

**LM Studio on host (Docker backend)**

```toml
provider = "openai_compatible"
base_url = "http://host.docker.internal:1234/v1"
model = "local-model"
```

Ensure `extra_hosts: host.docker.internal:host-gateway` on the backend service (Linux).

**vLLM** — enable tool calling on the server, e.g. `--enable-auto-tool-choice --tool-call-parser llama3_json`.

**Verification**

1. Settings **AI & Privacy** — provider table shows label, model, base URL, configured badge.
2. **Test AI provider** — `POST /api/v1/ai/test` returns `ok: true` with latency and sample (no audit row).
3. Chat header shows **Local · Ollama** or **Cloud · OpenAI** badge beside privacy badge.
4. AC5 CI: `cargo test --test ai_local_provider_isolation` (wiremock; no GPU).
5. Operator UAT: full profile chat without `OPENAI_API_KEY`; confirm no outbound `api.openai.com` traffic.

Restart backend after any `[ai]` change — no in-app provider switch or model pull.

### ML forecasting verification (US-0009)

After US-0001–US-0008 surfaces are operational, migration `009_forecast_ml.sql` is applied, and optional ML overlay is enabled:

1. **Full profile + sidecar:** `docker compose --profile full up -d`; verify `curl http://localhost:8090/health` → `{"status":"ok"}`.
2. Enable ML in `backend/config/default.toml` (`[forecast_ml] enabled = true`) or `FORECAST_ML_ENABLED=true`; **restart backend**.
3. Run sync with ≥12 mo Firefly history (≥24 mo for MSTL seasonal callout on Monthly tab).
4. Open `http://localhost:8080/forecast` — Long-term **Baseline | ML | Compare** at 6/12/24 mo; confirm p10–p90 bands in ML mode and dual series in Compare.
5. Monthly tab — seasonal callout when `/api/v1/forecast/meta` reports `seasonal_detected=true`.
6. Planning — risk badge 0–100 with component tooltip on active plan; Compare tab risk column.
7. Wealth → Crypto — 3/6/12 mo projected EUR when US-0007 exchanges connected (≥8 weekly snapshots).
8. Grafana Dashboard 5 (`uid=forecast-horizons`) — switch `$forecast_variant` Baseline / ML Enhanced; confirm Confidence band, Seasonal detected, Portfolio 3/6/12 mo EUR, Active plan risk score panels.
9. Confirm alerts and plan hooks still use `model_kind=baseline` only (DEC-0007 authority preserved).
10. Optional: `DATABASE_URL=... cargo test --test forecast_ml_integration` for DB skip-metadata persistence proof.

**Default:** `[forecast_ml] enabled = false` — minimal/standard profile serves baseline-only without sidecar.

### Wealth and alerts verification (US-0005)

After sync with asset accounts, successful forecast recompute, and active plan with category adjustments:

1. Open `http://localhost:8080/wealth` — net worth stat, account breakdown, mixed-currency banner when applicable, wealth-over-time chart.
2. Run **Sync now** — confirm post-sync `"alerts"` phase; verify scarcity, budget drift, or plan viability alerts when conditions breach thresholds.
3. Open `http://localhost:8080/alerts` and header notification bell — acknowledge/dismiss active alerts; confirm unread badge updates.
4. Grafana dashboards: **Portfolio** (`uid=portfolio`); **Cashflow** (`uid=cashflow`) — confirm `$scarcity_threshold` tracks `alert_config`.
5. Adjust thresholds in `backend/config/default.toml` `[alerts]`; restart backend to mirror into `alert_config`.
6. Optional: `DATABASE_URL=... cargo test --test wealth_alerts_integration` for snapshot upsert + post-sync scarcity alert proof.
