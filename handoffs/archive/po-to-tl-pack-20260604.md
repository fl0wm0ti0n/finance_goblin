# PO to TL archive pack (2026-06-04)

- Rollover trigger: `PO_TO_TL_HOT_MAX_LINES=500, PO_TO_TL_HOT_MAX_SECTIONS=40`
- Source: `handoffs/po_to_tl.md`
- Archived units (oldest first, contiguous prefix): 5
- Retained units in hot file: 7
- First archived heading: `## intake-20260605-bug0003 — BUG-0003 omniflow production API 500 / Bitunix / Grafana SQL`
- Last archived heading: `## intake-20260604-bug0001 — BUG-0001 omniflow production regressions (auth + Grafana)`
- Verification tuple (mandatory):
  - archived_body_lines=357
  - retained_body_lines=485

---

## intake-20260605-bug0003 — BUG-0003 omniflow production API 500 / Bitunix / Grafana SQL

**From:** PO  
**To:** Tech Lead  
**Date:** 2026-06-05  
**Work item:** BUG-0003 (defect)  
**Next phase:** `/discovery`

### Summary

Operator investigation on `financegnome.omniflow.cc` (US-0010 external, post-Q0007). Single bug **BUG-0003** with three independently testable sub-defects. **Do not merge with BUG-0002** unless PO explicitly extends (BUG-0002 = Firefly PAT / risk-score / settings display).

