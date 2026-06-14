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

### README maintenance (US-0016)

Living-doc updates for root `README.md` **`### Product status`** bind to phase boundaries
only — not per-commit. Normative contract: **DEC-0070** and `docs/engineering/architecture.md`
§ **US-0016** / § **US-0017** (per-segment hooks).

#### Release segment (definition)

The **release segment** is the scope of work reconciled in the current `/release` or
`/refresh-context` pass:

- a standard sprint id (`Sxxxx`),
- a quick task id (`Qxxxx`), or
- a paired intake batch (multiple US/BUG ids closed together in one release artifact).

Each US or BUG that reaches **DONE** / **CLOSED** within that segment must receive exactly one
Product status bullet before the phase completes.

#### Validator commands (when to use each)

| Repo state | Command | Use when |
|------------|---------|----------|
| `template/` **absent** (current) | `python scripts/validate_doc_profile.py --repo . --no-template-parity` | CI, release gate, local edits before push |
| `template/` **present** | `python scripts/validate_doc_profile.py --repo .` | After full `template/README.md` + `template/docs/developer/README.md` mirror lands |

**Template flip gate:** drop `--no-template-parity` only in the **same change set** that adds
the complete `template/` tree. Do not ship a partial `template/README.md` stub.

#### Release (`/release`)

After backlog reconciliation (≈ step 10), before runbook readiness (≈ step 14):

1. For **each** **US** or **BUG** in the **current release segment** (see definition above) that
   transitions to **DONE** / **CLOSED**, append one bullet to root `README.md` **`### Product
   status`** under **`## Purpose`** in the form `{US-xxxx|BUG-xxxx} — {one-line outcome}`
   (newest first).
2. Trim to the **8** most recent entries; drop oldest.
3. Run `python scripts/validate_doc_profile.py --repo . --no-template-parity` — non-zero exit
   → fail closed; remediation → this subsection.

#### Refresh-context (`/refresh-context`)

After backlog status reconciliation:

1. When the **release segment** or sprint artifacts closed **one or more** US/BUG ids since the
   prior refresh, verify **each** closed id appears in root `README.md` **`### Product status`**;
   update missing bullets before completing refresh.
2. When README or doc-profile surfaces were touched, run
   `python scripts/validate_doc_profile.py --repo . --no-template-parity`.

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
**DEC-0075** + `docs/engineering/architecture.md` `# US-0090` +
`scripts/caveman_compress_input.py`.

Non-substitution with `TOKEN_PROFILE` and `CAVEMAN_MODE`:

`TOKEN_PROFILE` controls context breadth. `CAVEMAN_MODE` controls reply voice. `CAVEMAN_COMPRESS_INPUT` controls input-side file compression. All three axes are orthogonal: setting one does not change the others, and none substitutes for another.

#### Activation gate (DEC-0075 §2)

All three conditions must hold before any mutation occurs:

1. `CAVEMAN_COMPRESS_INPUT=1` in `.cursor/scratchpad.md`.
2. `CAVEMAN_FILE_SCOPE` non-empty.
3. CLI invoked with `--write`.

Any failing condition short-circuits with a reason code from §7 and exit `2`.
Default / unset / partial state = no-op.

#### Sidecar originals (DEC-0075 §3)

Before mutating any file, the script writes the pre-mutation bytes to
`docs/.caveman-originals/<relative/path>/<filename>`. Atomic order: sidecar
first (temp + replace), then target (temp + replace). The tree is anchored
by `docs/.caveman-originals/.gitkeep` and excluded from VCS by the repo-root
`.gitignore` anchor for US-0090.

#### Deny-list policy (DEC-0075 §4)

Layered, read in this order (**deny always wins**):

1. Hard-coded baseline in `scripts/caveman_compress_input.py` (`DENY_BASELINE`).
2. Merged secret-like patterns from repo-root `.gitignore` (`.env*`, `*secret*`,
   `*credential*`, `*token*`, `*private*`).
3. Optional `.cursorignore` overlay when
   `CAVEMAN_COMPRESS_INGEST_CURSORIGNORE=1` in scratchpad.

Deny-list baseline is versioned via `deny_list_version` (SHA-256 of sorted
canonical JSON) and reported by `--report`.

#### Allow-list grammar (DEC-0075 §5)

Three forms in `CAVEMAN_FILE_SCOPE`:

| Form | Example | Notes |
|------|---------|-------|
| Named profile | `docs-prose-only` | Frozen v1 table; new profiles require subsequent DEC. |
| Raw CSV globs | `docs/user-guides/**/*.md,handoffs/archive/*.md` | Forward slashes only. |
| Hybrid | `profile:docs-prose-only;globs:handoffs/archive/*.md` | One profile per scope; unknown tokens fail closed. |

#### Safe-mode minifier (DEC-0075 §6)

Four-step, strictly idempotent pipeline:

1. Collapse two-or-more consecutive blank lines into a single blank line
   (outside fenced code).
2. Trim trailing whitespace on non-fence lines.
3. Normalize `CRLF` / `CR` → `LF`.
4. Preserve the source file's EOF-newline status.

Aggressive mode is **deferred**; v1 ships safe-mode only. All safe-mode
transformations keep the 9 DEC-0072 §4 literal regions byte-identical; any
drift is fail-closed with `CAVEMAN_COMPRESS_LITERAL_REGION_DAMAGED`.

#### Reason-code vocabulary (DEC-0075 §7)

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

#### CLI contract (DEC-0075 §8)

| Flag | Semantics |
|------|-----------|
| `--dry-run` | (default) inventory + diff summary to stdout; no mutation. |
| `--write` | Perform sidecar + target mutation on eligible files (sidecar first). |
| `--verify-originals` | Walk sidecar tree; verify bidirectional presence; fail closed with `CAVEMAN_COMPRESS_ORIGINAL_MISSING` on orphan. |
| `--report` | Emit canonical JSON report on stdout (incompatible with `--write`). |

#### Scratchpad keys (US-0090 additions)

| Key | Values | Default | Notes |
|-----|--------|---------|-------|
| `CAVEMAN_COMPRESS_INPUT` | `0` \| `1` | `0` | Activation gate bit (DEC-0075 §2). |
| `CAVEMAN_FILE_SCOPE` | string | empty | Profile name, CSV globs, or hybrid (§5). |
| `CAVEMAN_COMPRESS_INGEST_CURSORIGNORE` | `0` \| `1` | `0` | Optional overlay (§4). |

#### Template parity (DEC-0075 §10)

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

<a id="backdated-firefly-imports"></a>

### Backdated Firefly imports (BUG-0025 / DEC-0002)

When bulk-importing or backdating transactions in Firefly III, the mirror may not
immediately reflect new months on **Category spending trend** or expense-series
charts — even though sync reports success.

| Topic | Detail |
|-------|--------|
| **Symptom** | Category trend / expense-series missing months after a Firefly backdated import (e.g. **Wohnen - Stromkosten** shows only the most recent month) |
| **Cause** | **DEC-0002** — Firefly `GET /api/v1/transactions?start=` filters by **transaction date**. Scheduled incremental sync uses `watermark − overlap_days` (default **7 days**). Rows dated before that window are not fetched. |
| **Fix ≤365d** | On **Sync Status** (`/sync`), click **Sync now** — manual Full sync uses a **365-day** lookback by transaction date (post BUG-0025 fix). |
| **Fix >365d** | Reset the transactions cursor, then run manual Full sync: `DELETE FROM sync_cursors WHERE entity_type = 'transactions';` then **Sync now**. The next Full sync uses the cold-start **365-day** path. |
| **Safety** | Mirror upserts by Firefly transaction `id` — cursor reset does **not** create duplicate rows. |

**Verify:** `GET /api/v1/categories/expense-series?category_id=<id>` should show
bars for each month present in Firefly after remediation.

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
# expect: flow-finance-ai, grafana, stats-forecast
```

See `scripts/compose-config-check.sh` (invoked by `bash tests/run-tests.sh`).

#### 7a. Omniflow ML enablement (US-0013 / DEC-0076)

**Release (S0014):** `0.14.0-us0013` — operator notes `handoffs/releases/S0014-release-notes.md` and `docs/user-guides/US-0013.md`. Enables StatsForecast sidecar on the external profile with opt-in ML overlay (DEC-0049 default-off preserved).

**Compose profile union:** Base `docker-compose.yml` defines `stats-forecast` with `profiles: [full]`. Overlay `docker-compose.external.yml` additively merges `profiles: [external]` and traefik network attachment — one container when either profile is active. Do **not** combine `external` with `minimal`, `standard`, `full`, or `bundled-firefly`.

```bash
docker compose -f docker-compose.yml -f docker-compose.external.yml \
  --profile external up -d --build flow-finance-ai stats-forecast
```

**Env (names only — set in operator `.env`):**

| Variable | Default | Purpose |
|----------|---------|---------|
| `FORECAST_ML_ENABLED` | `false` | Opt-in ML overlay; set `true` after sidecar healthy |
| `STATS_FORECAST_URL` | `http://stats-forecast:8090` | Internal sidecar URL (container port, not host remap) |
| `STATS_FORECAST_PORT` | `8091` | Host debug port for health probe (8090 clash on omniflow) |

