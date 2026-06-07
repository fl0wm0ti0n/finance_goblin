# QA findings — S0013 / US-0016

**Story:** US-0016 — Root README living documentation  
**Sprint:** S0013  
**Phase:** `/qa`  
**Orchestrator:** `auto-20260606-us0016-001`  
**Decision:** DEC-0070  
**QA agent:** fresh subagent (`qa-20260608-s0013-us0016`)  
**Date:** 2026-06-08  
**SECURITY_REVIEW:** 0 (skipped)

## Verdict

**PASS** — All blocking acceptance criteria (AC-1 through AC-5) satisfied. No blocking findings for US-0016. Hand off to `/verify-work`.

## Test plan

| # | Check | Method | Result |
|---|-------|--------|--------|
| 1 | AC-1 split layout + real content | Inspect `README.md` H2 structure and section bodies | **PASS** |
| 2 | AC-2 Contributing pointer, no DEV_* H2 | Grep `README.md` for `##` headings and `DEV_` | **PASS** |
| 3 | AC-3 related docs + compose commands | Inspect `## Related documentation` and Quickstart table | **PASS** |
| 4 | AC-4 doc profile validator | `python3 scripts/validate_doc_profile.py --repo . --no-template-parity` | **PASS** (exit 0) |
| 5 | AC-5 runbook maintenance + dev pointer | Inspect runbook § README maintenance; `docs/developer/README.md` | **PASS** |
| 6 | Validator self-test | `python3 scripts/validate_doc_profile.py --self-test` | **PASS** (exit 0) |
| 7 | CI gate wiring | Inspect `tests/run-tests.sh` § doc validation | **PASS** |
| 8 | TEST_COMMAND baseline | `bash tests/run-tests.sh` | **FAIL** (pre-existing; see note) |
| 9 | User-visible metadata guard | `python scripts/check-user-visible-metadata.py` | **SKIP** (entrypoint absent) |
| 10 | Security review gate | `SECURITY_REVIEW=0` | **SKIP** |

## Acceptance criteria results

| AC | Requirement | Result | Evidence |
|----|-------------|--------|----------|
| AC-1 | DEC-0059 user H2s (Purpose, Quickstart, Examples, Limitations, Related documentation) with Flow Finance AI content | **PASS** | `README.md` lines 6–116: six user H2s populated with project-specific compose profiles, API examples, analytics routes, and real limitations (not stub text) |
| AC-2 | `## Contributing` → `docs/developer/README.md`; no forbidden `DEV_*` H2 in root | **PASS** | `## Contributing` at line 118 links to `docs/developer/README.md`; zero `## DEV_*` headings |
| AC-3 | Links `docs/user-guides/`, `docs/engineering/runbook.md`; documents minimal / bundled-firefly / external omniflow compose commands | **PASS** | Related documentation table (lines 97–103); Quickstart profile table (lines 32–36) and compose blocks (lines 107–116) |
| AC-4 | `validate_doc_profile.py --repo .` exits 0 with current profile flags | **PASS** | QA run: `[DOC_PROFILE_VALIDATE_OK]` exit 0 with `--no-template-parity`; wired in `tests/run-tests.sh` line 73 |
| AC-5 | Runbook documents README maintenance cadence (release/refresh-context Product status updates; validator at release gate) | **PASS** | `docs/engineering/runbook.md` § README maintenance (US-0016): validator command table, `/release` steps 1–3, `/refresh-context` steps 1–2 |
| AC-6 | `template/README.md` parity when `template/` exists | **DEFERRED** | Vacuous — `template/` absent; `--no-template-parity` retained per DEC-0070 |

## Validator output

```
$ python3 scripts/validate_doc_profile.py --repo . --no-template-parity
[DOC_PROFILE_VALIDATE_OK]
EXIT_CODE=0

$ python3 scripts/validate_doc_profile.py --self-test
[DOC_PROFILE_SELF_TEST_OK]
EXIT_CODE=0
```

## Supplementary checks

### TEST_COMMAND (`bash tests/run-tests.sh`)

**Result:** FAIL (exit 101) — pre-existing regression, **not introduced by US-0016**.

```
test wealth_uses_in_app_portfolio_analytics ... FAILED
assertion failed: !WEALTH_TSX.contains("target=\"_blank\"")
```

Failure occurs at `product_routes_regression` (line 55 of `tests/run-tests.sh`) before the doc-profile step (line 72). US-0016 changed documentation surfaces only (`README.md`, runbook, dev shard, `tests/run-tests.sh` gate addition, `installer.py` support). **Not blocking** US-0016 story acceptance; tracked as informational for push-eligibility (`TEST_FAILED`).

### User-visible metadata guard (US-0071)

**Result:** SKIP — `scripts/check-user-visible-metadata.py` entrypoint absent. Not in US-0016 acceptance scope; no blocking finding recorded.

### Generated baseline test evidence (US-0066)

| Field | Value |
|-------|-------|
| `generated_test_stack_profile` | rust+node (monorepo) |
| `generated_test_command` | `bash tests/run-tests.sh` |
| `generated_test_result` | fail |
| `generated_test_output_ref` | QA run 2026-06-08 — `product_routes_regression` wealth_uses_in_app_portfolio_analytics |
| `generated_test_paths_ref` | `tests/run-tests.sh`, `backend/tests/product_routes_regression.rs:44` |
| `generated_test_reason_code` | `TEST_FAILED` (pre-existing, out of US-0016 scope) |

### Runtime autopilot (US-0065)

**Result:** N/A — documentation-only story; no runtime startup required.

## Findings summary

| ID | Severity | Finding | Blocking US-0016 |
|----|----------|---------|------------------|
| QA-001 | informational | `TEST_COMMAND` fails on pre-existing `wealth_uses_in_app_portfolio_analytics` regression | No |
| QA-002 | informational | Metadata checker script absent (US-0071 guard skipped) | No |

**Blocking findings:** 0  
**Critical findings:** 0

## Handoff

**Next phase:** `/verify-work` in fresh subagent/chat  
**Stop reason:** QA_PASS — US-0016 AC-1 through AC-5 verified; no `handoffs/qa_to_dev.md` required
