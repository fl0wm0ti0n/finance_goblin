# QA findings — Q0021 / US-0017

**Story:** US-0017 — README living-doc expansion and troubleshooting  
**Quick task:** Q0021  
**Phase:** `/qa`  
**Orchestrator:** `auto-20260609-us0017-001`  
**Decision:** DEC-0070 (US-0017 extension)  
**QA agent:** fresh subagent (`qa-20260606-q0021-us0017`)  
**Date:** 2026-06-06  
**SECURITY_REVIEW:** 0 (skipped)

## Verdict

**PASS** — All blocking acceptance criteria (AC-1 through AC-5) satisfied. DEC-0070 US-0017 layout extension verified. Zero blocking findings. Hand off to **`/verify-work`** in a fresh subagent/chat.

## Scope

Doc-only DEC-0070 extension: omniflow smoke H3, troubleshooting H3, per-segment Product status maintenance hooks, operator user guide UG1, validator gate. No application code changes.

**Inputs reviewed:** `handoffs/dev_to_qa.md`, `sprints/quick/Q0021/summary.md`, `README.md`, `docs/developer/README.md`, `docs/engineering/runbook.md` (§ README maintenance), `docs/user-guides/US-0017.md`, `docs/product/acceptance.md` (US-0017), `decisions/DEC-0070.md`, `docs/engineering/decisions.md` (DEC-0070 US-0017 extension). No host `.env`, `.env_prod`, or operator secret values read.

## Test plan

| # | Check | Method | Result |
|---|-------|--------|--------|
| 1 | AC-1 omniflow smoke H3 | Inspect `README.md` `### Omniflow smoke (external profile)` + `docs/user-guides/US-0017.md` | **PASS** |
| 2 | AC-2 troubleshooting H3 | Inspect `README.md` `### Troubleshooting` + user guide symptom table | **PASS** |
| 3 | AC-3 Product status verify-only | Inspect `### Product status` bullets vs post-US-0016 closures | **PASS** (verify-only; US-0017 bullet deferred to `/release`) |
| 4 | AC-4 per-segment maintenance hooks | Inspect `docs/developer/README.md` Quality gates + runbook § README maintenance | **PASS** |
| 5 | AC-5 validator + split layout | `python3 scripts/validate_doc_profile.py --repo . --no-template-parity`; count root `##` H2 | **PASS** |
| 6 | DEC-0070 layout contract | Confirm H3 placement; no dedicated `## Troubleshooting` | **PASS** |
| 7 | Validator self-test | `python3 scripts/validate_doc_profile.py --self-test` | **PASS** (exit 0) |
| 8 | Triad hot surface | `python3 scripts/enforce-triad-hot-surface.py --check` | **PASS** (exit 0) |
| 9 | Secrets guard | Grep README + user guide for credential literals | **PASS** (placeholders only) |
| 10 | USER_GUIDE_MODE UG1 | Inspect `docs/user-guides/US-0017.md` required sections | **PASS** |
| 11 | User-visible metadata guard | `scripts/check-user-visible-metadata.py` | **SKIP** (entrypoint absent; repo precedent S0013/Q0020) |
| 12 | TEST_COMMAND baseline | `bash tests/run-tests.sh` | **SKIP** (doc-only; no code delta; not in US-0017 acceptance) |
| 13 | Runtime autopilot | Omniflow live probes | **N/A** (doc-only; operator smoke deferred to verify-work if applicable) |
| 14 | Security review gate | `SECURITY_REVIEW=0` | **SKIP** |

## Acceptance criteria results

