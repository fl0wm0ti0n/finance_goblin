# PO to TL archive pack (2026-06-13)

- Rollover trigger: `PO_TO_TL_HOT_MAX_LINES=500, PO_TO_TL_HOT_MAX_SECTIONS=40`
- Source: `handoffs/po_to_tl.md`
- Archived units (oldest first, contiguous prefix): 5
- Retained units in hot file: 40
- First archived heading: `## Operator report`
- Last archived heading: `## Related`
- Verification tuple (mandatory):
  - archived_body_lines=33
  - retained_body_lines=357

---

## Operator report

> „bug ich kann immer noch nicht pläne entfernen — Delete plan ist immer ausgegraut“

## Live probe (2026-06-13)

| Check | Result |
|-------|--------|
| `GET /api/v1/plans` | **1 plan** — `test`, `is_active=true` |
| `/planning` | HTTP 200 (post today's container rebuild) |
| `isDeleteDisabled` | **true** — expected for sole active plan per **DEC-0082** |

## Hypotheses (discovery)

| ID | Hypothesis | Next verify |
|----|------------|-------------|
| **H1** | Sole-plan UX (**BS**) — only one active plan; delete blocked by design but copy unclear | Confirm plan count on operator env; improve tooltip |
| **H2** | Undeployed frontend on omniflow — Q0031 fix not live | **FRONTEND_DEPLOY** + check bundle for `resolveDisplayedPlanId` |
| **H3** | Multi-plan BM regression — fix insufficient | Repro with 2+ plans, non-active selected |

## Acceptance

- **(BR)** Multi-plan delete enablement (extends **BUG-0022** BM)
- **(BS)** Sole-plan explained disable (new UX)

## Related

**BUG-0022** DONE / **Q0031**; **DEC-0082**; `planSelector.ts`

**Recommended next phase:** `/discovery` (BUG-0024)

---

