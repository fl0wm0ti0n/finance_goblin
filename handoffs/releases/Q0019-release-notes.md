# Quick Release Notes — Q0019 / BUG-0011

**Quick task:** Q0019  
**Bug:** BUG-0011 — Planning mode broken (empty plan, compare sums, plan-vs-actual 404)  
**Date:** 2026-06-08  
**Backlog status:** DONE  
**Acceptance:** checked (`docs/product/acceptance.md` BUG-0011 rows AD/AE/AF)  
**Publish:** skipped (`RELEASE_PUBLISH_MODE=disabled`)

---

## Gate results

1. **Check-in test gate:** PASS — `cd backend && cargo test --lib` (160/160); `cargo test --test plans_integration` (5/5) @ 2026-06-08 release
2. **QA completion gate:** PASS — `sprints/quick/Q0019/qa-findings.md` (0 blockers)
3. **UAT / verify-work gate:** PASS — `sprints/quick/Q0019/uat.json`, `sprints/quick/Q0019/uat.md`, `sprints/quick/Q0019/qa-findings.md`; AD/AE/AF code PASS; runtime pass-with-prerequisites (BACKEND_FRONTEND_DEPLOY)
4. **Isolation compliance gate:** PASS — discovery through qa checkpoints in `docs/engineering/state.md`
5. **Strict runtime proof gate:** PASS — qa tuple `runtime-proof-qa-20260608-bug0011-q0019-001`; release tuple at finalization
6. **Release finalization gate:** PASS

---

## Summary

Planning mode defect fixes per **DEC-0073** (AE overlay-only compare delta) and **DEC-0074** (AF PVA 200 `no_active_plan`) plus **AD** first-run/add-line UX on US-0010 external profile:

| Scope | Fix |
|-------|-----|
| **AD** | First-run **Create empty plan**; inline add/edit adjustment form (POST/PATCH/DELETE); Custom Apply toast + query invalidation; Set active banner after create |
| **AE** | `monthly_overlay_delta_sum()` overlay-only compare metric; zero adjustments → **0.00**; compare footnote explains overlay-only delta vs projected balance |
| **AF** | `PlanVsActualApiResponse` tagged enum; route returns HTTP **200** `{ status: "no_active_plan" }`; PVA guided empty state (`retry: false`) with Set active CTA |

**Code proof:** `cargo test --lib` 160/160; `plans_integration` 5/5; overlay unit tests AE3 PASS; PVA API serialization AF1 PASS.

**Operator post-release:** Deploy backend + embedded frontend; run omniflow smoke AD-1–AD-3, AE-1–AE-2, AF-1–AF-2 per `sprints/quick/Q0019/uat.md`.

---

## Run

**Target service (external profile):** `flow-finance-ai` — rebuild/recreate for Q0019 backend + embedded frontend changes.

```bash
AUTHENTIK_SECRET_KEY=unused-external-profile docker compose \
  -f docker-compose.yml -f docker-compose.external.yml \
  --profile external up -d --build flow-finance-ai
```

**Operator gate — BACKEND_FRONTEND_DEPLOY (required before V1 smoke):**

```bash
docker compose -f docker-compose.yml -f docker-compose.external.yml \
  --profile external up -d --force-recreate flow-finance-ai
```

- `start_command`: docker compose commands above
- `runtime_mode`: remote (omniflow external US-0010); `AUTH_DEV_BYPASS=true` when OIDC unset
- `runtime_context_ref`: `docs/engineering/runbook.md` (§20 BUG-0011 hotfix)

**Profile rule:** **`external` only** — do not combine with `bundled-firefly`.

---

## Connect

- `service_url`: `https://financegnome.omniflow.cc`
- `service_port`: 443 (HTTPS via Traefik)
- `health_endpoint`: `GET /health`
- Planning UI: `/planning`

Required operator env (names only):

| Variable | Purpose |
|----------|---------|
| `DATABASE_HOST` | **`postgres`** on external profile (prerequisite §11) |
| `FIREFLY_BASE_URL` / `FIREFLY_PERSONAL_ACCESS_TOKEN` | Mirror data for forecast baseline |

---

## Verify

| Row | Check | Pass signal |
|-----|-------|-------------|
| **(AD)** | `/planning` Scenarios — Create empty plan | Editable plan with add form visible |
| **(AD)** | Add adjustment inline | POST creates row; PATCH edits row |
| **(AD)** | Custom template Apply | Toast "Custom plan ready — add lines below" |
| **(AE)** | Compare tab — zero-adjustment plan | `monthly_delta_sum` ≈ **0.00**; not full forecast net |
| **(AE)** | Compare tab — Leasing template | Overlay delta ~ leasing amount (~-300) |
| **(AF)** | `GET /api/v1/plans/active/plan-vs-actual` (no active) | HTTP **200** `{ status: "no_active_plan" }` |
| **(AF)** | Plan vs Actual tab (no active) | Guided card with Set active CTA — not blank/404 |
| Regression | OIDC `/planning` three tabs | Scenarios + Compare + Plan vs Actual load without console errors |

**Automated (release):**

```bash
cd backend && cargo test --lib
cd backend && cargo test --test plans_integration
```

**Live (operator post-deploy):** AD-1–AD-3, AE-1–AE-2, AF-1–AF-2, REG-1 per `sprints/quick/Q0019/uat.md`.

---

## Credentials

- Traefik basic auth — operator shell / password manager
- No inline secrets in release artifacts

---

## Changes

| Area | Summary |
|------|---------|
| `backend/src/plan/overlay.rs` | `monthly_overlay_delta_sum()` helper + unit tests |
| `backend/src/plan/repository.rs` | Compare `version_metrics` wired to overlay-only sum |
| `backend/src/plan/service.rs` | `project_adjustments_in_memory` overlay-only compare path |
| `backend/src/plan/types.rs` | `PlanVsActualApiResponse` tagged enum |
| `backend/src/api/plans.rs` | Route 200 `no_active_plan`; adjustment CRUD routes |
| `backend/tests/plans_integration.rs` | Compare + PVA integration tests (5 tests) |
| `frontend/src/pages/PlanningPage.tsx` | First-run create, inline add/edit, PVA guided state, footnote, banner |
| `frontend/src/lib/api.ts` | PVA + adjustment API types |
| Runbook | §20 BUG-0011 operator smoke |

**Linked decisions:** DEC-0073, DEC-0074  
**Research fulfilled:** R-0070, R-0015–R-0017, R-0020  
**Deferred:** Full planning UX epic **US-0014** if operator wants holistic redesign beyond defect fixes

---

## Known Issues

- V1 omniflow runtime probes pass-with-prerequisites until **BACKEND_FRONTEND_DEPLOY**
- OIDC browser regression smoke deferred (API/code PASS per prior bug releases)
- Grafana Dashboard 3 unchanged — compare fix is React Compare tab + `/compare` API only

---

## Regression scope

- US-0004 plan engine baseline + Set active semantics preserved (no auto-activate on create)
- BUG-0008 subscription alerts unchanged (coordinate-only)
- Grafana Dashboard 3 provisioning JSON unchanged (R-0020)
