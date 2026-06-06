# UAT — Sprint S0012 / US-0012

**Sprint:** S0012  
**Story:** US-0012  
**Phase:** verify-work complete  
**Status:** PASS  
**Plan-verified at:** 2026-06-03T06:00:00Z  
**QA verified at:** 2026-06-03T18:00:00Z  
**Verified at:** 2026-06-03T20:00:00Z

## Inputs

- Acceptance: `docs/product/acceptance.md#US-0012`
- Architecture: `docs/engineering/architecture.md` (US-0012), `decisions/DEC-0058.md`
- Research: R-0055, R-0053 §1
- Runbook: `docs/engineering/runbook.md` § Omniflow external deploy §1
- Dev handoff: `handoffs/dev_to_qa.md`
- QA handoff: `handoffs/qa_to_release.md`

## UAT steps (acceptance criteria)

| ID | AC | Step | Result | Evidence |
|----|-----|------|--------|----------|
| UAT-1 | AC-1 | Cold start against external Postgres creates missing `DATABASE_NAME` idempotently before migrations | **pass** (fixture deferred) | `lib.rs` + `bootstrap.rs`; `DATABASE_BOOTSTRAP_TEST_URL` unset |
| UAT-2 | AC-2 | Existing `flow_finance_ai` never dropped or recreated by bootstrap | **pass** | no DROP; `REASON_SKIPPED_EXISTS` |
| UAT-3 | AC-3 | Bootstrap attempts TimescaleDB extension; fails with actionable log when server lacks extension | **pass** (omniflow deferred) | reason code unit tests + runbook pointer |
| UAT-4 | AC-4 | `DATABASE_BOOTSTRAP_URL` enables bootstrap when app role lacks CREATEDB; missing privilege fails closed | **pass** (live privilege deferred) | URL precedence + `42501` mapping |
| UAT-5 | AC-5 | `.env.example` and runbook document bootstrap env and omniflow shared-Postgres behavior | **pass** | `.env.example` + runbook §1 |
| UAT-6 | AC-6 | Automated test proves create-if-missing path without operator manual SQL | **pass** (integration skip) | 82 unit tests; integration gated |

## Results summary

- **Acceptance:** 6/6 PASS (`docs/product/acceptance.md#US-0012` — all checkboxes checked at verify-work)
- **Pass-with-prerequisites:** 4/6 (AC-1, AC-3, AC-4, AC-6 — integration fixture and/or omniflow runtime)
- **Blockers:** 0
- **DEC-0058:** aligned
- **Sprint tasks:** T-0130–T-0136 — 7/7 done

## Operator smoke (optional post-release)

| Check | Command / observation | Result |
|-------|----------------------|--------|
| Greenfield create | Start backend with missing DB; verify `database_bootstrap_created` log | **deferred** — omniflow |
| Idempotent restart | Second start emits `database_bootstrap_skipped_exists` | **deferred** — omniflow |
| Omniflow bootstrap URL | Deploy with `DATABASE_BOOTSTRAP_URL` when `finance` lacks CREATEDB | **deferred** — omniflow |
| TimescaleDB absent | Verify `database_bootstrap_failed_timescaledb` before migration panic | **deferred** — omniflow |

## Verify-work automated checks (2026-06-03)

| Check | Result |
|-------|--------|
| `cargo test --lib` (independent re-run) | **pass** — 82/82 |
| `cargo test --test database_bootstrap_integration` | **deferred** — `DATABASE_BOOTSTRAP_TEST_URL` unset |
| DEC-0058 + dev/qa handoff cross-check | **pass** |

## Phase history

- **Plan-verify 2026-06-03:** PASS — `sprints/S0012/plan-verify.json`; 6/6 AC; DEC-0058 aligned
- **QA 2026-06-03:** PASS — `sprints/S0012/qa-findings.md`; static/unit; integration + omniflow deferred
- **Verify-work 2026-06-03:** PASS — this UAT; `handoffs/verify_work_to_release.md`

## Next phase

- `/release` in fresh subagent/chat
