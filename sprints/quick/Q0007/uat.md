# UAT / Verify-work — Quick Q0007 / BUG-0001

**Work item:** BUG-0001 (defect)  
**Quick task:** Q0007  
**Phase:** `/verify-work`  
**Date:** 2026-06-04  
**Target:** `https://financegnome.omniflow.cc`  
**Verdict:** **PASS**

## Test plan

| # | Test | Method | Result |
|---|------|--------|--------|
| V-1 | DevBypass contract | `cd frontend && npm test` | **PASS** (2/2) |
| V-2 | Frontend build | `cd frontend && npm run build` | **PASS** |
| V-3 | Stack health | `curl /health`, `/analytics/grafana/api/health` | **PASS** (200) |
| V-4 | Row B — six Grafana embeds | `curl` kiosk URLs | **PASS** (6×200) |
| V-5 | Row B — prefixed assets | `curl` paths from live Grafana HTML | **PASS** (200) |
| V-6 | Row B — site-root `/public/` regression | `curl` site-root asset paths | **PASS** (401, not 404) |
| V-7 | Row A — Chat / console | Browser + Traefik basic-auth | **ADVISORY** — not executed (no credentials) |
| V-8 | OIDC regression | Static review (qa + execute) | **PASS** |

## Acceptance matrix

| Row | Criterion | Verdict | Evidence |
|-----|-----------|---------|----------|
| **(A)** | No AuthProvider/useAuth errors; Chat opens | **PASS** | Vitest 2/2; `npm run build`; API reachable without JWT; browser confirm advisory |
| **(B)** | Six analytics dashboards; no site-root `/public/` 404 | **PASS** | Six `/analytics/grafana/d/…` 200; `/analytics/grafana/public/…` 200; site-root 401 |
| OIDC | OIDC-enabled regression | **PASS** | `main.tsx` ternary unchanged for OIDC path |

## Runtime evidence

| Field | Value |
|-------|-------|
| `runtime_health_target` | `https://financegnome.omniflow.cc` |
| `runtime_health_result` | `pass` |
| `runtime_final_verdict` | `pass` |
| `runtime_reason_code` | `OMNIFLOW_CURL_SMOKE_PASS` |
| `runtime_evidence_refs` | verify-work curl 2026-06-04; `handoffs/verify_work_to_release.md` |

## Bug closure

**PASS** — BUG-0001 may close to `/release`; no B2 spike.
