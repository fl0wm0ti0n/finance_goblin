# PO to TL archive pack (2026-06-06)

- Rollover trigger: `PO_TO_TL_HOT_MAX_LINES=500, PO_TO_TL_HOT_MAX_SECTIONS=40`
- Source: `handoffs/po_to_tl.md`
- Archived units (oldest first, contiguous prefix): 2
- Retained units in hot file: 8
- First archived heading: `## research-20260608-us0016 — US-0016 root README research → architecture`
- Last archived heading: `## discovery-20260608-us0016 — US-0016 root README discovery → research`
- Verification tuple (mandatory):
  - archived_body_lines=96
  - retained_body_lines=471

---

## research-20260608-us0016 — US-0016 root README research → architecture

**From:** Tech Lead  
**To:** PO / Architecture  
**Date:** 2026-06-08  
**Story:** US-0016  
**Orchestrator run:** auto-20260606-us0016-001  
**Next phase:** `/architecture`

### Summary

Web + code research completed for US-0016 discovery open questions. Added **[R-0067](docs/engineering/research.md#r-0067--us-0016-root-readme-research-template-parity-product-status-maintenance-hooks)**; extended **R-0066** status pointer. No host `.env` or secrets read. Recommend architecture record **DEC-0070** to lock template-flag posture, Product status placement, and maintenance hook wording before sprint-plan/execute.

### Discovery open questions — research resolution

| Question | Resolution (per research) |
|----------|---------------------------|
| **Template parity** | `template/` tree **absent** (zero files). Partial stub `template/README.md` **rejected** — parity also requires `template/docs/developer/README.md`. Use **`--no-template-parity`** in CI/release gate until full installer template tree ships; AC-6 satisfied vacuously until then. |
| **Product status placement** | Validator H2 budget counts **required USER_* H2s only** — extra H2s do not consume budget (`count_profile_root_h2s`). Recommend **`### Product status` under `## Purpose`** (capped 8 bullets, reverse-chronological); reject dedicated `## Product status` H2 and reject nesting under Related documentation. |
| **Maintenance binding** | Phase-boundary hooks (not per-commit): **release** — after backlog reconciliation, append Product status bullet per closed US/BUG, then `validate_doc_profile --no-template-parity`; **refresh-context** — verify/update Product status when items closed since prior refresh; **runbook** new § `README maintenance (US-0016)` + dev shard pointer. Exact wording in R-0067 §3. |

### Key findings by topic

| Topic | Research | Recommendation for architecture |
|-------|----------|--------------------------------|
| Split layout | R-0066 + R-0067 | Unchanged — 5 user H2s + `## Contributing`; dev shard unchanged except workflow pointer |
| Template CI | R-0067 §1 | `--no-template-parity` until `template/` exists; DEC-0070 gate to drop flag when tree lands |
| Living doc | R-0067 §2–3 | `### Product status` under Purpose; release + refresh-context checklist hooks; runbook subsection |
| External practice | R-0067 sources | Concise root README; link deeper docs; update on meaningful releases — aligns with manual phase-boundary cadence |

### Risks (carried forward)

1. **Stale Product status** — mitigated by release fail-closed validator + refresh-context verify step (R-0067 §3).
2. **Template flag drift** — mitigated by DEC-0070 + runbook note to remove `--no-template-parity` when template tree merges.
3. **Scope creep** — capped 8-bullet Product status + backlog link (R-0066 anti-patterns).

### Evidence

- Research: [R-0067](docs/engineering/research.md#r-0067--us-0016-root-readme-research-template-parity-product-status-maintenance-hooks), [R-0066](docs/engineering/research.md#r-0066--root-readme-split-layout-and-living-doc-maintenance)
- Code: `scripts/doc_profile_lib.py`, `scripts/validate_doc_profile.py`
- Acceptance: `docs/product/acceptance.md` § US-0016 (6 rows, unchanged)
- Prior handoff: `#discovery-20260608-us0016`

---

## discovery-20260608-us0016 — US-0016 root README discovery → research

**From:** PO  
**To:** Tech Lead  
**Date:** 2026-06-08  
**Story:** US-0016  
**Orchestrator run:** auto-20260606-us0016-001  
**Next phase:** `/research`

### Summary

Discovery refined the **root `README.md`** entry-point as a DEC-0059 split-layout living document. No UI surface; the "design" is the validator-enforced H2 structure (`scripts/doc_profile_lib.py` / `scripts/validate_doc_profile.py`) under active profile `DOC_AUDIENCE_PROFILE=both` / `DOC_DETAIL_LEVEL=balanced`. Vision updated with the information-architecture reference and discovery notes; backlog US-0016 block refined with split-layout target, optional-mode crosslinks, and template-parity posture. Acceptance unchanged (6 rows). Builds on intake handoff `intake-20260607-us0016` and research pointer **R-0066**.

### Confirmed structure (carry to research/architecture)

| Item | Resolution |
|------|------------|
| Root user H2s (required) | `Purpose`, `Quickstart`, `Examples`, `Limitations`, `Related documentation` (exact titles) |
| Contributing pointer | single `## Contributing` → `docs/developer/README.md`; **no `DEV_*` H2** in root |
| H2 budget | profile-scoped root H2 count ≤ **8** for `(both, balanced)`; 5 user H2s fit |
| Developer shard | `DEV_PREREQS`, `DEV_WORKFLOW`, `DEV_QUALITY_GATES`, `DEV_ARCHITECTURE` stay in `docs/developer/README.md` |
| Optional-mode crosslinks | `USER_GUIDE_MODE=1` → `docs/user-guides`; `SPEC_PACK_MODE=1` → engineering/spec paths |
| Content sources | product value, compose profiles in `.env.example`, `docs/engineering/runbook.md` (R-0066) |
| Living-doc cadence | curated **Product status** at release + refresh-context when US/BUG closes; validator at release gate |

### Open questions for `/research`

1. **Template parity** — `template/` tree absent today; choose stub `template/README.md` vs CI `--no-template-parity` (AC-6 conditional).
2. **Product status placement** — `###` subsection (budget-safe, discovery lean) vs dedicated `## Product status` H2 (consumes H2 budget) vs nested under Related documentation.
3. **Maintenance binding** — exact release/refresh-context checklist hook wording to enforce the living-doc promise.

### Risks (from intake, still open)

1. Scope creep — README duplicating backlog; mitigate with capped Product status bullets + links.
2. Template parity gap — no `template/README.md` today; CI flag vs stub decision.
3. Stale status — bind upkeep to `validate_doc_profile` + release checklist.

### Recommended next steps

1. `/research` — resolve the three open questions; extend/confirm R-0066.
2. `/architecture` — DEC for Product status placement + maintenance hook + template-parity decision.

### Evidence

- Vision: `docs/product/vision.md` (US-0016 reference + Discovery notes 2026-06-08)
- Backlog: `docs/product/backlog.md` US-0016 (#### Discovery notes 2026-06-08)
- Research pointer: [R-0066](docs/engineering/research.md#r-0066--root-readme-split-layout-and-living-doc-maintenance)
- Acceptance: `docs/product/acceptance.md` § US-0016 (6 rows, unchanged)

---