After changing ML env vars, recreate `flow-finance-ai`:

```bash
docker compose -f docker-compose.yml -f docker-compose.external.yml \
  --profile external up -d --force-recreate flow-finance-ai
```

**Health probe (runtime gate):**

```bash
curl -sf "http://localhost:${STATS_FORECAST_PORT:-8091}/health"
```

Backend `health_ok()` GET `/health` runs before the sync `forecast_ml` phase (60s HTTP timeout). Compose healthcheck (`start_period: 30s`) is advisory only — first sync after cold start may skip ML if sidecar is still warming (DEC-0052 acceptable).

**Min history:** `[forecast_ml] min_monthly_points = 12` unchanged. Requires **Full Firefly sync** with ≥12 monthly net-cashflow points per asset account before ML overlay persists.

**Full sync + recompute steps:**

1. Deploy overlay + set `FORECAST_ML_ENABLED=true` in operator `.env`
2. Restart `stats-forecast` and `flow-finance-ai`
3. Confirm sidecar health (`curl` above or from traefik network: `curl -sf http://stats-forecast:8090/health`)
4. Trigger **Full sync** from Settings → Sync
5. Verify `GET /api/v1/forecast/meta` → `ml_status: success` or documented skip reason
6. Verify React `/forecast` Compare tab and Grafana `$forecast_variant=ml_enhanced`

**Degraded mode troubleshooting:**

| `ml_skipped_reason` | Meaning | Remediation |
|---------------------|---------|-------------|
| `sidecar_disabled` | `FORECAST_ML_ENABLED=false` or unset | Set `FORECAST_ML_ENABLED=true`; recreate backend |
| `sidecar_unavailable` | Health probe failed | Check `stats-forecast` logs; wait for healthcheck; re-sync |
| `insufficient_history` | &lt;12 monthly points | Run Full sync after more Firefly history accumulates |
| `sidecar_error` | Sidecar HTTP/forecast error | Check sidecar logs; verify mirror data |

**Cold start:** First sync after deploy may record `sidecar_unavailable` — re-sync after `/health` returns OK.

**Memory:** StatsForecast sidecar RSS ~200–400 MB under load (R-0044); monitor on shared omniflow host.

**Cross-link:** `docs/user-guides/US-0013.md`

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

#### 19. BUG-0008 hotfix — Subscription alerts & detection recall (Q0018 / released 2026-06-08)

**Release:** BUG-0008 **DONE** — operator notes `handoffs/releases/Q0018-release-notes.md`. Fixes omniflow subscription alert/list mismatch and under-detection on US-0010 external profile: **(W)** alert fingerprint dedup, reconciled unread-count API, orphan lifecycle, frontend banner/toast; **(X)** payee normalization, transfer counterparty guard, 730-day detection window without resync spam. Per **DEC-0071**, **DEC-0072 Phase 1**.

**Deploy (explicit — backend + embedded frontend):**

```bash
AUTHENTIK_SECRET_KEY=unused-external-profile docker compose \
  -f docker-compose.yml -f docker-compose.external.yml \
  --profile external up -d --build flow-finance-ai
```

**Operator gate — BACKEND_FRONTEND_DEPLOY (required before V1 smoke):**

```bash
docker compose -f docker-compose.yml -f docker-compose.external.yml \
  --profile external up -d --force-recreate flow-finance-ai
```

**Migration:** `010_subscription_alert_fingerprint.sql` runs on backend startup — confirm DB healthy after deploy.

**Env (names only — operator `.env`):**

| Variable | Purpose |
|----------|---------|
| `DATABASE_HOST` | **`postgres`** on external profile (prerequisite from §11) |
| `FIREFLY_BASE_URL` / `FIREFLY_PERSONAL_ACCESS_TOKEN` | Mirror data gate (922+ txs) |

**Operator smoke (post-deploy):**

| Step | Check | Pass |
|------|-------|------|
| Backend unit | `cd backend && cargo test --lib` | 156/156 |
| BUG-0008 suite | `cd backend && cargo test --test bug0008_subscription_alerts` | 8/8 |
| App health | `curl -sf https://financegnome.omniflow.cc/health` | HTTP 200 |
| Row W | `GET /api/v1/subscriptions/alerts/unread-count` | `reconciled: true`; count ≤ pending patterns |
| Row W | `/subscriptions` banner + toast | Count from `unread_new_detection`; toast on delta only |
| Row W | Confirm/reject pattern | Orphan alerts marked read |
| Row X | Pattern count after optional sync | > 12 patterns (recall improvement) |
| Row X | Post-resync spam guard | `unread_new_detection` not >> `pending_patterns` |
| Regression | Subscription routes + OIDC/external profile | **PASS** |

**Automated regression:**

```bash
cd backend && cargo test --lib
cd backend && cargo test --test bug0008_subscription_alerts
```

**Boundaries:** X Phase 2 category gate deferred; AI-in-pipeline deferred; US-0005 unified bell unchanged; BUG-0007 coordinate-only; OIDC/browser regression deferred to operator smoke.

#### 20. BUG-0011 hotfix — Planning mode fixes (Q0019 / released 2026-06-08)

**Release:** BUG-0011 **DONE** — operator notes `handoffs/releases/Q0019-release-notes.md`. Fixes omniflow planning mode on US-0010 external profile: **(AD)** first-run Create empty plan + inline add/edit adjustments; **(AE)** overlay-only compare `monthly_delta_sum` (zero adjustments → **0.00**); **(AF)** plan-vs-actual HTTP **200** `no_active_plan` guided UX. Per **DEC-0073**, **DEC-0074**.

**Deploy (explicit — backend + embedded frontend):**

```bash
AUTHENTIK_SECRET_KEY=unused-external-profile docker compose \
  -f docker-compose.yml -f docker-compose.external.yml \
  --profile external up -d --build flow-finance-ai
```

**Operator gate — BACKEND_FRONTEND_DEPLOY (required before V1 smoke):**

```bash
docker compose -f docker-compose.yml -f docker-compose.external.yml \
  --profile external up -d --force-recreate flow-finance-ai
```

**Env (names only — operator `.env`):**

| Variable | Purpose |
|----------|---------|
| `DATABASE_HOST` | **`postgres`** on external profile (prerequisite from §11) |
| `FIREFLY_BASE_URL` / `FIREFLY_PERSONAL_ACCESS_TOKEN` | Mirror data for forecast baseline |

**Operator smoke (post-deploy):**

| Step | Check | Pass |
|------|-------|------|
| Backend unit | `cd backend && cargo test --lib` | 160/160 |
| Plans integration | `cd backend && cargo test --test plans_integration` | 5/5 |
| App health | `curl -sf https://financegnome.omniflow.cc/health` | HTTP 200 |
| Row AD | `/planning` Create empty plan + add adjustment | Editable plan; POST/PATCH succeed |
| Row AD | Custom Apply toast | "Custom plan ready — add lines below" |
| Row AE | Compare — zero-adjustment plan | `monthly_delta_sum` ≈ **0.00** |
| Row AE | Compare — Leasing template | Overlay delta ~ **-300** |
| Row AF | `GET /api/v1/plans/active/plan-vs-actual` (no active) | HTTP **200** `{ status: "no_active_plan" }` |
| Row AF | Plan vs Actual tab (no active) | Guided card — not blank/404 |
| Regression | OIDC `/planning` three tabs | Scenarios + Compare + PVA load without console errors |

**Automated regression:**

```bash
cd backend && cargo test --lib
cd backend && cargo test --test plans_integration
```

**Boundaries:** Grafana Dashboard 3 unchanged (R-0020); BUG-0008 coordinate-only; no auto-activate on create (US-0004 preserved).

#### 21. US-0014 — Planning mode intuitive UX (S0015 / released 2026-06-08)

**Release:** US-0014 **DONE** — operator notes `handoffs/releases/S0015-release-notes.md`. Polishes planning mode on US-0010 external profile per **DEC-0077**: **(AC-1)** first-run template grid + Create empty plan; **(AC-2/AC-5)** success confirmations + add-lines invalidation; **(AC-3/AC-4)** Compare/PVA contextual UX verify; **(AC-6)** set-active banner Dashboard 3 copy; **(AC-7)** operator-visible mutation error cards; **(AC-8)** OIDC three-tab smoke template.

**Deploy (explicit — backend + embedded frontend):**

```bash
AUTHENTIK_SECRET_KEY=unused-external-profile docker compose \
  -f docker-compose.yml -f docker-compose.external.yml \
  --profile external up -d --build flow-finance-ai
```

**Operator gate — BACKEND_FRONTEND_DEPLOY (required before AC-8 OIDC smoke):**

```bash
docker compose -f docker-compose.yml -f docker-compose.external.yml \
  --profile external up -d --force-recreate flow-finance-ai
```

**Operator smoke (post-deploy):**

