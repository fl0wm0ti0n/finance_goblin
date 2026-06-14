# PO to TL archive pack (2026-06-13)

- Rollover trigger: `PO_TO_TL_HOT_MAX_LINES=650, PO_TO_TL_HOT_MAX_SECTIONS=60`
- Source: `handoffs/po_to_tl.md`
- Archived units (oldest first, contiguous prefix): 2
- Retained units in hot file: 60
- First archived heading: `## Discovery summary`
- Last archived heading: `## Hypothesis verdicts`
- Verification tuple (mandatory):
  - archived_body_lines=12
  - retained_body_lines=506

---

## Discovery summary

Code audit + live API + browser probe on **localhost:18080** resolve the three intake hypotheses. Operator *immer ausgegraut* aligns with **sole globally active plan** (expected **DEC-0082** disable) plus **insufficient UX copy (BS)** — not a **Q0031** selector regression on the current bundle. Multi-plan **BR** passes locally; **omniflow** **FRONTEND_DEPLOY** remains the open **BM** verification path.

## Hypothesis verdicts

| ID | Verdict | Key evidence |
|----|---------|--------------|
| **H1** sole-plan UX | **CONFIRMED (BS)** | 1 active plan → `isDeleteDisabled=true`; tooltip *Set another plan active…* — no create-second-plan steps |
| **H2** omniflow pre-Q0031 | **LIKELY (BR on omniflow)** | Q0031/Q0032 release notes defer **FRONTEND_DEPLOY**; omniflow not probed |
| **H3** multi-plan BM regression | **RULED OUT (localhost)** | 2 plans; select `discovery-scenario` (non-active) → delete enabled; vitest 8/8 |

