# PO to TL archive pack (2026-06-06)

- Rollover trigger: `PO_TO_TL_HOT_MAX_LINES=500, PO_TO_TL_HOT_MAX_SECTIONS=40`
- Source: `handoffs/po_to_tl.md`
- Archived units (oldest first, contiguous prefix): 1
- Retained units in hot file: 8
- First archived heading: `## discovery-20260609-us0017 — US-0017 README living-doc expansion discovery (hot pointer)`
- Last archived heading: `## discovery-20260609-us0017 — US-0017 README living-doc expansion discovery (hot pointer)`
- Verification tuple (mandatory):
  - archived_body_lines=73
  - retained_body_lines=448

---

## discovery-20260609-us0017 — US-0017 README living-doc expansion discovery (hot pointer)

**From:** PO  
**To:** Tech Lead  
**Date:** 2026-06-09  
**Story:** US-0017  
**Orchestrator run:** `auto-20260609-us0017-001`  
**Next phase:** `/research`

### Summary

Doc-only story: expand root README **Examples** and **Troubleshooting** from Q0020/BUG-0013 operator smoke paths; strengthen per-segment Product status maintenance in developer README + runbook. No analytics code. **BUG-0013 DONE** — smoke checklist in `sprints/quick/Q0020/uat.md` and runbook §23 are canonical copy sources.

### Gap analysis (README vs acceptance)

| AC row | Current state | Gap |
|--------|---------------|-----|
| Examples — omniflow smoke | Localhost curls only; route table without host URLs | Add `### Omniflow smoke (external profile)` with Traefik host commands |
| Troubleshooting | ML-unavailable bullet in Limitations only | Add `### Troubleshooting` under Limitations: gates, symptom table, ML-off vs data-missing |
| Product status | US-0015, BUG-0013, US-0013–0016 listed (refresh-context) | Verify at execute; no new ids expected |
| Maintenance contract | US-0016 hooks in runbook + developer README | Tighten "each closed US/BUG **in segment**" wording |
| Validator / layout | 6 H2s; `validate_doc_profile` PASS | Preserve; no new root H2 |

### Recommended doc sections

**`## Examples` → `### Omniflow smoke (external profile)`**

- `BACKEND_FRONTEND_DEPLOY` one-liner + runbook §23 link
- `curl` health + `POST /api/v1/sync/trigger` against `https://financegnome.omniflow.cc` (basic-auth placeholder)
- Six `/analytics/{slug}` URLs (table or list)
- Exchange sanity: `GET /api/v1/wealth` → `crypto.subtotal_eur`
- Recompute pointer: Full sync from SPA + `GET /api/v1/forecast/meta` `last_computed_at`

**`## Limitations` → `### Troubleshooting`**

| Symptom | Likely cause | Action |
|---------|--------------|--------|
| Flat **0 €** panels post-deploy | Gates skipped / stale image | BACKEND_FRONTEND_DEPLOY → GRAFANA_PROVISIONING_RELOAD → FULL_FIREFLY_SYNC + recompute |
| Budgets **−€150K** MTD | Pre-DEC-0079 artifact | Deploy + Grafana reload |
| Crypto **€0** | Pre-DEC-0080 or exchanges-only sync | Deploy + Full sync + exchange sync |
| Forecast **0 €** on defaults | Wrong account or no recompute | Full sync; verify acct **114** |
| **ML unavailable** banner | ML overlay off (US-0013) | Expected — not data-missing |
| Browser **Failed to fetch** | Embed/WS edge | curl ds/query 200; Traefik session; no dashboard Save |

**Maintenance:** `docs/developer/README.md` Quality gates + runbook § README maintenance — per-segment bullet for each closed US/BUG.

### Discovery decomposition evidence

- Feature/workflow count: 2 doc subsections + 2 maintenance wording edits (low — **single story retained**)
- Cross-cutting impact: `README.md`, `docs/developer/README.md`, runbook § README maintenance only
- Acceptance breadth: 5 AC unchanged (`docs/product/acceptance.md` § US-0017)
- Risk surface: H2 budget (mitigated by H3 placement); basic-auth placeholder hygiene (no secrets)

### Research pointers (lightweight `/research`)

- Confirm omniflow `curl -u '<user>:<pass>'` placeholder pattern (consistent with runbook § Omniflow)
- Confirm `### Troubleshooting` under Limitations vs expanded bullets — R-0067 H3 precedent
- **No new R-xxxx required** — [R-0066](docs/engineering/research.md#r-0066--root-readme-split-layout-and-living-doc-maintenance), [R-0067](docs/engineering/research.md#r-0067--us-0016-root-readme-research-template-parity-product-status-maintenance-hooks) sufficient

### Artifacts updated

- `docs/product/backlog.md#US-0017`
- `docs/product/vision.md` (US-0017 discovery 2026-06-09)
- `handoffs/resume_brief.md`

### Recommended next steps

1. `/research` — formalize curl templates + Troubleshooting H3 contract (doc-only; skip web research unless norm conflict)
2. `/architecture` — confirm DEC-0070 extension for per-segment maintenance wording if needed
3. `/sprint-plan` — expect small task count (README + 2 doc shards)

---