| Step | Check | Pass |
|------|-------|------|
| Frontend unit | `cd frontend && npm test` | 5/5 |
| Plans integration | `cd backend && cargo test --test plans_integration` | 5/5 |
| App health | `curl -sf https://financegnome.omniflow.cc/health` | HTTP 200 |
| AC-1 | Empty state template grid + Create empty plan | 6 templates; inline add form visible |
| AC-2/AC-5 | Create empty / from template | Green success confirmation; add line updates Compare/PVA |
| AC-3 | Compare — zero-adjustment plan | `monthly_delta_sum` ≈ **0.00**; overlay footnote |
| AC-4 | Plan vs Actual (no active) | Guided card — not blank/404 |
| AC-6 | Set-active banner | Mentions Plan vs Actual + Grafana Dashboard 3 (Budgets) |
| AC-7 | Force mutation failure | Red error card with Dismiss |
| AC-8 | OIDC `/planning` three tabs | Scenarios + Compare + PVA load without console errors |

**Automated regression:**

```bash
cd frontend && npm test
cd backend && cargo test --test plans_integration
```

**Boundaries:** DEC-0073/DEC-0074 compare/PVA contracts frozen; no auto-activate on create (US-0004 preserved); user guide `docs/user-guides/US-0014.md`.

#### 22. US-0015 — AI-assisted forecast category bucket mapping (S0016 / released 2026-06-06)

**Release:** US-0015 **DONE** — operator notes `handoffs/releases/S0016-release-notes.md`. AI bucket cascade on US-0010 external profile per **DEC-0078**: **(AC-1)** config precedence guard; **(AC-2)** rule→LLM→Variable cascade with 0.75 threshold; **(AC-3)** privacy allowlist `prepare_bucket_features`; **(AC-4)** monthly API `bucket_sources` + `ai_mapped`; **(AC-5)** ForecastPage AI-mapped badge; **(AC-6)** `forecast_bucket_assignment` audit; **(AC-7)** OIDC `/forecast` Monthly smoke template (pass-with-prerequisites at release).

**Deploy (explicit — backend + embedded frontend):**

```bash
AUTHENTIK_SECRET_KEY=unused-external-profile docker compose \
  -f docker-compose.yml -f docker-compose.external.yml \
  --profile external up -d --build flow-finance-ai
```

**Operator gate — BACKEND_FRONTEND_DEPLOY (required before AC-7 OIDC smoke):**

```bash
docker compose -f docker-compose.yml -f docker-compose.external.yml \
  --profile external up -d --force-recreate flow-finance-ai
```

After deploy: Full Firefly sync + forecast recompute before Monthly tab smoke.

**Operator smoke (post-deploy):**

| Step | Check | Pass |
|------|-------|------|
| Backend unit | `cd backend && cargo test --lib` | 169/169 |
| Frontend unit | `cd frontend && npm test` | 5/5 |
| App health | `curl -sf https://financegnome.omniflow.cc/health` | HTTP 200 |
| Sync + recompute | Full Firefly sync + forecast recompute | `GET /api/v1/forecast/meta` shows `computation_id` |
| AC-4 | `GET /api/v1/forecast/monthly` | `bucket_sources` + `ai_mapped` when provenance present |
| AC-5 | `/forecast` Monthly tab | AI-mapped badge when `ai_mapped=true`; no badge on config-only months |
| AC-7 | OIDC `/forecast` Monthly smoke | Steps 1–8 per `sprints/S0016/uat.md` |
| Regression | AI Chat six tools | Unchanged (BUG-0007) |
| Regression | Forecast Compare / ML tabs | US-0013 ML overlay unchanged |

**Automated regression:**

```bash
cd backend && cargo test --lib
cd frontend && npm test
```

**Boundaries:** Config-mapped buckets never AI-overridden; rolling residual → Variable only (MVP); chat tool registry unchanged; user guide `docs/user-guides/US-0015.md`.

#### 23. BUG-0013 hotfix — Omniflow analytics regression (Q0020 / released 2026-06-09)

**Release:** BUG-0013 **DONE** — operator notes `handoffs/releases/Q0020-release-notes.md`. Fixes omniflow analytics on US-0010 external profile per **DEC-0079** and **DEC-0080**: **(AL)** budgets MTD planned `<= CURRENT_DATE` cap + footnote; **(AN/AK)** Bitunix futures wallet array parse + linear unrealized USDT→EUR; **(AJ)** subscriptions price-changes empty-state copy; **(AK2)** portfolio performance % min-snapshot footnote; **(AM)** waived per R-0077; **(AI)** ops regression smoke after Full sync.

**Deploy (backend + Grafana provisioning):**

```bash
AUTHENTIK_SECRET_KEY=unused-external-profile docker compose \
  -f docker-compose.yml -f docker-compose.external.yml \
  --profile external up -d --build flow-finance-ai grafana
```

**Operator gate — BACKEND_FRONTEND_DEPLOY (required before AN/AK wealth probes):**

```bash
docker compose -f docker-compose.yml -f docker-compose.external.yml \
  --profile external up -d --force-recreate flow-finance-ai
```

**Operator gate — GRAFANA_PROVISIONING_RELOAD (required before AL/AJ/AK live panels):**

```bash
docker compose -f docker-compose.yml -f docker-compose.external.yml \
  --profile external up -d --force-recreate grafana
```

**Operator gate — FULL_FIREFLY_SYNC:** Settings → Sync → **Full sync** (not exchanges-only) + forecast recompute before AI/AN/AK wealth probes.

**Grafana UI warning:** Do **not** click **Save** on analytics dashboards after variable changes — persisted `current` blocks override provisioning JSON (see §17).

**Operator smoke (post-deploy + all three gates):**

| Step | Check | Pass |
|------|-------|------|
| Backend unit | `cd backend && cargo test --lib` | 174/174 |
| App health | `curl -sf https://financegnome.omniflow.cc/health` | HTTP 200 |
| Row AL | `/analytics/budgets` MTD summary | Planned/actual/deviation plausible — not −€150K artifact |
| Row AN | `GET /api/v1/wealth` | `crypto.subtotal_eur` > 0 when Bitunix equity > 0 |
| Row AK | `/analytics/portfolio` crypto stat | Non-zero after sync + recompute |
| Row AK | Performance % panel | "Needs ≥2 snapshots" or data when history exists |
| Row AJ | `/analytics/subscriptions` price changes | Event rows or documented empty-state |
| Row AI | `/analytics/cashflow` + `/analytics/forecast-horizons` acct 114 | Non-empty signed baseline balances |
| Regression | Six `/analytics/{slug}` routes + `POST …/ds/query` | **200** embed without transport errors |

**Automated regression:**

```bash
cd backend && cargo test --lib
```

**Boundaries:** US-0015 AI buckets unchanged; BUG-0011 planning unchanged; DEC-0064 crypto subtotal rules preserved; MetaMask extension console noise out of scope.

#### 24. BUG-0014 hotfix — Post-rebuild omniflow cluster (Q0022 / released 2026-06-07)

**Release:** BUG-0014 **DONE** — operator notes `handoffs/releases/Q0022-release-notes.md`. Fixes post-rebuild omniflow cluster on US-0010 external profile per **DEC-0081**, **DEC-0082**, **DEC-0083**: **(AO)** forecast-horizons panel 13 dual-scenario ML copy; **(AQ)** wealth holdings cap 50 + unified `fx_incomplete` + native qty/EUR table; **(AS)** plan delete UI + active 409 guard + five `target_type` options; **(AP)** AP2 conditional skipped pending AP1_SQL_PROBE; **(AR)** AR1 conditional skipped pending AR partial probe; **(AT)** ops-only three-service compose.

**Deploy (backend + Grafana + stats-forecast when ML enabled):**

```bash
AUTHENTIK_SECRET_KEY=unused-external-profile docker compose \
  -f docker-compose.yml -f docker-compose.external.yml \
  --profile external up -d --build flow-finance-ai grafana stats-forecast
```

**Operator gate — BACKEND_FRONTEND_DEPLOY (required before wealth/planning probes):**

```bash
docker compose -f docker-compose.yml -f docker-compose.external.yml \
  --profile external up -d --force-recreate flow-finance-ai
```

**Operator gate — THREE_SERVICE_COMPOSE (required before AO-1 / AT-1):**

```bash
docker compose -f docker-compose.yml -f docker-compose.external.yml \
  --profile external up -d stats-forecast
```

**Operator gate — GRAFANA_PROVISIONING_RELOAD (required before AO live panel):**

```bash
docker compose -f docker-compose.yml -f docker-compose.external.yml \
  --profile external up -d --force-recreate grafana
```

**Operator gate — FULL_FIREFLY_SYNC:** Settings → Sync → **Full sync** (not exchanges-only) + forecast recompute acct 114 before AR probes.

**Operator gate — AP1_SQL_PROBE:** Run on `exchange_holdings` before AP2 evaluation:

```sql
SELECT product_type, asset, quantity, market_value_eur, unrealized_pnl_eur
FROM exchange_holdings WHERE exchange_id='bitunix'
ORDER BY product_type, asset;
```

**Grafana UI warning:** Do **not** click **Save** on analytics dashboards after variable changes — persisted `current` blocks override provisioning JSON (see §17).

**Operator smoke (post-deploy + all gates):**