| Sub | Priority | Symptom | Intake hypothesis |
|-----|----------|---------|-------------------|
| **F** | P0 | Many `GET /api/v1/*` → **500** (~30s); logs `response failed` + DB timeout pattern | `DATABASE_HOST=host.docker.internal` in container overrides overlay `${DATABASE_HOST:-postgres}` — pool cannot reach DB on traefik network |
| **G** | P0 | `POST /api/v1/exchanges/bitunix/test` → **400** empty body | `ExchangeService::new` uses TOML `bitunix.enabled` not `effective_enabled()` → unknown exchange; plus spot `openapi.bitunix.com` query-sign vs futures `fapi.bitunix.com` header auth ([R-0058](docs/engineering/research.md#r-0058--bitunix-futures-api-auth-vs-connector-implementation)) |
| **H** | P1 | `POST .../analytics/grafana/api/ds/query` → **400** SQL error | Grafana datasource `${DATABASE_HOST}` same wrong host; duplicate dashboard UID warnings secondary |

**Out of scope:** wallet extension `contentscript.js` / ObjectMultiplex.

### Split decision

- **Chosen:** single **BUG-0003** (F+G+H)
- **Alternatives rejected:** three bugs (duplicate env); merge into BUG-0002 (different symptom cluster)
- **Discovery may decompose:** F1 env/compose/docs, G1 connector registry, G2 Bitunix auth/URL, H1 datasource host (may be F-only fix)

### Intake evidence (validated)

- `intake_run_id`: `intake-20260605-omniflow-prod-api-500`
- `selected_pack`: `small-intake-pack`
- `asked_topics`: outcome_success_criteria, impacted_components, constraints_compatibility_risks, required_tests_acceptance_checks, done_definition
- `missing_topics`: _(none)_
- `assumptions_confirmed`: `(none)`
- `topic_coverage`: see bundle (distinct `ie:` refs per topic)
- Evidence bundle: `handoffs/intake_evidence/intake-20260605-omniflow-prod-api-500.json` — `[INTAKE_EVIDENCE_VALIDATION_OK]`

### Acceptance rows (F / G / H)

| Sub | Pass criteria |
|-----|---------------|
| **F** | Representative `/api/v1/*` return **200** &lt;5s; settings `database_host: postgres`, `database_mode: external` |
| **G** | Bitunix test **200** or documented auth error — not **400** unknown exchange |
| **H** | Grafana ds/query **200** for provisioned SQL panels |

### Research refs

- [R-0052](docs/engineering/research.md#r-0052--external-compose-integration-on-omniflow-traefik-host) — `DATABASE_HOST=postgres` on traefik
- [R-0058](docs/engineering/research.md#r-0058--bitunix-futures-api-auth-vs-connector-implementation) — Bitunix futures vs spot connector (intake-time)
- [R-0032](docs/engineering/research.md#r-0032--exchange-connector-rest-patterns-binance-bybit-bitunix) — baseline spot signing

### Discovery guidance

1. Confirm F on omniflow: container env `DATABASE_HOST`, settings API, sample GET latency — no `.env` secret reads.
2. After F fix, re-test H (may be sufficient without Grafana code change).
3. G: verify connector list includes bitunix when credentials set; capture test HTTP body; compare futures header auth if spot test fails.
4. Map tasks to acceptance F/G/H; recommend `/quick` if ≤3 vertical slices.

### Related

- **BUG-0002** OPEN — separate track (Q0008 PAT verify blocked)

---

## discovery-20260604-bug0002 — BUG-0002 omniflow production integration defects discovery

**From:** PO  
**To:** Tech Lead  
**Date:** 2026-06-04  
**Work item:** BUG-0002 (defect)  
**Next phase:** `/architecture` (skip standalone `/research` — R-0057 sufficient for C)

### Summary

Discovery confirms **three independent fix slices** under one bug. Production curl evidence (no host secrets read) refutes sync API misroute and Traefik-only theories for D.

| Sub | Confirmed root cause | PO-recommended fix |
|-----|---------------------|-------------------|
| **C** | Empty/invalid `FIREFLY_PERSONAL_ACCESS_TOKEN` in container → Firefly **401** → sync `failed` (`unexpected status 401 Unauthorized`). Sync APIs routable (**200** on `/api/v1/sync/status`). | **C1** operator PAT + compose env load; **C2** treat empty PAT as unset + fail-fast/health message |
| **D** | Handler returns **404** when no `plan_risk_scores` row for active plan (`latest_for_active_plan()` → `None`). Route registered; not stale deploy routing. | **D1** return **200** documented empty-state JSON (acceptance-aligned) |
| **E** | TOML `enabled` decoupled from env `configured()`; prod: Bitunix configured=true, enabled=false | **E1** effective `enabled = configured() \|\| toml.enabled` in settings + startup mirror |

### Runtime proof (2026-06-04)

| Endpoint | HTTP | Evidence |
|----------|------|----------|
| `/api/v1/sync/status` | 200 | `state: failed`, `error_message: unexpected status 401 Unauthorized` |
| `/api/v1/plans/risk-score` | 404 | Empty body — application not-found, not missing router |
| `/api/v1/settings` | 200 | `binance.enabled=true, configured=false`; `bitunix.enabled=false, configured=true` |

`isolation_scope`: code inspection + public HTTPS curl; no operator `.env` / PAT values read.

### Sub-defect C — Firefly sync

- **Files:** `backend/src/config/mod.rs` (PAT overlay), `backend/src/firefly/mod.rs`, `docker-compose.yml` env passthrough, runbook PAT smoke.
- **Ruled out:** `/api/v1/sync/*` Traefik 404 (status **200**).
- **Operator action:** Set non-empty PAT; verify after container recreate (`printenv` names-only check per R-0057 / runbook).

### Sub-defect D — risk-score 404

- **Files:** `backend/src/api/plans.rs` `risk_score`, `backend/src/plan/risk.rs`, `frontend/src/pages/PlanningPage.tsx`.
- **Contributing:** C blocks successful sync → plan risk refresh may not run.
- **Contract:** Prefer `200` + `{ status: "no_score", reason: ... }` over bare 404.

### Sub-defect E — exchange settings

- **Files:** `backend/src/config/mod.rs` `settings_view`, `backend/src/exchanges/service.rs` `mirror_enabled_at_startup`, `backend/config/default.toml`, `frontend/src/pages/SettingsPage.tsx`.
- **Optional:** `binance.enabled=false` default (E2).

### Fix-task decomposition

1. **C1** — PAT operator + compose verification (ops/docs)
2. **C2** — Empty PAT guard + explicit health/sync error (backend)
3. **D1** — Risk-score empty-state 200 (backend + Planning UI)
4. **E1** — Effective enabled from credentials (backend)
5. **E2** — default.toml binance.enabled=false (optional)

C2, D1, E1 independently deployable; C1 gates omniflow acceptance for row C.

### Test plan (discovery)

| Check | Type | Pass criteria |
|-------|------|---------------|
| C — PAT loaded | Operator | `printenv FIREFLY_PERSONAL_ACCESS_TOKEN` non-empty (no value logged) |
| C — sync success | Operator smoke | Manual sync success; entity counts > 0; no 401 in `last_run.error_message` |
| C — sync API | curl | `/api/v1/sync/status` 200; `/api/v1/sync/trigger` accepted |
| D — risk empty | curl | `/api/v1/plans/risk-score` **200** empty-state OR **200** score when plan active |
| D — Planning UI | Operator | Planning page loads without treating 404 as hard error |
| E — settings | curl/UI | Bitunix-only env → Bitunix enabled=yes + configured=yes |
| Regression | Operator | OIDC-enabled + bundled-firefly profiles per acceptance |

### Risks

| Risk | Mitigation |
|------|------------|
| PAT in `.env` not in container | C1 runbook + compose cwd |
| Empty-state contract drift | Architecture documents JSON shape |
| False auto-enable | E1 mirrors credentials only; sync still validates keys |

### Research refs

- [R-0057](docs/engineering/research.md#r-0057--firefly-iii-api-docs-discovery-post-bug-0001) — PAT Bearer; no new R-xxxx
- [R-0001](docs/engineering/research.md#r-0001--firefly-iii-rest-api-integration-baseline), [R-0032](docs/engineering/research.md#r-0032--exchange-connector-rest-patterns-binance-bybit-bitunix)

### Architecture guidance

1. Accept **C2** empty-PAT guard and operator-facing error contract.
2. Accept **D1** 200 empty-state (do not rely on 404 for “no plan”).
3. Accept **E1** effective enabled semantics; decide E2 default.toml change.
4. Recommend **`/quick`** if ≤4 tasks and no new DEC — else minimal bug-fix sprint.
5. Map acceptance rows C/D/E to tasks; no acceptance rewrite unless architecture finds gap.

### Intake evidence (unchanged)

- `intake_run_id`: `intake-20260604-omniflow-prod-integration`
- Evidence bundle: `handoffs/intake_evidence/intake-20260604-omniflow-prod-integration.json`

---

## intake-20260604-bug0002 — BUG-0002 omniflow production integration defects

**From:** PO  
**To:** Tech Lead  
**Date:** 2026-06-04  
**Work item:** BUG-0002 (defect)  
**Next phase:** `/architecture` (discovery complete — see `discovery-20260604-bug0002` above)

### Summary

Operator report on `financegnome.omniflow.cc` (US-0010 external) **after BUG-0001 / Q0007**. Single bug **BUG-0002** with three independently testable sub-defects:

| Sub | Priority | Symptom | Intake hypothesis |
|-----|----------|---------|-------------------|
| **C** | P0 | Firefly sync / Sync Status not pulling data; console `sync:1` 404; prior sync `failed` | PAT/base URL connectivity, `/api/v1/sync/*` misroute, or connector error masked as UI 404 |
| **D** | — | `GET /api/v1/plans/risk-score` → **404** on public host | Route exists in `plans.rs` — suspect stale omniflow image, Traefik `financegnome-api` gap, or SPA fallback swallowing `/api` |
| **E** | — | Bitunix env set; Settings shows Binance enabled+configured, Bitunix no+configured | `default.toml` `binance.enabled=true` / `bitunix.enabled=false` decoupled from `configured()` env detection |

**Out of scope:** `contentscript.js` / ObjectMultiplex wallet extension noise.

### Split decision

- **Chosen:** single **BUG-0002** (sub-defects C+D+E)
- **Alternative rejected:** three bugs — duplicate environment/evidence overhead
- **Discovery may decompose:** C1 Firefly connector, D1 API routing/deploy, E1 settings enabled semantics

### Intake pack

- `selected_pack`: `small-intake-pack`
- `intake_run_id`: `intake-20260604-omniflow-prod-integration`
- Evidence: `handoffs/intake_evidence/intake-20260604-omniflow-prod-integration.json` — validation **OK**
- `asked_topics`: outcome_success_criteria, impacted_components, constraints_compatibility_risks, required_tests_acceptance_checks, done_definition
- `missing_topics`: _(none)_
- `assumptions_confirmed`: `(none)`

### Constraints

- Traefik split routers (API without basic-auth on `financegnome-api`)
- `AUTH_DEV_BYPASS=true`, OIDC unset
- Do **not** read operator `.env` / secrets
- Regression: OIDC-enabled and bundled-firefly profiles

### Research refs

- [R-0057](docs/engineering/research.md#r-0057--firefly-iii-api-docs-discovery-post-bug-0001) — Firefly OpenAPI discovery for sub-defect C
- [R-0001](docs/engineering/research.md#r-0001--firefly-iii-rest-api-integration-baseline), [R-0002](docs/engineering/research.md#r-0002--firefly-iii-pagination--incremental-sync-strategy)
- [R-0032](docs/engineering/research.md#r-0032--exchange-connector-rest-patterns-binance-bybit-bitunix) — exchange settings baseline

### Acceptance mapping (unchanged until discovery)

- **C:** Firefly sync success + entity counts; no blocking sync API 404
- **D:** `GET /api/v1/plans/risk-score` → 200
- **E:** Bitunix-only env → Bitunix enabled+configured; no false Binance enabled

### Recommended discovery focus

1. Reproduce C with backend sync logs + `curl` to `/api/v1/sync/status` and `/api/v1/sync/trigger` via public host (names-only env).
2. Reproduce D — compare in-container route list vs Traefik router; verify deploy tag post-Q0007.
3. Reproduce E — `GET /api/v1/settings` JSON vs operator env variable **names** only.
4. Propose fix slices; `/quick` if ≤3 vertical tasks.

---

## discovery-20260604-bug0001 — BUG-0001 omniflow production regressions discovery

**From:** PO  
**To:** Tech Lead  
**Date:** 2026-06-04  
**Work item:** BUG-0001 (defect)  
**Next phase:** `/architecture` (skip standalone `/research` — R-0056 §3 sufficient; bounded B2 spike only if env fix fails)

### Summary

Discovery confirms **two independent fix slices** under one bug, both regressions from incomplete Q0005 (auth) and missing Grafana `root_url` wiring (analytics) on the US-0010 external profile.

| Sub | Confirmed root cause | PO-recommended fix |
|-----|---------------------|-------------------|
| **A** | Conditional `AuthProvider` in `main.tsx`; `AppLayout` + `ChatPanel` call `useAuth()` unconditionally | **`DevBypassAuthProvider` stub** when `!isOidcConfigured` — single mount in `main.tsx`; audit all `useAuth()` consumers |
| **B** | Prefix-strip proxy passes Grafana HTML with root-absolute `/public/…` asset URLs | **`GF_SERVER_ROOT_URL=https://financegnome.omniflow.cc/analytics/grafana/`** in compose (operator override); **`serve_from_sub_path=false`** unchanged. Fallback: proxy `text/html` rewrite for `/public/` paths |

**Out of scope unchanged:** wallet extension console noise.

### Sub-defect A — auth scope

- **Files:** `frontend/src/main.tsx`, `frontend/src/components/AppLayout.tsx`, `frontend/src/components/chat/ChatPanel.tsx` (via `AiSheet.tsx`).
- **Already correct:** `App.tsx` `ProtectedRoute` / `OidcProtectedRoute` split; sidebar logout/name gated on `isOidcConfigured`.
- **Gap:** React hooks rules prevent conditional `useAuth()` — stub provider is lowest-churn fix vs component splits.
- **OIDC regression:** stub mounts only when authority unset; token provider + redirect flow untouched when OIDC configured.
- **Q0005 context:** Traefik auth loop fixes (`credentials: include`, router split Q0006) remain valid — this is a separate incomplete AuthProvider guard.

### Sub-defect B — Grafana scope

- **Files:** `docker-compose.yml` (Grafana env block), `.env.example`, optionally `backend/src/analytics/proxy.rs`.
- **Already correct:** iframe `src` in `AnalyticsEmbedPage.tsx`; prefix strip + WS + header filter in `proxy.rs`; Traefik `/analytics` router without auth (Q0006).
- **Gap:** No `GF_SERVER_ROOT_URL` in compose today; DEC-0057 risks table flagged this; omniflow smoke was deferred (`OMNIFLOW_HOST_UNAVAILABLE`).
- **Constraint:** Do **not** enable `GF_SERVER_SERVE_FROM_SUB_PATH` without new DEC (DEC-0057 Option 3 rejected).

### Fix-task decomposition

1. **A1** — Auth stub provider + consumer audit (frontend)
2. **B1** — `GF_SERVER_ROOT_URL` compose + `.env.example` (compose/docs)
3. **B2** — Proxy HTML rewrite (backend, **conditional** on B1 omniflow smoke fail)

Tasks A1 and B1 are independently deployable/testable; acceptance rows A and B map 1:1.

### Test plan (discovery)

| Check | Type | Pass criteria |
|-------|------|---------------|
| A — console clean | Operator smoke | OIDC-unset + `AUTH_DEV_BYPASS=true`: no AuthProvider/useAuth warn; no `user` TypeError |
| A — Chat usable | Operator smoke | Header AI button opens `ChatPanel`; `/chat` route works |
| A — OIDC regression | Build smoke | OIDC-configured build: redirect to IdP; token on API calls |
| A — unit | Vitest | Stub provider renders `ChatPanel` without throwing |
| B — assets prefixed | Operator smoke | Six `/analytics/{slug}` routes; no site-root `/public/` 404 |
| B — asset URLs | Network tab | `/analytics/grafana/public/build/…` and `/public/img/…` return 200 |
| B — integration | Rust test | Mock HTML asset path rewrite OR env contract documented |
| B — WS live | Operator smoke | Grafana Live refresh on one dashboard (US-0011 checklist) |
| Done | Operator | BUG-0001 acceptance rows A+B checked on `financegnome.omniflow.cc` |

### Risks

| Risk | Mitigation |
|------|------------|
| Stub masks prod OIDC misconfig | Stub gated on `!isOidcConfigured`; docs |
| Wrong `root_url` on non-omniflow hosts | `.env` override; default = US-0010 canonical |
| HTML rewrite fragility | Env-first; B2 only if B1 fails |
| OIDC regression | Explicit acceptance row |
| WS breakage | Existing proxy; US-0011 smoke |

### Research refs

- [R-0056 §3](docs/engineering/research.md#r-0056--us-0011-grafana-embed-proxy-auth-csp-subpath-websocket-traefik) — `GF_SERVER_ROOT_URL` + prefix strip (no new R-xxxx required)
- [DEC-0057](decisions/DEC-0057.md) — proxy contract; Panel links risk row
- `sprints/quick/Q0005/summary.md`, `sprints/quick/Q0006/summary.md` — auth context

### Architecture guidance

1. Accept **DevBypassAuthProvider** contract — document stub shape vs real `AuthContextProps` subset used by app.
2. Accept **B1-first Grafana fix** — compose env + operator override; gate B2 on smoke.
3. Recommend **quick sprint** (`/quick`) if task count ≤3 and no new DEC — else minimal bug-fix sprint.
4. Map acceptance rows A+B to tasks; no acceptance change unless architecture finds gap.

### Intake evidence (unchanged)

- `intake_run_id`: `intake-20260604-omniflow-prod-regressions`
- Evidence bundle: `handoffs/intake_evidence/intake-20260604-omniflow-prod-regressions.json`

---

## intake-20260604-bug0001 — BUG-0001 omniflow production regressions (auth + Grafana)

**From:** PO  
**To:** Tech Lead  
**Date:** 2026-06-04  
**Work item:** BUG-0001 (defect)  
**Next phase:** `/discovery`

### Summary

Operator smoke on **`financegnome.omniflow.cc`** (US-0010 external profile, Traefik `auth`, `AUTH_DEV_BYPASS=true`, OIDC unset) reports **two production regressions** filed as sub-defects under a **single bug**:

| Sub | Symptom | Likely cause |
|-----|---------|--------------|
| **A** | `AuthProvider context is undefined` / `useAuth()` `user` TypeError on AI/Chat | Q0005 conditional `AuthProvider` in `main.tsx`; `AppLayout.tsx`, `ChatPanel.tsx` still call `useAuth()` unconditionally |
| **B** | All six `/analytics/{slug}` Grafana iframes fail; `/public/build/` → 404 at site root | Grafana HTML emits root-absolute asset URLs; DEC-0057 prefix-strip proxy without `GF_SERVER_ROOT_URL` / rewrite |

**PO split decision:** **one BUG-0001** (same operator session/environment). Alternative two-bug split rejected to avoid duplicate evidence blocks; acceptance rows **A+B** remain independently testable within one checkbox.

**Out of scope:** wallet extension `contentscript.js` / ObjectMultiplex console noise.

### Intake evidence

- `intake_run_id`: `intake-20260604-omniflow-prod-regressions`
- `selected_pack`: `small-intake-pack`
- `intake_work_item_kind`: `bug`
- `asked_topics`: outcome_success_criteria, impacted_components, constraints_compatibility_risks, required_tests_acceptance_checks, done_definition
- `missing_topics`: _(none)_
- `assumptions_confirmed`: `(none)`
- Evidence bundle: `handoffs/intake_evidence/intake-20260604-omniflow-prod-regressions.json`
- Validator: `[INTAKE_EVIDENCE_VALIDATION_OK]`, `[BUG_VALIDATION_OK]`

### Acceptance (canonical)

- [ ] **BUG-0001** — **(A)** OIDC-unset + dev bypass: no AuthProvider/useAuth errors; Chat opens. **(B)** Six analytics routes load Grafana; no site-root `/public/` 404s. OIDC-enabled regression check.

### Discovery guidance

1. **A — Auth guard pattern:** Extend Q0005 split — either wrap shell/chat in no-op auth context when OIDC unset, or guard all `useAuth()` consumers (`AppLayout`, `ChatPanel`, `AiSheet`, etc.); preserve OIDC path.
2. **B — Grafana assets:** Revisit R-0056 §3 / DEC-0057 — evaluate `GF_SERVER_ROOT_URL=https://financegnome.omniflow.cc/analytics/grafana/` vs proxy HTML rewrite for `/public/` hrefs; **do not** assume `GF_SERVER_SERVE_FROM_SUB_PATH` without new DEC.
3. **Smoke:** US-0010 external profile + US-0011 analytics checklist on omniflow host.

### Research refs

- [R-0056](docs/engineering/research.md#r-0056--us-0011-grafana-embed-proxy-auth-csp-subpath-websocket-traefik) — Grafana subpath / root_url
- [DEC-0057](decisions/DEC-0057.md) — analytics proxy contract
- `sprints/quick/Q0005/summary.md` — partial AuthProvider fix context

---

