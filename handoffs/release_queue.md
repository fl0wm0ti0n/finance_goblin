# Release Queue Tracker

Canonical release queue for sprint-level release state.

## Queue rows

| sprint_id | story_refs | status | last_updated | release_notes_ref | gate_snapshot | release_version | remediation |
|-----------|------------|--------|--------------|-------------------|---------------|-----------------|-------------|
| S0001 | US-0001 | released | 2026-05-31T18:20:12Z | handoffs/releases/S0001-release-notes.md | check-in_test:pass; qa:pass; uat:pass; isolation:pass; publish:skipped(disabled) | 0.1.0-us0001 | — |
| S0002 | US-0002 | released | 2026-05-31T18:33:54Z | handoffs/releases/S0002-release-notes.md | check-in_test:pass; qa:pass; uat:pass; isolation:pass; publish:skipped(disabled) | 0.2.0-us0002 | — |
| S0003 | US-0003 | released | 2026-05-31T19:20:00Z | handoffs/releases/S0003-release-notes.md | check-in_test:pass; qa:pass; uat:pass; isolation:pass; publish:skipped(disabled) | 0.3.0-us0003 | — |
| S0004 | US-0004 | released | 2026-05-31T21:06:00Z | handoffs/releases/S0004-release-notes.md | check-in_test:pass; qa:pass; uat:pass; isolation:pass; publish:skipped(disabled) | 0.4.0-us0004 | — |
| S0005 | US-0005 | released | 2026-05-31T21:38:05Z | handoffs/releases/S0005-release-notes.md | check-in_test:pass; qa:pass; uat:pass; isolation:pass; publish:skipped(disabled) | 0.5.0-us0005 | — |
| S0006 | US-0006 | released | 2026-06-01T14:45:00Z | handoffs/releases/S0006-release-notes.md | check-in_test:pass; qa:pass; uat:pass; isolation:pass; publish:skipped(disabled) | 0.6.0-us0006 | — |
| S0007 | US-0007 | released | 2026-06-02T00:30:00Z | handoffs/releases/S0007-release-notes.md | check-in_test:pass; qa:pass; uat:pass; isolation:pass; publish:skipped(disabled) | 0.7.0-us0007 | — |
| S0008 | US-0008 | released | 2026-05-31T23:13:47Z | handoffs/releases/S0008-release-notes.md | check-in_test:pass; qa:pass; uat:pass; isolation:pass; publish:skipped(disabled) | 0.8.0-us0008 | — |
| S0009 | US-0009 | released | 2026-06-01T23:50:00Z | handoffs/releases/S0009-release-notes.md | check-in_test:pass; qa:pass; uat:pass; isolation:pass; publish:skipped(disabled) | 0.9.0-us0009 | — |
| S0010 | US-0010 | released | 2026-06-02T23:00:00Z | handoffs/releases/S0010-release-notes.md | check-in_test:pass; qa:pass; uat:pass-with-prerequisites(qa); isolation:pass; publish:skipped(disabled) | 0.10.0-us0010 | — |
| S0011 | US-0011 | released | 2026-06-03T01:30:00Z | handoffs/releases/S0011-release-notes.md | check-in_test:pass; qa:pass; uat:pass-with-prerequisites(verify-work); isolation:pass; publish:skipped(disabled) | 0.11.0-us0011 | — |
| S0012 | US-0012 | released | 2026-06-03T22:00:00Z | handoffs/releases/S0012-release-notes.md | check-in_test:pass; qa:pass; uat:pass-with-prerequisites(verify-work); isolation:pass; publish:skipped(disabled) | 0.12.0-us0012 | — |
| Q0014 | BUG-0012 (+Q0015) | released | 2026-06-06T00:30:00Z | handoffs/releases/Q0014-release-notes.md | check-in_test:pass(142/142); qa:pass; uat:pass; isolation:pass; runtime_proof:pass; publish:skipped(disabled) | bug0012-q0014 | — |
| Q0016 | BUG-0009 | released | 2026-06-06T23:45:00Z | handoffs/releases/Q0016-release-notes.md | check-in_test:pass(6/6 grafana_provisioning_bug0009); qa:pass; uat:pass; isolation:pass; runtime_proof:pass; publish:skipped(disabled) | bug0009-q0016 | — |
| Q0017 | BUG-0007 | released | 2026-06-08T00:00:00Z | handoffs/releases/Q0017-release-notes.md | check-in_test:pass(150/150 lib, 8/8 bug0007_ai_discovery); qa:pass; uat:pass(T partial advisory); isolation:pass; runtime_proof:pass; publish:skipped(disabled) | bug0007-q0017 | — |

## Status model

- `planned`: sprint exists, release flow not entered
- `ready`: verify-work completed and release is eligible to start
- `unreleased`: release flow entered; notes written; finalization not completed
- `released`: release finalization completed for the sprint
- `blocked`: deterministic fail-safe condition requiring remediation

## Deterministic transition contract

- Allowed lifecycle: `planned -> ready -> unreleased -> released`.
- `blocked` can be set on deterministic failure conditions.
- Only the target sprint row may change during one `/release` run.
- No destructive auto-reconciliation is allowed by default.

## Fail-safe reason codes

- `RELEASE_SPRINT_UNRESOLVED`
- `LEGACY_NOTES_SPRINT_UNRESOLVED`
- `QUEUE_ENTRY_MISSING`
- `NOTES_REF_MISSING`
- `STATUS_TRANSITION_INVALID`
- `BACKLOG_STATUS_DRIFT`
- `CANONICAL_STATUS_CONFLICT`
- `COMPATIBILITY_CRITICAL_OPEN`
- `COMPONENT_SCOPE_VIOLATION_UNAPPROVED`

## Remediation guidance

- `RELEASE_SPRINT_UNRESOLVED`: set explicit sprint context (`Sxxxx`) and rerun `/release`.
- `LEGACY_NOTES_SPRINT_UNRESOLVED`: preserve legacy notes, identify sprint manually, then create target sprint notes file.
- `QUEUE_ENTRY_MISSING`: create the target sprint queue row with required fields, then rerun `/release`.
- `NOTES_REF_MISSING`: add canonical `release_notes_ref` for target sprint row and rerun `/release`.
- `STATUS_TRANSITION_INVALID`: correct row status to a valid predecessor state and rerun `/release`.
- `BACKLOG_STATUS_DRIFT`: reconcile target story status/ACs in `docs/product/backlog.md` using release evidence, then rerun `/release`.
- `CANONICAL_STATUS_CONFLICT`: resolve canonical backlog status mismatch versus derived artifacts and rerun `/release`.
- `COMPATIBILITY_CRITICAL_OPEN`: resolve or explicitly decide on open critical compatibility findings before rerun.
- `COMPONENT_SCOPE_VIOLATION_UNAPPROVED`: resolve or explicitly approve out-of-scope component impact before rerun.