| Step | Check | Pass |
|------|-------|------|
| Backend unit | `cd backend && cargo test --lib` | 177/177 |
| App health | `curl -sf https://financegnome.omniflow.cc/health` | HTTP 200 |
| Row AO | `/analytics/forecast-horizons` panel 13 | Dual-scenario ML copy |
| Row AO-1 | `GET /api/v1/forecast/meta` | `ml_computation_id` set or sidecar-down copy |
| Row AP | AP1 SQL + `GET /api/v1/wealth` | `crypto.subtotal_eur` > 0 when priced |
| Row AQ | Wealth crypto tab | Native qty + EUR + FX banner when applicable |
| Row AR | API acct 114 + `/analytics/cashflow` | Non-zero balances; reopen AR1 if API≠Grafana |
| Row AS | `/planning` | Delete non-active; 409 active; five target_type options |
| Row AT | `docker ps \| grep stats-forecast` | Container running when ML enabled |
| Regression | Six `/analytics/{slug}` routes | Embed without transport errors |

**Automated regression:**

```bash
cd backend && cargo test --lib
cd backend && cargo test --test grafana_provisioning_bug0009
cd frontend && npm test -- --run
```

**Boundaries:** BUG-0013 DEC-0080 Bitunix pricing unchanged; US-0014 planning semantics preserved; DEC-0064 crypto subtotal rules preserved.

#### 25. BUG-0015 hotfix — Subscription confirm persistence after rebuild (Q0023 / released 2026-06-07)

**Release:** BUG-0015 **DONE** — operator notes `handoffs/releases/Q0023-release-notes.md`. Fixes confirm-once trust on US-0010 external profile per **DEC-0084**, **DEC-0085**, **DEC-0086**: **(AU)** card billing `payee_key` normalization + payee+interval merge before pending upsert; **(AV)** detection skip/merge by payee+interval + stale inactive; **(AW)** merge path suppresses spurious `new_detection` alerts.

**Deploy (backend only — postgres volume untouched):**

```bash
AUTHENTIK_SECRET_KEY=unused-external-profile docker compose \
  -f docker-compose.yml -f docker-compose.external.yml \
  --profile external up -d --build flow-finance-ai
```

**Operator gate — BACKEND_FRONTEND_DEPLOY (required before runtime probes):**

```bash
docker compose -f docker-compose.yml -f docker-compose.external.yml \
  --profile external up -d --force-recreate flow-finance-ai
```

**Operator gate — POSTGRES_PERSISTENCE_PROBE (H2 SQL after app rebuild, before Full sync):**

```sql
SELECT status, COUNT(*) FROM subscription_patterns GROUP BY status;
SELECT fingerprint, status, payee_key, interval_days, current_amount
FROM subscription_patterns
WHERE payee_key ILIKE '%cursor%' OR payee_key ILIKE '%apple%'
ORDER BY updated_at DESC;
```

| Outcome | Action |
|---------|--------|
| Zero `confirmed` rows after rebuild (no operator action) | **Ops** — volume/DB target (H2); do not run V1 until resolved |
| `confirmed` rows present; drift after Full sync | AU1–AU4 path validated |
| Single confirmed per merchant; still pending in UI | Reopen discovery — unlikely per H3 refutation |

**Operator gate — FULL_FIREFLY_SYNC:** Settings → Sync → **Full sync** (not exchanges-only) + subscription detection phase.

**Rebuild scope:** Recreate `flow-finance-ai` only — do not recreate postgres.

**Operator smoke (post-deploy + all gates):**

| Step | Check | Pass |
|------|-------|------|
| Backend unit | `cd backend && cargo test --lib` | 187/187 |
| App health | `curl -sf https://financegnome.omniflow.cc/health` | HTTP 200 |
| AU baseline | Confirm Cursor + Apple on `/subscriptions` before rebuild | Confirmed in UI |
| H2-1 | H2 SQL after rebuild, before Full sync | `confirmed` rows present |
| Row AU | `GET /api/v1/subscriptions?status=confirmed` + `/subscriptions` UI | Cursor/Apple confirmed; no Confirm/Reject for confirmed merchants |
| Row AV | API + optional SQL | No duplicate `status=pending` for same payee+interval |
| Row AW | `GET /api/v1/subscriptions/alerts/unread-count` + pending tab | Unread reconciles; no spurious `new_detection` |
| OIDC-1 | OIDC-enabled deploy regression | Auth + `/subscriptions` load without transport errors |

**Automated regression:**

```bash
cd backend && cargo test --lib
cd backend && cargo test --lib card_billing
cd backend && cargo test --lib interval_matches
cd frontend && npm test -- --run
```

**Boundaries:** BUG-0008 alert dedup unchanged; US-0003 confirm/reject UX preserved; DEC-0072 SEPA paths unchanged for non-card payees.

#### 26. US-0018 — Category filters & expense trend analytics (S0017 / released 2026-06-09)

**Release:** US-0018 **DONE** — operator notes `handoffs/releases/S0017-release-notes.md`. Category analytics on US-0010 external profile per **DEC-0087**, **DEC-0088**, **DEC-0089**, **DEC-0090**: **(AC-1)** shared `CategoryFilter` on Forecast Monthly, Planning Compare, Wealth Overview + Grafana `$category` on cashflow/budgets; **(AC-2)** `GET /api/v1/categories/expense-series` month-spine (12 default / 24 max); **(AC-3)** `CategoryTrendChart` bar chart + empty-state; **(AC-4)** server `summary` MoM/best/worst; **(AC-5)** `__uncategorized__` sentinel; **(AC-6)** OIDC smoke template (pass-with-prerequisites at release).

**Deploy (backend + frontend + Grafana provisioning):**

```bash
AUTHENTIK_SECRET_KEY=unused-external-profile docker compose \
  -f docker-compose.yml -f docker-compose.external.yml \
  --profile external up -d --build flow-finance-ai grafana
```

**Operator gate — BACKEND_FRONTEND_DEPLOY (required before AC-6 OIDC smoke):**

```bash
docker compose -f docker-compose.yml -f docker-compose.external.yml \
  --profile external up -d --force-recreate flow-finance-ai
```

**Operator gate — FULL_FIREFLY_SYNC:** Settings → Sync → **Full sync** (not exchanges-only) so `category_id` mirror is current.

**Operator gate — GRAFANA_PROVISIONING_RELOAD (required before AC-1 Grafana `$category` smoke):**

```bash
docker compose -f docker-compose.yml -f docker-compose.external.yml \
  --profile external up -d --force-recreate grafana
```

**Grafana UI warning:** Do **not** click **Save** on analytics dashboards after variable changes — persisted `current` blocks override provisioning JSON (see §17).

**Operator smoke (post-deploy + all three gates):**

| Step | Check | Pass |
|------|-------|------|
| Backend unit | `cd backend && cargo test --lib` | 193/193 |
| Frontend unit | `cd frontend && npm test -- --run` | 7/7 |
| App health | `curl -sf https://financegnome.omniflow.cc/health` | HTTP 200 |
| AC-1 | `/forecast` Monthly `CategoryFilter` + chart | Filter visible; chart loads; household cards unchanged |
| AC-1 | `/planning` Compare category widget | Filter + Actual spending trend; compare table unchanged |
| AC-1 | `/wealth` Overview category subsection | Category spending subsection renders |
| AC-1 | `/analytics/cashflow` + `/analytics/budgets` | `$category` variable filters panels |
| AC-2 | `GET /api/v1/categories/expense-series` | Per-month EUR spine; 12 default / 24 max |
| AC-5 | `category_id=__uncategorized__` | `uncategorized: true`; full spine with €0 months |
| AC-6 | OIDC 10-step checklist | `sprints/S0017/uat.md` § OIDC smoke checklist |
| Regression | US-0015 AI-mapped badge | Unchanged on `/forecast` Monthly |
| Regression | Read-only Firefly | No POST/PUT/PATCH/DELETE to Firefly during smoke |

**Automated regression:**

```bash
cd backend && cargo test --lib
cd frontend && npm test -- --run
```

**Boundaries:** US-0015 bucket mapping unchanged; forecast monthly API shape unchanged when category filter is UI-only (DEC-0089); T-0185 EXPLAIN probe deferred per DEC-0090; user guide `docs/user-guides/US-0018.md`.

#### 27. US-0019 — Goal-driven planning with per-plan stats & category savings (S0018 / released 2026-06-09)

**Release:** US-0019 **DONE** — operator notes `handoffs/releases/S0018-release-notes.md`. Goal planning on US-0010 external profile per **DEC-0091**, **DEC-0092**, **DEC-0093**, **DEC-0094**, **DEC-0095**, **DEC-0096**, **DEC-0097**: **(AC-1)** `goal_balance` template with target balance + date; **(AC-2)** `GET /api/v1/plans/{id}/goal-stats` + `GoalStatsStrip` per-plan (not household on detail); **(AC-3)** category `remove_outflow` cap via 3-month expense-series average + goal account fork; **(AC-4)** savings modal with checkbox apply only; **(AC-5)** aggregate-only savings path + optional `get_category_savings` AI tool; **(AC-6)** US-0014 template regression preserved; OIDC smoke template (pass-with-prerequisites at release).

**Deploy (backend + frontend):**

```bash
AUTHENTIK_SECRET_KEY=unused-external-profile docker compose \
  -f docker-compose.yml -f docker-compose.external.yml \
  --profile external up -d --build flow-finance-ai
```

**Operator gate — BACKEND_FRONTEND_DEPLOY (required before AC-6 OIDC smoke):**

