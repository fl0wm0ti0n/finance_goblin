---
description: "its-magic verify work: guided user acceptance testing."
---

# /verify-work

## Subagents
- qa

## Execution model
- Run `/verify-work` in a fresh QA subagent context.
- After writing outputs, stop and hand off to `/release` (or back to `/execute`
  if failures require fixes) in a new subagent/chat.

## Isolation evidence write requirement (US-0048 / DEC-0029)

At the end of `/verify-work`, append an isolation evidence entry to
`docs/engineering/state.md`:

- `phase_id=verify-work`
- `role=qa`
- `fresh_context_marker=<new marker for this subagent>`
- `timestamp=<ISO UTC>`
- `evidence_ref=<primary output ref>` (recommended: `sprints/Sxxxx/uat.json` and `sprints/Sxxxx/uat.md`)

## Inputs
- `docs/product/acceptance.md`
- `sprints/S0001/summary.md`

## Outputs (artifacts)
- `sprints/S0001/uat.json`
- `sprints/S0001/uat.md`
- `docs/engineering/state.md`

## Stop conditions
- Decision gate triggered

## UAT lifecycle rules (DEC-0009)

UAT artifacts transition from **placeholder** (created during `/sprint-plan`) to
**populated** (filled during `/verify-work`) to **verified** (confirmed during
`/release`). QA owns the placeholder → populated transition.

### Minimum UAT content before sprint completion
- `uat.json`: `steps` array is non-empty. Each step has a `description` and
  `result` (`pass` or `fail`). `passed` + `failed` = total steps count.
- `uat.md`: every UAT step is listed with its result. A results summary appears
  at the bottom linking back to story acceptance criteria.
- A sprint **cannot** be marked complete while UAT artifacts remain in
  placeholder state.

## Isolation compliance gate (US-0048 / DEC-0029)

Before handing off to `/release`, verify isolation evidence is present and valid
in `docs/engineering/state.md` for the target sprint lifecycle (at minimum:
`execute`, `qa`, and `verify-work`).

Fail-closed behavior (no continuation):

- Missing evidence: `PHASE_CONTEXT_ISOLATION_MISSING`
- Invalid schema/fields: `ISOLATION_EVIDENCE_INVALID`
- Stale evidence (reused marker / pre-resume evidence): `ISOLATION_EVIDENCE_STALE`
- Orchestrator/phase executed without fresh subagent: `PHASE_CONTEXT_ISOLATION_VIOLATION`

Remediation: re-run the missing/invalid phase(s) in fresh subagent contexts and
write new isolation evidence, then rerun `/verify-work` before proceeding to
`/release`.

## Strict runtime proof gate (US-0056 / DEC-0038)

Before handing off to `/release`, verify strict runtime proof tuples are present
and valid for the target lifecycle phases (`execute`, `qa`, `verify-work`).

Fail-closed behavior (no continuation):

- Missing runtime proof tuple: `RUNTIME_PROOF_MISSING`
- Invalid tuple shape/hash/linkage: `RUNTIME_PROOF_INVALID`
- Reused `runtime_proof_id`: `RUNTIME_PROOF_REUSED`
- Expired proof TTL/stale proof: `RUNTIME_PROOF_STALE`
- Ambiguous proof-to-checkpoint mapping: `RUNTIME_PROOF_AMBIGUOUS_LINK`

Remediation: rerun affected phase(s) in fresh subagent contexts and write new
strict-proof tuples linked to checkpoint evidence.

## Generated-test readiness evidence gate (US-0066 / DEC-0048)

Before handing off to `/release`, verify generated-test evidence is present and
traceable for generated-project scope:

- `sprints/Sxxxx/summary.md` must include generated baseline test scope/evidence refs.
- `sprints/Sxxxx/qa-findings.md` must include generated-test auto-run evidence
  (`command`, `result`, `output ref`, `paths ref`, reason code when failing).
- If generated-test evidence is missing/ambiguous, fail closed with
  `TEST_SCAFFOLD_GENERATION_FAILED` and require `/execute` or `/qa` rerun with
  deterministic evidence capture.

## Steps
1. Convert acceptance criteria into testable UAT steps. Derive steps directly from the story's acceptance criteria in `docs/product/acceptance.md`. Each AC should map to at least one UAT step.
2. Populate UAT artifacts: write derived steps into `uat.json` (with description and result per step, accurate pass/fail counts) and `uat.md` (step list with results, summary section). Ensure UAT artifacts are in **populated** state per DEC-0009 — not placeholder.
3. Record results and failures.
4. Update state with pass/fail summary.
5. Run the isolation compliance gate (above). If it fails, stop and hand off for remediation.
6. Update traceability index in `docs/engineering/state.md`: for each story verified in this sprint, set Status to `PASS` or `FAIL` and fill the Evidence column with artifact references (e.g., `S0001/uat.json`, `S0001/summary.md`). Use the DEC-0010 format.
7. Pre-handoff traceability check: confirm no OPEN or DONE story in the current sprint lacks a traceability index entry. If a gap is found, add the missing row before proceeding with the handoff.
8. If `AUTO_IMPLEMENTATION_LOOP=1` and UAT fails, write a handoff to dev/QA and
   continue the fix loop within `AUTO_LOOP_MAX_CYCLES`.
