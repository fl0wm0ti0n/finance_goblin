# QA -> Dev Handoff — Q0027 / BUG-0019

## Status

- Result: **FAIL** (return to dev — single scoped remediation item)
- Sprint: `Q0027` (`/quick`)
- Bug: `BUG-0019`
- Orchestrator: `auto-20260610-bug0019`
- Date: 2026-06-10

## Blocking reason

- Primary reason code: `QA_TEST_FAILED`
- Summary: Existing regression suite `backend/tests/grafana_provisioning_bug0009.rs`
  fails **1/6** — `account_id_variable_uses_abs_balance_sort` enforces the **DEC-0068**
  (BUG-0009) contract *"omit saved `current` on `account_id`"*, which **DEC-0108**
  (BUG-0019 CA fix) intentionally reverses by adding `current`. The implementation
  itself is verified correct (independent static guard all-PASS; runtime oracles for
  BG/BH all green; live Grafana already serves the fixed dashboards) — the conflict is
  between the new decision and the stale enforcing test, and must be resolved before
  verify-work/release.

## What QA verified as GOOD (do not re-touch)

- The three dashboard JSON edits are correct per DEC-0108 — independent static guard
  16/16 PASS; provisioning-only diff confirmed (13/5/4 lines).
- BH oracle: new panel 2 SQL → `transactions=922` mirror (old `records_synced`=0 — the
  exact defect); BG oracle: variable SQL first row = account 114; 731/731 non-zero
  series for latest baseline success; API 25 points non-zero from Jul 2026.
- Duplicate-UID provisioning warning: **pre-existing** (3 provider YAMLs unchanged since
  first commit; overlapping scan paths), **not introduced by Q0027**, **not blocking**
  BG/BH here — operator note for V1 + recommended follow-up bug (dedupe provider paths).

## Required remediation

1. Update `account_id_variable_uses_abs_balance_sort` in
   `backend/tests/grafana_provisioning_bug0009.rs`: replace the
   `account_var.get("current").is_none()` assertion (L144–147) with assertions that
   `current` is **present**, matches the DEC-0108 empty text/value shape
   (`{"text": "", "value": ""}` — i.e. no saved concrete value), and that no hardcoded
   account id (`114`) appears. Keep all other assertions (ABS sort, no
   alphabetical-only) unchanged. Test files are not in the DEC-0108 forbidden list.
2. Record supersession: one-line amendment in `decisions/DEC-0108.md` (and/or the
   DEC-0068 row in `docs/engineering/decisions.md`) noting DEC-0108 supersedes the
   DEC-0068 "omit `current`" clause for the `account_id` variable.
3. Re-run `cargo test --test grafana_provisioning_bug0009` → **6/6 PASS**; record in
   `sprints/quick/Q0027/progress.md`.

## Re-run criteria

- Update `handoffs/dev_to_qa.md` (Q0027 section) with remediation evidence.
- Re-run `/qa` in a fresh subagent; on PASS, qa hands off to `/verify-work` (V1 operator
  gates unchanged: GRAFANA_PROVISIONING_RELOAD + Full sync + 0-new-tx rerun).

## Evidence refs

- `handoffs/qa_report.md` (full findings, probe outputs)
- `sprints/quick/Q0027/progress.md`
- `decisions/DEC-0108.md`; `docs/engineering/decisions.md` § DEC-0068
- `backend/tests/grafana_provisioning_bug0009.rs` L123–149

`fresh_context_marker`: qa-20260610-bug0019-qa-fresh  
`runtime_proof_id`: runtime-proof-qa-20260610-bug0019-001  
`phase_boundary`: qa → execute (return)

**Next:** `/execute` (role: dev) in a fresh subagent/chat. Do not begin the fix in this subagent.