```bash
docker compose -f docker-compose.yml -f docker-compose.external.yml \
  --profile external up -d --force-recreate flow-finance-ai
```

**Operator gate — FULL_FIREFLY_SYNC:** Settings → Sync → **Full sync** (not exchanges-only) so `category_id` mirror and aggregates are current for overlay cap + savings ranking.

**Operator smoke (post-deploy + both gates):**

| Step | Check | Pass |
|------|-------|------|
| Backend unit | `cd backend && cargo test --lib` | 204/204 |
| Frontend unit | `cd frontend && npm test -- --run` | 9/9 |
| App health | `curl -sf https://financegnome.omniflow.cc/health` | HTTP 200 |
| AC-1 | `/planning` Scenarios Goal balance template | Create plan with target + date; appears in list |
| AC-2 | Goal stats strip on Scenarios + Compare | Monthly delta, yearly rollup, projected at target |
| AC-3 | Category `remove_outflow` adjustment | Recompute; Compare/PVA reflect capped change |
| AC-4 | Savings modal | Ranked categories; checkbox apply; no auto-apply |
| AC-5 | Savings privacy | Aggregate-only API; audit log on apply |
| AC-6 | OIDC 9-step checklist | `sprints/S0018/uat.md` § OIDC smoke checklist |
| Regression | US-0014 template grid + PVA guided card | Unchanged |
| Regression | DEC-0089 compare CategoryTrendChart | Actuals-only; compare API unchanged |
| Regression | Read-only Firefly | No POST/PUT/PATCH/DELETE to Firefly during smoke |

**Automated regression:**

```bash
cd backend && cargo test --lib
cd frontend && npm test -- --run
```

**Boundaries:** PVA tab unchanged (household active plan per DEC-0096); US-0018 category filter contract unchanged; user guide `docs/user-guides/US-0019.md`.

#### 28. US-0020 — Subscription manual discovery, majority category & operator tags (S0019 / released 2026-06-10)

**Release:** US-0020 **DONE** — operator notes `handoffs/releases/S0019-release-notes.md`. Subscription discover on US-0010 external profile per **DEC-0098**, **DEC-0099**, **DEC-0100**, **DEC-0101**, **DEC-0102**, **DEC-0103**: **(AC-1)** `GET /api/v1/subscriptions/discover` + Discover tab (account + payee + interval; cap 50); **(AC-2)** `POST …/discover/confirm` direct confirmed insert with DEC-0085 merge; **(AC-3)** RANK majority `display_category_id` + badge/tooltip tie-break; **(AC-4)** operator tag CRUD/assign/filter; **(AC-5)** product DB storage only — no Firefly write-back; **(AC-6)** US-0003/US-0008 regression preserved; OIDC smoke template (pass-with-prerequisites at release). **Last story in intake bundle.**

**Deploy (backend + frontend):**

```bash
AUTHENTIK_SECRET_KEY=unused-external-profile docker compose \
  -f docker-compose.yml -f docker-compose.external.yml \
  --profile external up -d --build flow-finance-ai
```

**Operator gate — BACKEND_FRONTEND_DEPLOY (required before AC-6 OIDC smoke):**

```bash
docker compose -f docker-compose.yml -f docker-compose.external.yml \
  --profile external up -d --force-recreate flow-finance-ai
```

**Operator gate — FULL_FIREFLY_SYNC:** Settings → Sync → **Full sync** (not exchanges-only) so mirror transactions and `category_id` are current for discover search + majority category.

**Operator smoke (post-deploy + both gates):**

| Step | Check | Pass |
|------|-------|------|
| Backend unit | `cd backend && cargo test --lib` | 213/213 |
| Frontend unit | `cd frontend && npm test -- --run` | 9/9 |
| App health | `curl -sf https://financegnome.omniflow.cc/health` | HTTP 200 |
| AC-1 | `/subscriptions` Discover tab | Search by account, payee, interval; results capped at 50 |
| AC-2 | Confirm discover candidate | Appears in All/confirmed list; DEC-0085 merge on duplicate |
| AC-3 | Majority category badge | Display category + RANK tie-break tooltip |
| AC-4 | Tag manager + filter | CRUD tags; multi-assign; filter All tab by slug |
| AC-5 | Storage contract | Tags + display_category in app DB only |
| AC-6 | OIDC 8-step checklist | `sprints/S0019/uat.md` § OIDC smoke checklist |
| Regression | US-0003 pending confirm/reject | Unchanged |
| Regression | US-0008 alert dedup | Manual confirm does not emit `new_detection` |
| Regression | Read-only Firefly | No POST/PUT/PATCH/DELETE to Firefly during smoke |

**Automated regression:**

```bash
cd backend && cargo test --lib
cd frontend && npm test -- --run
```

**Boundaries:** US-0003 auto-detection pipeline unchanged; DEC-0084..0086 confirm normalization preserved; optional Grafana `$tag` on subscriptions dashboard (DEC-0103 P2); user guide `docs/user-guides/US-0020.md`.

#### 29. BUG-0016 hotfix — SPA deep links HTTP 404 (Q0024 / released 2026-06-09)

**Release:** BUG-0016 **DONE** — operator notes `handoffs/releases/Q0024-release-notes.md`. SPA deep-link fallback on localhost and US-0010 external profile per **DEC-0104**, **DEC-0057**: **(AX1)** `ServeDir::fallback(ServeFile::new(index.html))` in `build_router` returns HTTP **200** HTML shell for client routes; **(AX2)** integration tests prove protected prefixes (`/health`, `/api/v1/*`, `/analytics/grafana/*`, `/assets/*`) are not replaced by SPA HTML.

**Deploy (backend serves built SPA + fallback):**

```bash
AUTHENTIK_SECRET_KEY=unused-external-profile docker compose \
  -f docker-compose.yml -f docker-compose.external.yml \
  --profile external up -d --build flow-finance-ai
```

**Operator gate — BACKEND_FRONTEND_DEPLOY (required before runtime probes):**

```bash
docker compose -f docker-compose.yml -f docker-compose.external.yml \
  --profile external up -d --force-recreate flow-finance-ai
```

Local override (`:18080`):

```bash
docker compose up -d --build flow-finance-ai
```

**Operator smoke (post-deploy):**

| Step | Check | Pass |
|------|-------|------|
| Backend unit | `cd backend && cargo test --lib` | 213/213 |
| SPA integration | `cd backend && cargo test --test spa_fallback_integration` | 5/5 |
| Frontend unit | `cd frontend && npm test -- --run` | 9/9 |
| AX-CURL-1 | `curl -sS -o /dev/null -w '%{http_code}' http://localhost:18080/forecast` | HTTP 200 |
| AX-CURL-2 | curl matrix `/subscriptions`, `/planning`, `/sync`, `/analytics/cashflow`, `/callback` | HTTP 200 + HTML |
| AX-CURL-3 | `/health` JSON; `/api/v1/nonexistent` JSON 404; `/assets/*` static when present | Non-HTML protected paths |
| AX-BROWSER-1 | Hard-refresh client routes on `financegnome.omniflow.cc` | Correct React page (not blank 404) |
| AX-BROWSER-2 | Bookmark reopen client routes | Correct React page |
| OIDC-1 | Complete OIDC login; `/callback` SPA shell | Session established |
| Regression | DEC-0057 Grafana proxy | `/analytics/grafana/*` not SPA HTML |

**Automated regression:**

```bash
cd backend && cargo test --lib
cd backend && cargo test --test spa_fallback_integration
cd frontend && npm test -- --run
```

**Boundaries:** No Traefik label changes; no backend `/callback` redirect; OIDC flow unchanged; supersedes BUG-0009 analytics 404 advisory.

#### 30. BUG-0017 hotfix — post-sync forecast recompute cluster (Q0025 / released 2026-06-10)

**Release:** BUG-0017 **DONE** — operator notes `handoffs/releases/Q0025-release-notes.md`. Post-sync forecast recompute cluster per **DEC-0105**, **DEC-0106**: **(AY1)** audit CHECK migration for `forecast_bucket_assignment` + extended `result_status`; **(BA1)** `ON DELETE CASCADE` on `paired_baseline_id`; **(BA2)** ml_enhanced-first retention order in `repository.rs`; **(BD1)** ForecastPage `isFetched` loading/empty guard.

**Deploy (backend migrations + forecast retention + frontend guard):**

```bash
AUTHENTIK_SECRET_KEY=unused-external-profile docker compose \
  -f docker-compose.yml -f docker-compose.external.yml \
  --profile external up -d --build flow-finance-ai
```

**Operator gate — BACKEND_FRONTEND_DEPLOY (required before runtime probes):**

```bash
docker compose -f docker-compose.yml -f docker-compose.external.yml \
  --profile external up -d --force-recreate flow-finance-ai
```

Local override (`:18080`):

```bash
docker compose up -d --build flow-finance-ai
```

**Operator gate — FULL_FIREFLY_SYNC (required before audit/meta/planning probes):**

```bash
curl -X POST http://localhost:18080/api/v1/sync/trigger
# Wait for sync status success; confirm forecast recompute completes
```

**Operator smoke (post-deploy):**

