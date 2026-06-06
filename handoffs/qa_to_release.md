# QA → Release / Verify-work Handoff

**From:** QA (`/qa`)  
**To:** Verify-work (`/verify-work`) then Release (`/release`)  
**Date:** 2026-06-04  
**Work item:** BUG-0001  
**Quick task:** Q0007  

## Verdict

**PASS** — ready for `/verify-work`. No `handoffs/qa_to_dev.md` fix list.

## Summary

Q0007 deliverables verified against `architecture-20260604-bug0001`: **A1** `DevBypassAuthProvider` completes Q0005 dev-bypass auth path (stub on `AuthContext` when `!isOidcConfigured`); **B1** `GF_SERVER_ROOT_URL` implements DEC-0057 / R-0056 env-first mitigation; **B2** skipped per execute gate. QA re-ran `npm test` (2/2) and `npm run build` — **PASS**. Omniflow live rows A+B **deferred** until operator redeploys `flow-finance-ai` + `grafana` with new images/env.

## Evidence

| Artifact | Path |
|----------|------|
| QA findings | `sprints/quick/Q0007/qa-findings.md` |
| Dev handoff | `handoffs/dev_to_qa.md` |
| Architecture | `handoffs/tl_to_dev.md` (`architecture-20260604-bug0001`) |
| Acceptance | `docs/product/acceptance.md` (BUG-0001) |
| Execute summary | `sprints/quick/Q0007/summary.md` |

## Acceptance snapshot

| Row | QA verdict | Runtime / operator |
|-----|------------|-------------------|
| **(A)** OIDC-unset + dev bypass: no useAuth errors; Chat opens | **PASS** (static + Vitest) | Console + ChatPanel smoke **PENDING** — frontend redeploy |
| **(B)** Six analytics routes; no site-root `/public/` 404 | **PASS** (static compose) | Grafana asset Network tab **PENDING** — Grafana recreate with `GF_SERVER_ROOT_URL` |
| OIDC regression | **PASS** (static guard) | IdP redirect smoke optional in verify-work |

**Deferred items:**

- **Omniflow runtime:** Post-redeploy witness for BUG-0001 rows A+B — `OPERATOR_REDEPLOY_PENDING` (host `/health` reachable; B1 not yet verified on running Grafana).
- **B2 gate:** Implement proxy HTML rewrite only if post-redeploy B smoke still fails.

## Advisories (non-blocking)

1. Operator redeploy per `handoffs/dev_to_qa.md` (external compose profile).
2. Hard refresh; verify `/analytics/grafana/public/…` returns 200 in Network tab.
3. Optional `.env` override for non-omniflow: `GF_SERVER_ROOT_URL=http://localhost:8080/analytics/grafana/`

## Verify-work focus

1. Witness or execute omniflow smoke after operator redeploy; record timestamps.
2. If row B still fails → escalate B2 spike; else close BUG-0001 acceptance checkboxes.
3. Release gate: QA PASS + verify-work PASS + isolation checkpoints in `state.md`.

## Boundaries preserved

- No `GF_SERVER_SERVE_FROM_SUB_PATH`
- No proxy prefix / JWT stack changes
- No Q0005/Q0006 Traefik router revert
- B2 not implemented unless B1 smoke fails

---
**Next phase:** `/verify-work` in new subagent/chat
