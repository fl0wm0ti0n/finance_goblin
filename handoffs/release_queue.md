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
| S0013 | US-0016 | released | 2026-06-08T04:40:00Z | handoffs/releases/S0013-release-notes.md | check-in_test:pass-with-story-scope(doc-profile exit 0); qa:pass; uat:pass; isolation:pass; runtime_proof:pass; publish:skipped(disabled) | 0.13.0-us0016 | — |
| Q0018 | BUG-0008 | released | 2026-06-08T06:25:00Z | handoffs/releases/Q0018-release-notes.md | check-in_test:pass(156/156 lib, 8/8 bug0008_subscription_alerts); qa:pass; uat:pass-with-prerequisites(verify-work); isolation:pass; runtime_proof:pass; publish:skipped(disabled) | bug0008-q0018 | — |
| Q0019 | BUG-0011 | released | 2026-06-08T08:45:00Z | handoffs/releases/Q0019-release-notes.md | check-in_test:pass(160/160 lib, 5/5 plans_integration); qa:pass; uat:pass-with-prerequisites(qa); isolation:pass; runtime_proof:pass; publish:skipped(disabled) | bug0011-q0019 | — |
| S0014 | US-0013 | released | 2026-06-08T11:15:00Z | handoffs/releases/S0014-release-notes.md | check-in_test:pass(compose-config-check, forecast_ml_integration 3/3); qa:pass; uat:pass-with-prerequisites(qa); isolation:pass; runtime_proof:pass; publish:skipped(disabled) | 0.14.0-us0013 | — |
| S0015 | US-0014 | released | 2026-06-08T13:30:00Z | handoffs/releases/S0015-release-notes.md | check-in_test:pass(npm test 5/5, plans_integration 5/5); qa:pass; uat:pass-with-prerequisites(verify-work); isolation:pass; runtime_proof:pass; publish:skipped(disabled) | 0.15.0-us0014 | — |
| S0016 | US-0015 | released | 2026-06-06T19:30:00Z | handoffs/releases/S0016-release-notes.md | check-in_test:pass(cargo lib 169/169, npm test 5/5); qa:pass; uat:pass-with-prerequisites(verify-work); isolation:pass; runtime_proof:pass; publish:skipped(disabled) | 0.16.0-us0015 | — |
| Q0020 | BUG-0013 | released | 2026-06-09T02:00:00Z | handoffs/releases/Q0020-release-notes.md | check-in_test:pass(174/174); qa:pass; uat:pass-with-prerequisites(verify-work); isolation:pass; runtime_proof:pass; publish:skipped(disabled) | bug0013-q0020 | — |
| Q0021 | US-0017 | released | 2026-06-09T22:00:00Z | handoffs/releases/Q0021-release-notes.md | check-in_test:pass-with-story-scope(doc-profile exit 0); qa:pass; uat:pass; isolation:pass; runtime_proof:pass; publish:skipped(disabled) | 0.17.0-us0017 | — |
| Q0022 | BUG-0014 | released | 2026-06-07T12:00:00Z | handoffs/releases/Q0022-release-notes.md | check-in_test:pass(177/177); qa:pass; uat:pass-with-prerequisites(verify-work); isolation:pass; runtime_proof:pass; publish:skipped(disabled) | bug0014-q0022 | — |
| Q0023 | BUG-0015 | released | 2026-06-07T14:00:00Z | handoffs/releases/Q0023-release-notes.md | check-in_test:pass(187/187); qa:pass; uat:pass-with-prerequisites(verify-work); isolation:pass; runtime_proof:pass; publish:skipped(disabled) | bug0015-q0023 | — |
| S0017 | US-0018 | released | 2026-06-09T00:00:00Z | handoffs/releases/S0017-release-notes.md | check-in_test:pass(193/193 lib, 7/7 vitest); qa:pass; uat:pass-with-prerequisites(verify-work); isolation:pass; runtime_proof:pass; publish:skipped(disabled) | 0.18.0-us0018 | — |
| S0018 | US-0019 | released | 2026-06-09T23:30:00Z | handoffs/releases/S0018-release-notes.md | check-in_test:pass(204/204 lib, 9/9 vitest); qa:pass; uat:pass-with-prerequisites(verify-work); isolation:pass; runtime_proof:pass; publish:skipped(disabled) | 0.19.0-us0019 | — |
| S0019 | US-0020 | released | 2026-06-10T23:30:00Z | handoffs/releases/S0019-release-notes.md | check-in_test:pass(213/213 lib, 9/9 vitest); qa:pass; uat:pass-with-prerequisites(verify-work); isolation:pass; runtime_proof:pass; publish:skipped(disabled) | 0.20.0-us0020 | — |
| Q0024 | BUG-0016 | released | 2026-06-09T20:42:00Z | handoffs/releases/Q0024-release-notes.md | check-in_test:pass(213/213 lib, spa_fallback 5/5, 9/9 vitest); qa:pass; uat:pass-with-prerequisites(verify-work); isolation:pass; runtime_proof:pass; publish:skipped(disabled) | bug0016-q0024 | — |
| Q0025 | BUG-0017 | released | 2026-06-10T23:10:00Z | handoffs/releases/Q0025-release-notes.md | check-in_test:pass(213/213 lib, forecast_integration 3/3, 9/9 vitest); qa:pass; uat:pass-with-prerequisites(verify-work); isolation:pass; runtime_proof:pass; publish:skipped(disabled) | bug0017-q0025 | — |
| Q0026 | BUG-0018 | released | 2026-06-10T23:20:00Z | handoffs/releases/Q0026-release-notes.md | check-in_test:pass(213/213 lib, wealth_alerts_integration 3/3, 9/9 vitest); qa:pass; uat:pass-with-prerequisites(verify-work); isolation:pass; runtime_proof:pass; publish:skipped(disabled) | bug0018-q0026 | — |
| Q0027 | BUG-0019 | released | 2026-06-10T21:11:18Z | handoffs/releases/Q0027-release-notes.md | check-in_test:pass(grafana_provisioning_bug0009 6/6, static guard 21/21); qa:pass; uat:pass-with-prerequisites(verify-work); isolation:pass; runtime_proof:pass; publish:skipped(disabled) | bug0019-q0027 | — |
| Q0028 | BUG-0020 | released | 2026-06-11T09:45:00Z | handoffs/releases/Q0028-release-notes.md | check-in_test:pass(bug0020 7/7, bug0008 8/8, subscriptions_integration 1/1); qa:pass; uat:pass-with-prerequisites(verify-work); isolation:pass; runtime_proof:pass; publish:skipped(disabled) | bug0020-q0028 | — |
| Q0029 | BUG-0021 | released | 2026-06-11T13:00:00Z | handoffs/releases/Q0029-release-notes.md | check-in_test:pass(bug0021 4/4, lib 213/213, npm 9/9); qa:pass; uat:pass-with-prerequisites(verify-work); isolation:pass; runtime_proof:pass; publish:skipped(disabled) | bug0021-q0029 | — |
| Q0030 | BUG-0023 | released | 2026-06-12T22:00:00Z | handoffs/releases/Q0030-release-notes.md | check-in_test:pass(bug0023 4/4, lib 218/218, npm 9/9); qa:pass; uat:pass-with-prerequisites(verify-work); isolation:pass; runtime_proof:pass; publish:skipped(disabled) | bug0023-q0030 | — |
| Q0031 | BUG-0022 | released | 2026-06-13T12:00:00Z | handoffs/releases/Q0031-release-notes.md | check-in_test:pass(npm 17/17, lib active_plan_delete 1/1); qa:pass; uat:pass-with-prerequisites(verify-work); isolation:pass; runtime_proof:pass; publish:skipped(disabled) | bug0022-q0031 | — |
| S0020 | US-0021 | released | 2026-06-13T10:45:00Z | handoffs/releases/S0020-release-notes.md | check-in_test:pass(221/221 lib, us0021 6/6, npm 17/17); qa:pass; uat:pass-with-prerequisites(verify-work); isolation:pass; runtime_proof:pass; publish:skipped(disabled) | 0.21.0-us0021 | — |
| Q0032 | BUG-0026 | released | 2026-06-13T15:00:00Z | handoffs/releases/Q0032-release-notes.md | check-in_test:pass(npm 24/24); qa:pass; uat:pass-with-prerequisites(verify-work); isolation:pass; runtime_proof:pass; publish:skipped(disabled) | bug0026-q0032 | — |
| Q0033 | BUG-0024 | released | 2026-06-13T16:00:00Z | handoffs/releases/Q0033-release-notes.md | check-in_test:pass(npm 31/31); qa:pass; uat:pass-with-prerequisites(verify-work); isolation:pass; runtime_proof:pass; publish:skipped(disabled) | bug0024-q0033 | — |
| Q0034 | BUG-0025 | released | 2026-06-14T18:00:00Z | handoffs/releases/Q0034-release-notes.md | check-in_test:pass(221/221 lib, bug0025 3/3, npm 31/31); qa:pass; uat:pass-with-prerequisites(verify-work); isolation:pass; runtime_proof:pass; publish:skipped(disabled) | bug0025-q0034 | — |
| S0021 | US-0022 | released | 2026-06-14T19:23:00Z | handoffs/releases/S0021-release-notes.md | check-in_test:pass(221/221 lib, meta_test 3/3, npm 31/31); qa:pass; uat:pass-with-prerequisites(verify-work); isolation:pass; runtime_proof:pass; publish:skipped(disabled) | 0.22.0-us0022 | — |

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