| Step | Check | Pass |
|------|-------|------|
| Backend unit | `cd backend && cargo test --lib` | 213/213 |
| Forecast integration | `cd backend && cargo test --test forecast_integration` | 3/3 |
| Frontend unit | `cd frontend && npm test -- --run` | 9/9 |
| V1-SYNC | `POST /api/v1/sync/trigger`; inspect logs | No audit CHECK WARN; no FK WARN |
| V1-META | `GET /api/v1/forecast/meta` | Fresh `computation_id`, `stale=false` |
| V1-AUDIT | `SELECT * FROM ai_tool_audit WHERE tool_name='forecast_bucket_assignment' LIMIT 5` | Rows present after recompute |
| V1-BB | Month-bucket SQL per R-0087; ML meta | Honest `ml_skipped_reason` when gate fails |
| V1-BC | Planning Compare after recompute | **Plan stale** badge clears |
| V1-BD | Forecast nav from Home | Loading skeleton; no false empty when meta has data |
| OIDC-1 | OIDC regression smoke | Standard OIDC checks pass |

**Automated regression:**

```bash
cd backend && cargo test --lib
cd backend && cargo test --test forecast_integration
cd frontend && npm test -- --run
```

**Boundaries:** Sync success semantics unchanged; true `insufficient_history` ML gate preserved; SPA fallback (BUG-0016) unchanged; BB month-bucket SQL deferred to operator per R-0087.

#### 31. BUG-0018 hotfix — alert evaluation SQL qualification (Q0026 / released 2026-06-10)

**Release:** BUG-0018 **DONE** — operator notes `handoffs/releases/Q0026-release-notes.md`. Post-sync wealth alert evaluation per **DEC-0107**: **(BE1)** qualify `fbd.balance` and `fbd.ts` in `evaluate_scarcity` daily aggregate query; **(T1)** `wealth_alerts_integration` scarcity regression gate.

**Deploy (backend alert SQL fix only — no migration):**

```bash
AUTHENTIK_SECRET_KEY=unused-external-profile docker compose \
  -f docker-compose.yml -f docker-compose.external.yml \
  --profile external up -d --build flow-finance-ai
```

**Operator gate — BACKEND_FRONTEND_DEPLOY (required before runtime probes):**

```bash
docker compose -f docker-compose.yml -f docker-compose.external.yml \
  --profile external up -d --force-recreate flow-finance-ai
```

Local override (`:18080`):

```bash
docker compose up -d --build flow-finance-ai
```

**Operator gate — FULL_FIREFLY_SYNC (required before alerts API / header bell probes):**

```bash
curl -X POST http://localhost:18080/api/v1/sync/trigger
# Wait for sync status success; confirm alert evaluation phase completes without 42702
```

**Operator smoke (post-deploy):**

| Step | Check | Pass |
|------|-------|------|
| Backend unit | `cd backend && cargo test --lib` | 213/213 |
| Alerts integration | `cd backend && cargo test --test wealth_alerts_integration` | 3/3 |
| Frontend unit | `cd frontend && npm test -- --run` | 9/9 |
| V1-SYNC | `POST /api/v1/sync/trigger`; inspect logs | No `alert evaluation failed` / 42702 |
| V1-ALERTS | `GET /api/v1/alerts?status=active` | Rows when scarcity rule matches |
| V1-BELL | Header Alerts bell | Non-empty active preview when rules match |
| V1-SUB-REG | `GET /api/v1/subscriptions/alerts` | Dedup regression per BUG-0008 |
| OIDC-1 | OIDC regression smoke | Standard OIDC checks pass |

**Automated regression:**

```bash
cd backend && cargo test --lib
cd backend && cargo test --test wealth_alerts_integration
cd frontend && npm test -- --run
```

**Boundaries:** R-0024 warn-only sync semantics unchanged; subscription alert path separate; sibling evaluators unchanged; SPA fallback (BUG-0016) unchanged.

#### 32. BUG-0019 hotfix — Grafana provisioning account default + mirror-count panel (Q0027 / released 2026-06-10)

**Release:** BUG-0019 **DONE** — operator notes `handoffs/releases/Q0027-release-notes.md`. Grafana provisioning-only fix per **DEC-0108**: **(CA1/CA2)** `cashflow.json` `$account_id` `sort: 0` + empty `current` + `model_kind = 'baseline'` in panels 1–3; **(CA3)** `forecast-horizons.json` `sort: 0` + `current`; **(CB1)** `platform-health.json` panel 2 mirror `COUNT(*)` UNION ALL SQL.

**Deploy (Grafana provisioning reload only — no backend image change):**

```bash
docker compose restart grafana
```

**Operator gate — GRAFANA_PROVISIONING_RELOAD (required before runtime probes):**

```bash
docker compose restart grafana
# Confirm StartedAt fresh; Grafana API serves sort:0 + current + mirror SQL
```

**Operator gate — FULL_FIREFLY_SYNC_PLUS_INCREMENTAL_RERUN (required for BH incremental regression):**

```bash
curl -X POST http://localhost:18080/api/v1/sync/trigger
# Full sync baseline; then incremental with 0 new transactions — panel 2 must match mirror COUNT
```

**Operator smoke (post-restart):**

| Step | Check | Pass |
|------|-------|------|
| Static guard | python3 JSON assertions | 21/21 |
| Provisioning test | `cd backend && cargo test --test grafana_provisioning_bug0009` | 6/6 |
| BG-DIRECT | Grafana Cashflow (no `var-account_id`) | Default funded account; 731/731 non-zero series |
| BG-API | `GET /api/v1/forecast/monthly?account_id=114` | 25 points; non-zero from Jul 2026 |
| BG-FH | Forecast Horizons default account | sort:0 + current |
| BH-FULL | Platform Health panel 2 after Full sync | transactions = mirror COUNT (922 fixture) |
| BH-INCR | After 0-new-tx incremental sync | Panel count unchanged |
| OIDC-1 | Omniflow BG/BH regression | Operator OIDC browser (optional post-release) |

**Rollback:**

```bash
git revert <Q0027-dashboard-json-commits>
docker compose restart grafana
```

**Automated regression:**

```bash
cd backend && cargo test --test grafana_provisioning_bug0009
```

**Boundaries:** `upsert_cursor` / sync semantics unchanged; `AnalyticsEmbedPage.tsx` unchanged; alert evaluation (BUG-0018) unchanged; duplicate-UID provisioning warning pre-existing — recommend follow-up bug.

#### 33. BUG-0020 hotfix — subscription list reconcile + display_category backfill (Q0028 / released 2026-06-11)

**Release:** BUG-0020 **DONE** — operator notes `handoffs/releases/Q0028-release-notes.md`. Subscription list data-quality fix per **DEC-0109**: **(DA1)** migration 016 YouTube confirmed merge + Strom pending collapse; **(DB1)** confirmed `display_category_id` backfill (DEC-0100 RANK); **(DA2)** SubscriptionsPage All-tab `pending`+`confirmed` only; **(DA3)** detection forward pending guard.

**Prerequisite — fix docker build blocker (required before image build):**

```bash
# Remove unused `hasForecast` in frontend/src/pages/ForecastPage.tsx (TS6133)
cd frontend && npm run build   # must exit 0 before docker build
```

**Deploy (backend migration + detection guard + frontend filter):**

```bash
AUTHENTIK_SECRET_KEY=unused-external-profile docker compose \
  -f docker-compose.yml -f docker-compose.external.yml \
  --profile external up -d --build flow-finance-ai
```

Local override (`:18080`):

```bash
docker compose up -d --build flow-finance-ai
```

**Operator gate — MIGRATION_016_APPLY (required before BI/BJ runtime probes):**

```bash
cd backend && sqlx migrate run
# Migration 016 may already be applied manually via psql; resolve migration 15 checksum conflict if sqlx fails
```

Confirm: 6 confirmed rows; 6/6 `display_category_id` non-null; ≤1 YouTube confirmed; Strom pending collapsed.

**Operator gate — FULL_FIREFLY_SYNC (required for detection regression):**

```bash
curl -X POST http://localhost:18080/api/v1/sync/trigger
# Confirm no new duplicate confirmed YouTube after sync
```

**Operator smoke (post-deploy):**

| Step | Check | Pass |
|------|-------|------|
| TS6133 fix | `cd frontend && npm run build` | exit 0 |
| Integration | `cd backend && cargo test --test bug0020_subscription_list_quality` | 7/7 |
| Regression | `cargo test --test bug0008_subscription_alerts` + `subscriptions_integration` | 8/8 + 1/1 |
| BI-API | `GET /api/v1/subscriptions?status=confirmed` | ≤1 per payee_key; 1 YouTube |
| BJ oracle | netflix/kindle→18, youtube→66, hgp→56, florian→3 | R-0090 samples match |
| BI-ALL | `/subscriptions` All tab | No triplicate Strom / duplicate YouTube |
| REG-DETECT | Post full sync | No new YouTube dup |
| OIDC-1 | Omniflow list endpoints | HTTP 200 (optional) |

**Rollback:**

```bash
git revert <Q0028-migration-and-code-commits>
# Restore pre-migration DB from backup if reconcile ran in production
docker compose up -d --build flow-finance-ai
```

**Boundaries:** Unfiltered `GET /api/v1/subscriptions` unchanged; discover/tags API unchanged; All-tab scope change — rejected/inactive hidden per DEC-0109.

