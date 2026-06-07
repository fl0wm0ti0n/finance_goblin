# PO to TL archive pack (2026-06-06)

- Rollover trigger: `PO_TO_TL_HOT_MAX_LINES=500, PO_TO_TL_HOT_MAX_SECTIONS=40`
- Source: `handoffs/po_to_tl.md`
- Archived units (oldest first, contiguous prefix): 1
- Retained units in hot file: 8
- First archived heading: `## architecture-20260609-us0017 — US-0017 README living-doc expansion architecture (hot pointer)`
- Last archived heading: `## architecture-20260609-us0017 — US-0017 README living-doc expansion architecture (hot pointer)`
- Verification tuple (mandatory):
  - archived_body_lines=55
  - retained_body_lines=448

---

## architecture-20260609-us0017 — US-0017 README living-doc expansion architecture (hot pointer)

**From:** Tech Lead  
**To:** PO / Sprint-plan  
**Date:** 2026-06-09  
**Story:** US-0017  
**Orchestrator run:** `auto-20260609-us0017-001`  
**Next phase:** `/sprint-plan` or `/quick`

### Summary

Doc-only story formalized as **DEC-0070 extension** (no DEC-0081). Two H3 additions under existing H2s; per-segment Product status maintenance wording; validator gate unchanged. No application code.

### Decision

| ID | Contract |
|----|----------|
| **DEC-0070** (US-0017 extension) | `### Omniflow smoke (external profile)` under Examples; `### Troubleshooting` under Limitations; per-segment maintenance hooks; `validate_doc_profile --no-template-parity` exit 0 after execute |

### Execute scope (frozen)

| Task | Surface | AC |
|------|---------|-----|
| **E1** | `README.md` § Examples → H3 Omniflow smoke | AC-1 |
| **E2** | `README.md` § Limitations → H3 Troubleshooting | AC-2 |
| **E3** | `README.md` § Product status verify | AC-3 (verify-only) |
| **E4** | `docs/developer/README.md` per-segment wording | AC-4 |
| **E5** | `docs/engineering/runbook.md` § README maintenance | AC-4 |
| **E6** | `validate_doc_profile --no-template-parity` | AC-5 |

### Operator content sources (copy, do not duplicate inline)

- Omniflow curls: R-0078 §2 (Traefik `-u` placeholders — no secrets)
- Symptom table: R-0078 §3 / Q0020 uat.md
- Gates: BACKEND_FRONTEND_DEPLOY → GRAFANA_PROVISIONING_RELOAD → FULL_FIREFLY_SYNC + recompute

### Out of scope

- Analytics code; dedicated `## Troubleshooting` H2; full runbook §23 inline; DEC-0081

### Artifacts updated

- `docs/engineering/architecture.md` § **US-0017**
- `docs/engineering/decisions.md` — DEC-0070 US-0017 extension accepted
- `decisions/DEC-0070.md` — US-0017 extension subsection
- `docs/product/backlog.md#US-0017`
- `handoffs/resume_brief.md`
- `docs/engineering/state.md` — architecture checkpoint

`triad_hot_surface`: US-0017 architecture appended; --rollover + --check PASS (2026-06-09; units=7,1)

**Recommended sprint:** `/quick` (~5–6 tasks, doc-only)

---

