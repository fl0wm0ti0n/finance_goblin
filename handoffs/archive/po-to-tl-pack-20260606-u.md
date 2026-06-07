# PO to TL archive pack (2026-06-06)

- Rollover trigger: `PO_TO_TL_HOT_MAX_LINES=500, PO_TO_TL_HOT_MAX_SECTIONS=40`
- Source: `handoffs/po_to_tl.md`
- Archived units (oldest first, contiguous prefix): 1
- Retained units in hot file: 10
- First archived heading: `## intake-20260606-bug0013-us0017 — Omniflow analytics regression + README expansion (hot pointer)`
- Last archived heading: `## intake-20260606-bug0013-us0017 — Omniflow analytics regression + README expansion (hot pointer)`
- Verification tuple (mandatory):
  - archived_body_lines=50
  - retained_body_lines=499

---

## intake-20260606-bug0013-us0017 — Omniflow analytics regression + README expansion (hot pointer)

**From:** PO | **Work items:** BUG-0013 (P0), US-0017 (P2) | **Run:** `intake-20260606-omniflow-regression-readme` | **Next:** `/discovery`  
**Evidence:** `handoffs/intake_evidence/intake-20260606-omniflow-regression-readme.json` | **Research:** [R-0076](docs/engineering/research.md#r-0076--omniflow-analytics-regression-hypotheses-post-us-0015)

### Summary

Operator report on `financegnome.omniflow.cc` post-US-0015: Grafana analytics mostly zeros/empty (cashflow scarcity chart, subscriptions price changes, portfolio crypto/FX/performance, forecast 0 €, budgets MTD −€150K), Grafana `Failed to fetch` on ds/query, crypto not parsed. Separate ask: keep root README up-to-date for devs and users beyond US-0016 baseline.

### Decomposition decision

| Work item | Priority | Scope |
|-----------|----------|-------|
| **BUG-0013** | P0 | Six sub-defects AI–AN: cashflow/forecast zeros, subscriptions price changes, portfolio crypto/FX/performance, budgets MTD, Grafana fetch, crypto pipeline |
| **US-0017** | P2 | README Examples/Troubleshooting expansion + release/refresh-context maintenance hook |

**Split axis:** user-value boundary (data correctness vs documentation). **Not** reopening BUG-0009/0010 without discovery evidence.

### Intake evidence

- `intake_run_id`: `intake-20260606-omniflow-regression-readme`
- `selected_pack`: `small-intake-pack`
- `asked_topics`: outcome_success_criteria, impacted_components, constraints_compatibility_risks, required_tests_acceptance_checks, done_definition
- `missing_topics`: _(none)_
- `assumptions_confirmed`: `(none)`
- `bug_routing_guard`: INTAKE_BUG_ROUTING_OK (exit 0, kind=story)

### Discovery priorities (BUG-0013)

1. Confirm **BACKEND_FRONTEND_DEPLOY** + Full sync + recompute baseline before code attribution ([R-0076](docs/engineering/research.md#r-0076--omniflow-analytics-regression-hypotheses-post-us-0015) §1)
2. Probe `POST /analytics/grafana/api/ds/query` per dashboard with `$account_id=114` vs default
3. Trace crypto: exchange sync → positions → `net_worth_snapshots` → portfolio panel SQL
4. Budgets MTD SQL: plan currency, join keys, MTD window
5. Annotation/ds/query proxy path for AM sub-defect

### US-0017 discovery note

Doc-only scope — may run parallel track. Expand Troubleshooting with deploy+recompute prerequisite; bind Product status updates to release checklist.

### Acceptance traceability

- **BUG-0013:** 1 row, 6 sub-criteria AI–AN in `docs/product/acceptance.md`
- **US-0017:** 5 rows in `docs/product/acceptance.md`

### Out of scope

MetaMask extension console noise; US-0013 ML production enablement.

---