#### 34. BUG-0021 hotfix — CategoryFilter static import + wealth Role column (Q0029 / released 2026-06-11)

**Release:** BUG-0021 **DONE** — operator notes `handoffs/releases/Q0029-release-notes.md`. Frontend UX polish per **DEC-0110** + **DEC-0111**: **(EA1/EA2)** static `CategoryFilter` on Forecast Monthly and Wealth Overview; **(EA3)** PlanningPage P2 parity; **(EB1)** `COALESCE(attributes, root)` `account_role` in `load_asset_accounts`; **(EB2)** `formatAccountRole` label map.

**Deploy (backend SQL + frontend static imports):**

```bash
AUTHENTIK_SECRET_KEY=unused-external-profile docker compose \
  -f docker-compose.yml -f docker-compose.external.yml \
  --profile external up -d --build flow-finance-ai
```

Local override (`:18080`):

```bash
docker compose up -d --build flow-finance-ai
```

**Operator gate — BACKEND_FRONTEND_DEPLOY (required before BK browser + BL API/UI oracles):**

Running container predates Q0029. Confirm `cd frontend && npm run build` and `cargo test --test bug0021_wealth_account_role` pass before docker build. Set `AUTHENTIK_SECRET_KEY` on external profile if compose build requires it.

**Operator gate — SNAPSHOT_UPSERT_OR_SYNC (optional for BL-SNAPSHOT / BL-GRAFANA):**

```bash
curl -X POST http://localhost:18080/api/v1/sync/trigger
# Or wait for daily net_worth_snapshots upsert after deploy
```

**Operator smoke (post-deploy):**

| Step | Check | Pass |
|------|-------|------|
| Build | `cd frontend && npm run build` | exit 0; no `CategoryFilter` lazy chunk |
| Integration | `cd backend && cargo test --test bug0021_wealth_account_role` | 4/4 |
| BK-FORECAST | Forecast → Monthly — CategoryFilter | ≤1s interactive; no **Loading category filter…** |
| BK-WEALTH | Wealth → Overview — CategoryFilter | Same snappy load |
| BL-API | `GET /api/v1/wealth` | Non-null `account_role` on asset accounts |
| BL-UI | Wealth Account breakdown Role column | Checking / Cash wallet / Savings labels |
| BL-SNAPSHOT | `net_worth_snapshots.payload.accounts` | `account_role` populated post-upsert (optional) |
| OIDC-1 | Omniflow `/api/v1/wealth` | HTTP 200 (optional) |

**Rollback:**

```bash
git revert <Q0029-code-commits>
docker compose up -d --build flow-finance-ai
```

**Boundaries:** `CategoryTrendChart` lazy+Suspense unchanged; subscription list (BUG-0020) unchanged; Grafana provisioning (BUG-0019) unchanged.

#### 35. BUG-0023 hotfix — Crypto wealth EUR values (Q0030 / released 2026-06-12)

**Release:** BUG-0023 **DONE** — operator notes `handoffs/releases/Q0030-release-notes.md`. Crypto wealth EUR display fix per **DEC-0064**, **DEC-0080**, **DEC-0081**, **DEC-0038**: **(BO1–BO3)** Bitunix futures wallet ingest hardening (equity fallback, `code==0` reject, parse-skip warn); **(BP1)** migration 017 `exposure_eur` + `entryValue` persist; **(BP2)** `holdings_all.value_eur = market_value_eur.or(exposure_eur)` with wallet-only subtotal; **(BQ1)** baseline captured before `total_return_pct` in same recompute.

**Deploy (backend only):**

```bash
AUTHENTIK_SECRET_KEY=unused-external-profile docker compose \
  -f docker-compose.yml -f docker-compose.external.yml \
  --profile external up -d --build flow-finance-ai
```

Local override (`:18080`):

```bash
docker compose up -d --build flow-finance-ai
```

**Operator gate — BACKEND_DEPLOY (required before BO/BP/BQ live oracles):**

Rebuild backend; migration `017_bug0023_exposure_eur.sql` applies on startup. Confirm tests pass before docker build:

```bash
cd backend && cargo test --test bug0023_crypto_wealth_eur
cd backend && cargo test --lib
```

**Operator gate — EXCHANGE_SYNC (required after deploy):**

```bash
curl -X POST http://localhost:18080/api/v1/sync/trigger
```

**Operator gate — PNL_RECOMPUTE (required after exchange sync):**

Post-sync PnL recompute populates `exposure_eur`, wallet `market_value_eur`, baseline, and `total_return_pct`. Triggered automatically after exchange sync in normal pipeline.

**Operator smoke (post-deploy + all gates):**

| Step | Check | Pass |
|------|-------|------|
| Integration | `cd backend && cargo test --test bug0023_crypto_wealth_eur` | 4/4 |
| BO-API | `GET /api/v1/wealth` | `crypto.subtotal_eur` ~€2000; `bitunix.subtotal_eur` not €0 |
| BO-UI | Wealth → Crypto — Bitunix card | Not **€-0,00** |
| BP-API | `GET /api/v1/wealth` | Linear `holdings_all[].value_eur` non-null |
| BP-UI | Holdings Value EUR column | Not all em dash |
| BQ-API | `GET /api/v1/wealth` | `pnl.total_return_pct` non-null |
| BQ-UI | PnL Total return % | Not em dash with non-zero unrealized |
| OIDC-1 | Omniflow `/api/v1/wealth` | HTTP 200 (optional) |

**Rollback:**

```bash
git revert <Q0030-code-commits>
docker compose up -d --build flow-finance-ai
```

**Boundaries:** `holdings_top` wallet-only filter unchanged (DEC-0064); CategoryFilter (BUG-0021) unchanged; subscription list (BUG-0020) unchanged.

#### 36. BUG-0022 hotfix — Plan delete selector regression (Q0031 / released 2026-06-13)

**Release:** BUG-0022 **DONE** — operator notes `handoffs/releases/Q0031-release-notes.md`. Plan delete selector fix per **DEC-0082**, **DEC-0024**, **DEC-0074**: **(BM1)** `resolveDisplayedPlanId` (`selectedPlanId ?? globalActiveId ?? firstPlanId`); **(T1)** `planSelector.ts` + 8 vitest cases for `isDeleteDisabled` matrix. Frontend-only — no backend changes.

**Deploy (frontend rebuild only — no migration):**

```bash
AUTHENTIK_SECRET_KEY=unused-external-profile docker compose \
  -f docker-compose.yml -f docker-compose.external.yml \
  --profile external up -d --build flow-finance-ai
```

Local override (`:18080`):

```bash
docker compose up -d --build flow-finance-ai
```

**Operator gate — FRONTEND_DEPLOY (required before BM/BN live UI smoke):**

Running container predates Q0031; `/planning` returns **404** pre-deploy. Confirm tests pass before docker build:

```bash
cd frontend && npm test && npm run build
```

**Operator smoke (post-deploy):**

| Step | Check | Pass |
|------|-------|------|
| Build | `cd frontend && npm test && npm run build` | 17/17; exit 0 |
| BM-UI | `/planning` with 2+ plans — select non-active | Delete plan **enabled** |
| BM-UI | Confirm delete modal | Non-active plan removed; list refreshes |
| BN-UI | Select globally active plan | Delete **disabled** + tooltip |
| BN-API | `DELETE /api/v1/plans/:active_id` | **409** `active_plan_delete_forbidden` |
| OIDC-1 | Omniflow `/planning` + `/api/v1/plans` | HTTP 200 (optional) |

**Rollback:**

```bash
git revert <Q0031-code-commits>
docker compose up -d --build flow-finance-ai
```

**Boundaries:** Backend DEC-0082 409 guard unchanged; crypto wealth (BUG-0023) unchanged; CategoryFilter (BUG-0021) unchanged.

#### 37. BUG-0026 hotfix — Forecast monthly Income card mismatch (Q0032 / released 2026-06-13)

**Release:** BUG-0026 **DONE** — operator notes `handoffs/releases/Q0032-release-notes.md`. Forecast summary month fix per **DEC-0089**: **(H1)** `resolveForecastSummaryPoint` skips partial zero-income head; **(F1)** shared subtitle above card grid; **(T1)** 7 vitest cases including account **114** partial-month trap. Frontend-only — no backend changes.

**Deploy (frontend rebuild only — no migration):**

```bash
AUTHENTIK_SECRET_KEY=unused-external-profile docker compose \
  -f docker-compose.yml -f docker-compose.external.yml \
  --profile external up -d --build flow-finance-ai
```

Local override (`:18080`):

```bash
docker compose up -d --build flow-finance-ai
```

**Operator gate — FRONTEND_DEPLOY (required before BZ/CA live UI smoke):**

Running container predates Q0032; browser reproduces Income **0.00** + no subtitle pre-deploy. Confirm tests pass before docker build:

```bash
cd frontend && npm test && npm run build
```

**Operator smoke (post-deploy):**

