# PO to TL archive pack (2026-06-09)

- Rollover trigger: `PO_TO_TL_HOT_MAX_LINES=500, PO_TO_TL_HOT_MAX_SECTIONS=40`
- Source: `handoffs/po_to_tl.md`
- Archived units (oldest first, contiguous prefix): 1
- Retained units in hot file: 11
- First archived heading: `## intake-20260609-ui-audit-bugs — BUG-0016..0021 post-US-0020 UI audit intake (hot pointer)`
- Last archived heading: `## intake-20260609-ui-audit-bugs — BUG-0016..0021 post-US-0020 UI audit intake (hot pointer)`
- Verification tuple (mandatory):
  - archived_body_lines=35
  - retained_body_lines=500

---

## intake-20260609-ui-audit-bugs — BUG-0016..0021 post-US-0020 UI audit intake (hot pointer)

**From:** PO **To:** Tech Lead **Bugs:** BUG-0016, BUG-0017, BUG-0018, BUG-0019, BUG-0020, BUG-0021 **Run:** `intake-20260609-ui-audit` **Next:** `/discovery` on **BUG-0016** (P1 queue) **Source:** `handoffs/intake_evidence/ui-audit-20260609-local.json`

### Summary

Agent UI click-through on `http://localhost:18080` after US-0020 rebuild; six grouped bug intakes persisted with validated evidence bundles and acceptance rows **AX**–**BL**.

| Bug | Priority | Cluster | Intake evidence |
|-----|----------|---------|-----------------|
| **BUG-0016** | P1 | SPA deep-link 404 | `intake-20260609-spa-deep-link.json` |
| **BUG-0017** | P1 | Forecast recompute (audit, FK, ML, planning stale, empty flash) | `intake-20260609-forecast-recompute.json` |
| **BUG-0018** | P1 | Alert SQL `balance` ambiguous | `intake-20260609-alert-evaluation.json` |
| **BUG-0019** | P1 | Grafana cashflow zeros + sync entity counts | `intake-20260609-grafana-metrics.json` |
| **BUG-0020** | P2 | Subscription duplicates + uncategorized | `intake-20260609-subscriptions-list.json` |
| **BUG-0021** | P3 | CategoryFilter delay + Wealth Role column | `intake-20260609-frontend-ux.json` |

### Validators (PASS)

- `python scripts/intake_evidence_validate.py --file handoffs/intake_evidence/intake-20260609-*.json` (×6)
- `python scripts/bug_issue_validate.py --backlog docs/product/backlog.md --check-acceptance`

### Recommended discovery order

1. **BUG-0016** — SPA fallback (unblocks bookmarks/refresh)
2. **BUG-0017** — forecast pipeline (blocks planning/ML trust)
3. **BUG-0018** — alerts
4. **BUG-0019** — Grafana metrics (extends BUG-0013/0014 residual)
5. **BUG-0020** — subscriptions list quality
6. **BUG-0021** — P3 UX polish (defer optional)

`triad_hot_surface`: intake prepended; run `--rollover` + `--check` after this edit.

---