| AC | Requirement | Result | Evidence |
|----|-------------|--------|----------|
| AC-1 | Examples include omniflow external-profile smoke (sync trigger, forecast recompute pointer, six `/analytics/{slug}` routes, exchange sanity) — not localhost-only | **PASS** | `README.md` lines 84–131: `OMNI=https://financegnome.omniflow.cc`, placeholder Traefik auth, gate sequence, sync trigger, `forecast/meta`, wealth/crypto probe, Grafana health, six-route table. `docs/user-guides/US-0017.md` mirrors smoke block |
| AC-2 | Limitations/Troubleshooting documents empty-Grafana vs ML-unavailable, `BACKEND_FRONTEND_DEPLOY` cadence, sync+recompute prerequisite | **PASS** | `README.md` lines 148–166: gate sequence, **6-row** symptom table, explicit ML-unavailable vs empty-panel distinction. User guide lines 62–75 duplicate table + distinction |
| AC-3 | Product status lists US-0015 and post-US-0016 closures (verify-only at execute; US-0017 bullet at release) | **PASS** (verify-only) | `### Product status` includes US-0015 (line 19), US-0016 (line 24), BUG-0013, US-0013–0014 closures. US-0017 append correctly deferred to `/release` per E3 |
| AC-4 | Developer README + runbook require Product status update per closed US/BUG in release segment | **PASS** | `docs/developer/README.md` lines 24–29: **each** closed id in **current release segment** (`Sxxxx`, `Qxxxx`, paired intake). Runbook § README maintenance lines 134–176: release segment definition, per-id release + refresh-context hooks |
| AC-5 | `validate_doc_profile.py` exits 0; split layout preserved (no `DEV_*` H2; H2 budget ≤ 8) | **PASS** | QA run `[DOC_PROFILE_VALIDATE_OK]` exit 0 with `--no-template-parity`. Six root H2s: Purpose, Quickstart, Examples, Limitations, Related documentation, Contributing — no new root H2; Troubleshooting is H3 under Limitations |

## DEC-0070 alignment

| Contract | Status | Evidence |
|----------|--------|----------|
| `### Omniflow smoke (external profile)` under `## Examples` | **PASS** | `README.md` line 84 |
| `### Troubleshooting` under `## Limitations` (not `## Troubleshooting`) | **PASS** | `README.md` line 148 |
| Per-segment Product status maintenance | **PASS** | Dev README + runbook § README maintenance |
| `--no-template-parity` validator gate | **PASS** | Exit 0 |
| No DEC-0081; extends DEC-0070 only | **PASS** | `docs/engineering/decisions.md` US-0017 extension table |

## Validator output

```
$ python3 scripts/validate_doc_profile.py --repo . --no-template-parity
[DOC_PROFILE_VALIDATE_OK]
EXIT_CODE=0

$ python3 scripts/validate_doc_profile.py --self-test
[DOC_PROFILE_SELF_TEST_OK]
EXIT_CODE=0
```

## Generated baseline test evidence (US-0066)

| Field | Value |
|-------|-------|
| `generated_test_stack_profile` | n/a (doc-only) |
| `generated_test_command` | _(not run — no code delta)_ |
| `generated_test_result` | skipped |
| `generated_test_output_ref` | n/a |
| `generated_test_paths_ref` | n/a |
| `generated_test_reason_code` | `DOC_ONLY_SCOPE` |

## Runtime QA evidence (US-0065)

| Field | Value |
|-------|-------|
| `runtime_startup_command` | _(not run)_ |
| `runtime_stack_profile` | n/a |
| `runtime_mode` | local (doc validation only) |
| `runtime_health_target` | `financegnome.omniflow.cc` — curl templates documented; live probes optional at verify-work |
| `runtime_health_result` | n/a |
| `runtime_log_summary` | n/a |
| `runtime_retry_count` | 0 |
| `runtime_retry_ledger` | [] |
| `runtime_final_verdict` | n/a |
| `runtime_reason_code` | `DOC_ONLY_SCOPE` |
| `runtime_evidence_refs` | `README.md#omniflow-smoke-external-profile`, `docs/user-guides/US-0017.md` |

**Environment label:** local (doc validation only; no omniflow runtime probes in QA).

## Findings summary

| ID | Severity | Finding | Blocking US-0017 |
|----|----------|---------|------------------|
| — | — | No findings | — |

**Blocking findings:** 0  
**Critical findings:** 0

## Handoff

**Next phase:** `/verify-work` in fresh subagent/chat  
**Stop reason:** QA_PASS — US-0017 AC-1 through AC-5 verified; no `handoffs/qa_to_dev.md` required

---

**Stop here.** Do not begin `/verify-work` in this subagent.