| Step | Check | Pass |
|------|-------|------|
| Build | `cd frontend && npm test && npm run build` | 24/24; exit 0 |
| BZ-UI | `/forecast` Monthly account **114** | Income card **3266.16** matches July chart bar |
| CA-UI | Summary cards subtitle | **"Forecast for July 2026"** above four cards |
| BZ-API | `GET /api/v1/forecast/monthly?account_id=114` | series[1] income **3266.16** (live-confirmed pre-deploy) |
| DEC-0089 | Category filter on `/forecast` | Card values unchanged |
| OIDC-1 | Omniflow `/forecast` + monthly API | HTTP 200 (optional) |

**Rollback:**

```bash
git revert <Q0032-code-commits>
docker compose up -d --build flow-finance-ai
```

**Boundaries:** Backend forecast API unchanged; plan delete (BUG-0022) unchanged; crypto wealth (BUG-0023) unchanged; CategoryFilter (BUG-0021) unchanged.

#### 39. BUG-0024 hotfix — Plan delete sole-plan copy gap (Q0033 / released 2026-06-13)

**Release:** BUG-0024 **DONE** — operator notes `handoffs/releases/Q0033-release-notes.md`. Sole-plan delete guidance per **DEC-0082**: **(H1)** `shouldShowSolePlanDeleteHint` + `SOLE_PLAN_DELETE_HINT`; **(F1)** inline muted hint below **Delete plan** row on `/planning`; **(T1)** +7 vitest cases. Frontend-only — no backend changes. **Q0031** selector regression preserved.

**Deploy (frontend rebuild only — no migration):**

```bash
AUTHENTIK_SECRET_KEY=unused-external-profile docker compose \
  -f docker-compose.yml -f docker-compose.external.yml \
  --profile external up -d --build flow-finance-ai
```

Local override (`:18080`):

```bash
docker compose up -d --build flow-finance-ai
```

**Operator gate — FRONTEND_DEPLOY (required before BS live UI smoke):**

Running container predates Q0033; sole-plan inline hint absent pre-deploy. Confirm tests pass before docker build:

```bash
cd frontend && npm test && npm run build
```

**Operator smoke (post-deploy):**

| Step | Check | Pass |
|------|-------|------|
| Build | `cd frontend && npm test && npm run build` | 31/31; exit 0 |
| BS-UI | `/planning` with 1 sole active plan | Delete disabled + inline *To delete this plan, create another scenario, set it active, then delete this one.* |
| BR-UI | `/planning` with 2+ plans — select non-active | Delete plan **enabled** |
| BR-UI | Confirm delete modal | Non-active plan removed; list refreshes |
| BN-UI | Select globally active plan | Delete **disabled** + tooltip |
| BN-API | `DELETE /api/v1/plans/:active_id` | **409** `active_plan_delete_forbidden` |
| OIDC-1 | Omniflow `/planning` + `/api/v1/plans` | HTTP 200 (optional) |

**Rollback:**

```bash
git revert <Q0033-code-commits>
docker compose up -d --build flow-finance-ai
```

**Boundaries:** Backend DEC-0082 409 guard unchanged; forecast Income card (BUG-0026) unchanged; crypto wealth (BUG-0023) unchanged; CategoryFilter (BUG-0021) unchanged.

#### 40. BUG-0025 hotfix — Firefly Stromkosten mirror lag (Q0034 / released 2026-06-14)

**Release:** BUG-0025 **DONE** — operator notes `handoffs/releases/Q0034-release-notes.md`. Firefly mirror lag fix per **DEC-0002** extension: **(B1)** manual **Sync now** uses **365-day** transaction-date lookback; scheduled incremental unchanged; **(B2)** `last_firefly_run` API split from exchange-only `last_run`; **(F1)** Sync Status hero + trigger badge + exchange secondary + DEC-0002 callout; **(D1)** runbook `#backdated-firefly-imports` + cursor-reset SQL; **(T1)** integration 3/3.

**Deploy (backend + frontend rebuild — no migration):**

```bash
AUTHENTIK_SECRET_KEY=unused-external-profile docker compose \
  -f docker-compose.yml -f docker-compose.external.yml \
  --profile external up -d --build flow-finance-ai
```

Local override (`:18080`):

```bash
docker compose up -d --build flow-finance-ai
```

**Operator gate — BACKEND_REBUILD + FRONTEND_DEPLOY (required before BW/BX/BY live smoke):**

Running container predates Q0034; `last_firefly_run` absent; expense-series category **146** shows 2026-05 only; DEC-0002 callout absent. Confirm tests pass before docker build:

```bash
cd backend && cargo test --lib && cargo test --test bug0025_sync_transaction_window
cd frontend && npm test && npm run build
```

**Post-deploy — manual Full sync:**

On **Sync Status** (`/sync`), click **Sync now** — triggers manual Full Firefly ingest with 365-day lookback.

**Operator smoke (post-deploy):**

| Step | Check | Pass |
|------|-------|------|
| Build | backend lib + bug0025 3/3; frontend npm 31/31 + build | exit 0 |
| BW-API | `GET /api/v1/categories/expense-series?category_id=146` after manual Sync now | Multi-month Stromkosten bars — not 2026-05 only |
| BW-UI | `/forecast` Category spending trend **Wohnen - Stromkosten** | Bars per month with Firefly data |
| BX-UI | `/sync` DEC-0002 callout + runbook link | Callout visible |
| BY-API | `GET /api/v1/sync/status` | `last_firefly_run` distinct from exchange-only `last_run` |
| BY-UI | `/sync` hero **Last Firefly sync** + trigger badge | Hero uses Firefly run; exchange secondary when newer |
| BY-HIST | Sync history `trigger` column | manual / scheduled / scheduled_exchanges distinguished |
| OIDC-1 | Omniflow `/sync` + `/forecast` | HTTP 200 (optional) |

**Rollback:**

```bash
git revert <Q0034-code-commits>
docker compose up -d --build flow-finance-ai
```

**Boundaries:** Scheduled incremental sync (`watermark − overlap_days`) unchanged; plan delete sole-plan hint (BUG-0024) unchanged; forecast Income card (BUG-0026) unchanged; crypto wealth (BUG-0023) unchanged.

#### 38. US-0021 — Subscription transaction explorer with rich filters (S0020 / released 2026-06-13)

**Release:** US-0021 **DONE** — operator notes `handoffs/releases/S0020-release-notes.md`. Dual-mode Discover on US-0010 external profile per **DEC-0112**, **DEC-0113**, **DEC-0114**: **(AC-1)** `GET /api/v1/subscriptions/transactions/search` + Transactions mode (individual expense rows, 100/page); **(AC-2)** rich filters — account, payee, category, Geldbereich, date range; **(AC-3)** hint badges on filtered subset (row metadata only, no auto-emit); **(AC-4)** multi-select → `POST /transactions/preview-group` → confirm via DEC-0099/DEC-0085; **(AC-5)** Suggested patterns sub-tab unchanged (DEC-0098); US-0020 tags/majority + US-0003/US-0008 regression preserved; **(AC-6)** OIDC smoke template (pass-with-prerequisites at release).

**Deploy (backend + frontend — no migration):**

```bash
AUTHENTIK_SECRET_KEY=unused-external-profile docker compose \
  -f docker-compose.yml -f docker-compose.external.yml \
  --profile external up -d --build flow-finance-ai
```

**Operator gate — BACKEND_FRONTEND_DEPLOY (required before AC-1..AC-4 and AC-6 live smoke):**

Running container predates S0020; tx-search returns **404** and `/subscriptions` returns **404** pre-deploy. Confirm tests pass before docker build:

```bash
cd backend && cargo test --lib && cargo test --test us0021_transaction_search
cd frontend && npm test && npm run build
```

```bash
docker compose -f docker-compose.yml -f docker-compose.external.yml \
  --profile external up -d --force-recreate flow-finance-ai
```

Local override (`:18080`):

```bash
docker compose up -d --build flow-finance-ai
```

**Operator smoke (post-deploy):**

| Step | Check | Pass |
|------|-------|------|
| Backend unit | `cd backend && cargo test --lib` | 221/221 |
| Integration | `cd backend && cargo test --test us0021_transaction_search` | 6/6 |
| Frontend unit | `cd frontend && npm test` | 17/17 |
| App health | `curl -sf https://financegnome.omniflow.cc/health` | HTTP 200 |
| AC-1 | `/subscriptions` Discover → **Transactions** | Individual expense rows; 100/page pagination |
| AC-2 | Rich filters | Account, payee, category, Geldbereich, date range |
| AC-3 | Hint badges | Account **114** + payee **SEPA-Lastschrift** — interval/confidence badges |
| AC-4 | Multi-select activate | ≥2 txs → preview-group → confirm subscription/standing order |
| AC-5 | **Suggested patterns** sub-tab | US-0020 discover candidates unchanged |
| AC-5 | Regression | US-0020 tags/majority; US-0003 pending; US-0008 alert dedup |
| AC-6 | OIDC external profile | Discover tx search + confirm flow |
| API probe | `GET /api/v1/subscriptions/transactions/search?account_id=114` | HTTP 200 (not 404) |

**Automated regression:**

```bash
cd backend && cargo test --lib
cd backend && cargo test --test us0021_transaction_search
cd frontend && npm test
```

**Boundaries:** `DetectionPipeline::run_candidates` unchanged; global `min_emit_confidence: 60` unchanged; DEC-0098 discover path frozen; user guide `docs/user-guides/US-0021.md`.

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
