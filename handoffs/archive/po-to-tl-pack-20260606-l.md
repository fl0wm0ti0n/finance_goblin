# PO to TL archive pack (2026-06-06)

- Rollover trigger: `PO_TO_TL_HOT_MAX_LINES=500, PO_TO_TL_HOT_MAX_SECTIONS=40`
- Source: `handoffs/po_to_tl.md`
- Archived units (oldest first, contiguous prefix): 1
- Retained units in hot file: 9
- First archived heading: `## research-20260608-us0014 — US-0014 planning UX technical research`
- Last archived heading: `## research-20260608-us0014 — US-0014 planning UX technical research`
- Verification tuple (mandatory):
  - archived_body_lines=53
  - retained_body_lines=490

---

## research-20260608-us0014 — US-0014 planning UX technical research

**From:** Tech Lead  
**To:** PO / Architecture  
**Date:** 2026-06-08  
**Story:** US-0014  
**Orchestrator run:** auto-20260608-us0014-001  
**Next phase:** `/architecture`

### Summary

Codebase + TanStack Query pattern research completed for US-0014 planning mutation feedback. Extended **[R-0072](docs/engineering/research.md#r-0072--us-0014-planning-ux-epic-gap-beyond-bug-0011)** §4 and appended **[R-0073](docs/engineering/research.md#r-0073--us-0014-planning-mutation-error-toast-patterns)** resolving all four discovery open questions: page-local feedback helper (not global MutationCache), immediate `plan-vs-actual` invalidation, scoped success confirmations, and user-guide section outline. Recommends **DEC-0077** error surface contract. No host `.env` or secrets read.

### Key findings by topic

| Topic | Research | Recommendation for architecture |
|-------|----------|--------------------------------|
| **Error UX** | [R-0073 §1](docs/engineering/research.md#r-0073--us-0014-planning-mutation-error-toast-patterns) | **Page-local `showPlanningFeedback`** — success green / error red card; single active slot; mandatory `onError` on 7 mutations |
| **Invalidation** | [R-0073 §2](docs/engineering/research.md#r-0073--us-0014-planning-mutation-error-toast-patterns) | Invalidate `plan-vs-actual` **immediately** on adjustment CRUD + activate + createPlan — do not wait for `plan_stale` badge |
| **Confirmation scope** | [R-0073 §3](docs/engineering/research.md#r-0073--us-0014-planning-mutation-error-toast-patterns) | Required toasts on createPlan, applyTemplate (all templates), addAdjustment, activate; optional on edit/delete |
| **User guide** | [R-0073 §5](docs/engineering/research.md#r-0073--us-0014-planning-mutation-error-toast-patterns) | Incremental over US-0004 — first-run, Set active + Dashboard 3, Compare footnote, troubleshooting; create at architecture |

### Discovery open questions — research resolution

| Question | Resolution (per research) |
|----------|---------------------------|
| Error UX — helper vs inline vs MutationCache? | **Page-local helper** — matches `PlanningPage.tsx` / `SubscriptionsPage.tsx` inline card; reject global MutationCache + toast library |
| Invalidation — PVA immediately or after recompute? | **Immediate** — badge advisory only; PVA tab may be open |
| Confirmation scope — every template vs primary CTA? | **Required** on create/add/activate/template apply; **optional** on PATCH/DELETE adjustment |
| User guide section scope? | **Delta doc** over US-0004 — Purpose, First visit, Set active, Compare semantics, Troubleshooting; file at architecture |

### Risks surfaced (carry to architecture)

1. **Toast flood** — single feedback slot; new message replaces prior on rapid retry
2. **Duplicate Q0019 work** — AC-1/AC-3/AC-4 verify-only; do not rewrite shipped paths
3. **DEC-0073/0074 frozen** — no Compare formula or PVA API changes
4. **AC-8 operator gate** — OIDC smoke requires **BACKEND_FRONTEND_DEPLOY** prerequisite

### Recommended DEC-0077 area decision

| ID | Scope | Proposed decision |
|----|-------|-------------------|
| **DEC-0077** | Planning mutation feedback | Page-local helper; mandatory onError on 7 mutations; success/error card variants; PVA invalidation matrix; set-active banner Dashboard 3 copy |
| *(sub-element)* | Backend | **No API changes** — frontend-only unless regression found |
| *(sub-element)* | User guide | `docs/user-guides/US-0014.md` at architecture; validate at execute S3 |

### Recommended next steps

1. `/architecture` — formalize **DEC-0077**; update `architecture.md` § US-0014; create spec-pack + user guide stub
2. `/sprint-plan` — materialize US-0014-S1..S3 with **S2 weighted** (AC-7 primary)

---

