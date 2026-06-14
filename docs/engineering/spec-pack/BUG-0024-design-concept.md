# Design Concept — BUG-0024

## Summary

BUG-0024 closes a sole-plan UX gap on `/planning`: after **Q0031** fixed the multi-plan delete selector, operators with **one** globally active plan still see a permanently disabled **Delete plan** button with tooltip-only copy that assumes another plan exists. Fix is frontend-only — show inline guidance describing the create→activate→delete workflow when delete is disabled for the sole active plan.

## Goals

- **BS:** Sole active plan — delete remains disabled per **DEC-0082** but UI shows **clear** create→activate→delete guidance — not silent gray button
- **BR:** Multi-plan non-active selection enables delete post-**FRONTEND_DEPLOY** — localhost already PASS; omniflow smoke deferred to V1
- Vitest coverage for `shouldShowSolePlanDeleteHint` predicate
- OIDC-enabled deploy regression pass

## Non-goals

- Backend `DELETE /api/v1/plans/:id` or **DEC-0082** 409 contract changes
- Selector priority changes (**BUG-0022** / Q0031 shipped)
- Create-plan CTA from disabled delete row (Option B — deferred)
- Delete active sole plan with auto-deactivate (Option C — rejected)
- New DEC record
- Playwright E2E (defer to verify-work / operator smoke)

## Key decisions

| Gate | Choice | Rationale |
|------|--------|-----------|
| GATE-COPY-1 | Inline hint below **Delete plan** row when sole active plan selected | Keyboard/screen-reader discoverable; satisfies **BS** |
| GATE-DEPLOY-1 | Operator **FRONTEND_DEPLOY** + omniflow 2-plan smoke | **BR** on omniflow pending deploy (**H2**) |
| GATE-SCOPE-1 | Frontend-only helper in `planSelector.ts` | Extends **DEC-0082** UX; no API change |
| GATE-TEST-1 | Vitest pure helper predicate | `planSelector.test.ts` precedent |
| GATE-DEC-1 | No new DEC | Presentation layer on existing guard |

**Spec-pack traceability:** `docs/engineering/spec-pack/BUG-0024-crs.md`, `docs/engineering/spec-pack/BUG-0024-technical-specification.md`
