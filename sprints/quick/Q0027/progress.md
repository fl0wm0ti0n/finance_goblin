# Q0027 progress

**Sprint:** Q0027 (BUG-0019)  
**Status:** EXECUTED (fix cycle 2) — qa re-run next (V1 remains verify-work scope)  
**Last updated:** 2026-06-10 (execute fix cycle 2, `auto-20260610-bug0019`)  
**Orchestrator:** `auto-20260610-bug0019`

## Fix cycle 2 (execute, 2026-06-10T20:53Z) — QA return remediation

QA verdict FAIL (`handoffs/qa_to_dev.md`) — single scoped item: stale DEC-0068
regression test conflicted with DEC-0108 `current` contract. Remediation:

| Item | Action | Result |
|------|--------|--------|
| 1 | `backend/tests/grafana_provisioning_bug0009.rs` `account_id_variable_uses_abs_balance_sort`: replaced `current().is_none()` assertion with: `current` **present**, `text == ""`, `value == ""` (DEC-0108 empty shape), no hardcoded `114` in the variable; ABS-sort + no-alphabetical assertions unchanged; file header doc comment notes the DEC-0108 amendment | done |
| 2 | Supersession recorded: `decisions/DEC-0108.md` header gains `**Supersedes:**` line (existing convention, cf. DEC-0104/DEC-0057); `docs/engineering/decisions.md` § DEC-0068 Y1 omit-`current` bullet annotated superseded-by-DEC-0108. (`decisions/DEC-0068.md` does not exist as a standalone file — DEC-0068 canonical record lives in `docs/engineering/decisions.md`) | done |
| 3 | `cargo test --test grafana_provisioning_bug0009` → **6 passed / 0 failed** (6/6 PASS). Static guard re-run (python json): 12/12 PASS (sort:0 + empty `current` shape + versions 2/3/2 + no `114` + platform-health mirror SQL) | done |

No dashboard JSON changed in fix cycle 2 (QA verified implementation correct).

## Task status

| ID | Status | Priority | Notes |
|----|--------|----------|-------|
| CA1 | done | P0 | cashflow.json `$account_id` sort:0 + `current` (empty text/value shape — Grafana resolves to first option on load; no hardcoded 114) + version 1→2 |
| CA2 | done | P0 | cashflow.json panels 1–3: all 4 latest-success subqueries (panel 1 has 2 targets) gained `AND model_kind = 'baseline'` |
| CA3 | done | P0 | forecast-horizons.json `$account_id` sort:0 + `current` + version 2→3; no panel rawSql changed |
| CB1 | done | P0 | platform-health.json panel 2 rawSql → DEC-0108 mirror COUNT(*) UNION ALL LEFT JOIN sync_cursors; `records_synced` dropped (not relabeled); version 1→2 |
| G1 | done | P0 | static assertions PASS (python json fallback — jq not installed); results below |
| V1 | open | P0 | verify-work after GRAFANA_PROVISIONING_RELOAD + Full sync + 0-new-tx rerun |

## Execute order

`(CA1 → CA2) ∥ CA3 ∥ CB1 → G1 → Grafana restart → V1`

## Operator gates

| Gate | Status |
|------|--------|
| **GRAFANA_PROVISIONING_RELOAD** (`docker compose restart grafana`) | pending |
| **FULL_FIREFLY_SYNC_PLUS_INCREMENTAL_RERUN** | pending |

## G1 static guard results

Recorded 2026-06-10 (execute). `jq` is not installed in the dev environment —
assertions run via python3 `json` fallback (equivalent checks; per
plan-verify dev note).

| Check | Result |
|-------|--------|
| cashflow `account_id` sort:0 + current | **PASS** — `sort=0`, `current={'text': '', 'value': ''}` (non-null, forecast_variant text/value shape) |
| forecast-horizons `account_id` sort:0 + current | **PASS** — `sort=0`, `current` non-null, same shape |
| cashflow panels 1–3 `model_kind = 'baseline'` | **PASS** — asserted per-panel (plan-verify note 2): panel 1 = 2/2 targets, panel 2 = 1/1, panel 3 = 1/1 (4 subqueries total, not 3 — panel 1 has two targets) |
| platform-health panel 2 six mirror tables | **PASS** — all six entities (`transactions, accounts, categories, budgets, tags, piggy_banks`) + `LEFT JOIN sync_cursors`; no bare `records_synced` in SQL |
| JSON parse all three files | **PASS** — python `json.load` exit 0 each; versions: cashflow 1→2, forecast-horizons 2→3, platform-health 1→2 (vs HEAD, working-tree diff per plan-verify note 3 — edits uncommitted) |
| `git diff --stat` provisioning-only | **PASS (scoped)** — execute-phase code edits touch only the three dashboard JSONs (13/5/4 changed lines); working tree also carries pre-existing uncommitted artifact/doc/backend changes from earlier sprints, not from this phase |
| No hardcoded account `114` | **PASS** — string `114` absent from all three files |

### Local Grafana reload sanity check (not V1)

`docker restart finance_goblin-grafana-1` performed locally: provisioning
scanned all three edited files with **no parse/provisioning errors**.
**Pre-existing environment warning** (not introduced by Q0027): duplicate
dashboard UIDs across two providers (`Analytics` + `Flow Finance AI` both scan
overlapping paths) → Grafana logs `"the same UID is used more than once"` and
`"Not saving new dashboard due to restricted database access"`, i.e. the
provisioners refuse DB writes while duplicates exist. **V1/operator must
resolve or account for this** (e.g. dedupe provider paths) or the edited
dashboards may not be re-saved into Grafana's DB despite the version bump.

## Next phase

`/qa` (role: qa) — handoff `handoffs/dev_to_qa.md` (Q0027/BUG-0019 section)
