# QA Findings — Quick Q0027 / BUG-0019 (cycle 2 re-run)

**Work item:** BUG-0019 (defect)  
**Quick task:** Q0027  
**QA phase:** `/qa` (implementation loop cycle 2)  
**Date:** 2026-06-10  
**Orchestrator:** `auto-20260610-bug0019`  
**Decisions:** DEC-0108 (supersedes DEC-0068 Y1 omit-`current` clause)  
**QA agent:** fresh subagent (`qa-20260610-bug0019-qa-cycle2-fresh`)  
**Prior QA:** cycle 1 FAIL — stale regression test (resolved in execute fix cycle 2)

## Verdict

**PASS** — All `handoffs/qa_to_dev.md` return items resolved. Regression suite **6/6 PASS**; supersession recorded; fix cycle 2 scope confirmed test + decision docs only (no dashboard JSON or backend code changes). Cycle 1 runtime evidence (BG/BH oracles, static guard 16/16) remains valid. Hand off to **`/verify-work`** for V1 operator gates.

## Scope (cycle 2 re-run)

Focused re-verification of execute fix cycle 2 remediation only. Cycle 1 QA already validated the three dashboard JSON edits, BG/BH runtime oracles, and duplicate-UID classification.

**Inputs reviewed:** `handoffs/dev_to_qa.md` (fix cycle 2 top section), `handoffs/qa_to_dev.md`, `handoffs/qa_report.md`, `sprints/quick/Q0027/progress.md`, `decisions/DEC-0108.md`, `docs/engineering/decisions.md` § DEC-0068 Y1, `backend/tests/grafana_provisioning_bug0009.rs`. No host `.env`/secret files read.

## Test plan (cycle 2)

| # | Test | Command / method | Result |
|---|------|-------------------|--------|
| T-1 | Fix cycle 2 scope — no dashboard/backend delta beyond test | `git diff HEAD --stat` scoped inspection; dev handoff file list | **PASS** — cycle 2 touches `grafana_provisioning_bug0009.rs`, `decisions/DEC-0108.md`, `docs/engineering/decisions.md`, `progress.md`, `dev_to_qa.md` only; grafana JSON diffs are execute cycle 1 (unchanged in fix2); `backend/src/` diffs pre-existing (Q0026 etc.) |
| T-2 | Regression suite re-run | `cargo test --test grafana_provisioning_bug0009` | **PASS** — 6 passed / 0 failed |
| T-3 | Updated assertion vs DEC-0108 | Code review L146–164 | **PASS** — `current` present; `text==""`, `value==""`; no hardcoded `114`; ABS sort + no-alphabetical-only kept |
| T-4 | Supersession recorded | `decisions/DEC-0108.md` L8; `decisions.md` DEC-0068 Y1 L524 | **PASS** — both carry superseded-by-DEC-0108 annotation |
| T-5 | Static guard spot-check | python3 json (17 assertions) | **PASS** — 17/17 (sort:0, empty `current` shape, versions 2/3/2, no `114`, platform-health mirror SQL) |
| T-6 | BG/BH runtime oracles | Cycle 1 evidence (`handoffs/qa_report.md`) — no JSON changes in fix2 | **ACCEPTED** — first-run probes remain valid |
| T-7 | Duplicate-UID provisioning caveat | Cycle 1 classification | **ACCEPTED** — pre-existing, non-blocking at qa; operator note for V1 |
| T-8 | V1 operator smoke | GRAFANA_PROVISIONING_RELOAD + Full sync + 0-new-tx rerun | **DEFERRED** — verify-work |

### T-2 output

```
running 6 tests
test account_id_variable_uses_abs_balance_sort ... ok
(... 5 more ...)
test result: ok. 6 passed; 0 failed; 0 ignored
```

## Acceptance row status (qa-stage, post cycle 2)

| Row | qa-stage evidence | Status |
|-----|-------------------|--------|
| **BG** | Cycle 1 static guard + runtime oracles PASS; cycle 2 test contract aligned | **PASS** at qa — visual kiosk/direct + OIDC remain V1 |
| **BH** | Cycle 1 mirror-count oracle PASS (transactions=922); test suite green | **PASS** at qa — Full-sync + 0-new-tx sequencing remains V1 |

## Non-blocking notes (carry to verify-work)

- **Duplicate-UID provisioning:** three provider YAMLs scan overlapping paths → Grafana may refuse DB writes on re-provision. Pre-existing (not Q0027); live API already serves fixed content. Recommend follow-up bug to dedupe provider scan paths.
- **43 `ml_enhanced` computations stuck `running`:** out-of-scope per DEC-0108; recommend new backlog bug.

## Handoff

- **Next phase:** `/verify-work` (role: qa) — see `handoffs/qa_to_verify_work.md`
- **No return items**

`fresh_context_marker`: qa-20260610-bug0019-qa-cycle2-fresh  
`runtime_proof_id`: runtime-proof-qa-20260610-bug0019-002  
`phase_boundary`: qa → verify-work
