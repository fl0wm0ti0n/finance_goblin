# Architecture archive pack (2026-06-13)

- Rollover trigger: `ARCH_HOT_MAX_LINES=3000, ARCH_HOT_MAX_STORY_SECTIONS=120`
- Source: `docs/engineering/architecture.md`
- Archived units (oldest first, contiguous prefix): 1
- Retained units in hot file: 12
- First archived heading: `## US-0013 — Production ML forecast & wealth analytics hardening`
- Last archived heading: `## US-0013 — Production ML forecast & wealth analytics hardening`
- Verification tuple (mandatory):
  - archived_body_lines=258
  - preamble_lines=10
  - retained_body_lines=2812

---

## US-0013 — Production ML forecast & wealth analytics hardening

**Status:** Architecture complete (2026-06-08)  
**Discovery:** `discovery-20260608-us0013` in `handoffs/po_to_tl.md`  
**Research:** [R-0071](research.md#r-0071--us-0013-production-ml-enablement-on-omniflow-external-profile); addenda R-0043, R-0044, R-0045, R-0053, R-0062  
**Decisions:** **DEC-0076** (external ML compose contract); extends DEC-0049, DEC-0052, DEC-0056, DEC-0066  
**Depends on:** US-0009 (ML feature stack), US-0010 (external profile), BUG-0010 DONE (baseline prerequisite)  
**Sprint:** **S0014** recommended — slices US-0013-S1..S4  
**Acceptance:** `docs/product/acceptance.md` § US-0013 (10 rows)  
**Spec-pack:** `docs/engineering/spec-pack/US-0013-{design-concept,crs,technical-specification}.md` (`SPEC_PACK_MODE=1`)  
**User guide:** `docs/user-guides/US-0013.md` (`USER_GUIDE_MODE=1`)

### Problem

US-0009 delivered a feature-complete ML stack, but omniflow (`--profile external`) is **baseline-only by design**: `stats-forecast` starts only on Compose profile `[full]`; `docker-compose.external.yml` has no sidecar; `[forecast_ml] enabled=false` default (DEC-0049). Result: zero `ml_enhanced` computations, empty Grafana ML panels, Compare disabled with `sidecar_disabled` (DEC-0066).

BUG-0010 closed baseline numbers (AA/AB/AC); AC3 production ML path was explicitly deferred to **US-0013**. Gap is **infra wiring + operator opt-in + verification** — not new ML research or UI greenfield.

`isolation_scope`: artifact + repo source reads; no host `.env` / secrets read.

### System context (external profile — target state)

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│  Traefik (external network) — financegnome.omniflow.cc                       │
└───────┬──────────────────────┬──────────────────────┬────────────────────────┘
        │                      │                      │
        ▼                      ▼                      ▼
┌───────────────┐    ┌─────────────────┐    ┌─────────────────────────┐
│ flow-finance-ai│    │ grafana         │    │ stats-forecast (NEW)    │
│ traefik only   │    │ traefik only    │    │ profiles: [full,external]│
│ FORECAST_ML_   │    │ internal embed  │    │ traefik network         │
│ ENABLED=true   │    │ via DEC-0057    │    │ host :8091 → :8090      │
└───────┬───────┘    └────────┬────────┘    └───────────┬─────────────┘
        │                     │                         │
        │ POST /v1/forecast   │ SQL                     │ GET /health
        └─────────────────────┴─────────────────────────┘
                                │
                                ▼
                    ┌───────────────────────┐
                    │ Host postgres (traefik)│
                    │ ml_enhanced rows       │
                    └───────────────────────┘
```

**Baseline authority unchanged (DEC-0050):** Alerts, plan hook, AI default, Grafana default variant remain `model_kind=baseline`.

### Architecture contract (DEC-0076)

```text
US-0013
├── S1 — External compose + ML config enablement (P0)
│   ├── docker-compose.external.yml: stats-forecast overlay (profiles [external], traefik network)
│   ├── flow-finance-ai env: FORECAST_ML_ENABLED, STATS_FORECAST_URL
│   ├── .env.example omniflow ML block
│   └── compose-config-check.sh: 3-service external set + stats-forecast traefik assert
├── S2 — Sync ML pipeline + API persistence (P0)
│   ├── Verify forecast_ml phase + health_ok() gate (existing code)
│   └── Verify GET /forecast variant=ml_enhanced after Full sync
├── S3 — UI + Grafana ML parity (P0)
│   ├── Verify ForecastPage Compare + sidecar_disabled copy (DEC-0066)
│   ├── Verify WealthPage portfolio-forecast + FX banner (DEC-0065, R-0034)
│   └── Verify Grafana forecast-horizons $forecast_variant=ml_enhanced
└── S4 — Runbook + CI sidecar fixture (P0)
    ├── runbook § Omniflow ML enablement
    └── retain forecast_ml_integration + compose assert (dual CI guard)
```

**Out of scope:** New ML models; monthly bucket attribution (US-0015); Grafana empty-state-only (BUG-0009 DONE).

### S1 — Compose overlay (frozen)

#### Profile union pattern

| File | `stats-forecast` contract |
|------|---------------------------|
| `docker-compose.yml` (base) | `profiles: [full]`; port `${STATS_FORECAST_PORT:-8090}:8090`; healthcheck unchanged |
| `docker-compose.external.yml` (overlay) | **Additive** `profiles: [external]` on same service key; `networks: [traefik]`; port `${STATS_FORECAST_PORT:-8091}:8090` |

Compose merges profile arrays → `[full, external]` on one service definition → **one container** when either profile active.

**Rejected:** duplicate `stats-forecast:` block in overlay; `profiles: [full, external]` on base only (starts sidecar on minimal unintentionally).

#### External overlay execute sketch

```yaml
# docker-compose.external.yml (append to existing services)
  stats-forecast:
    profiles: [external]
    networks:
      traefik:
    ports:
      - "${STATS_FORECAST_PORT:-8091}:8090"

  flow-finance-ai:
    environment:
      FORECAST_ML_ENABLED: ${FORECAST_ML_ENABLED:-false}
      STATS_FORECAST_URL: ${STATS_FORECAST_URL:-http://stats-forecast:8090}
```

Operator enables ML by setting `FORECAST_ML_ENABLED=true` in `.env` — preserves DEC-0049 default-off.

#### Network contract

| Element | Value |
|---------|-------|
| Sidecar attachment | **Traefik-only** — matches `flow-finance-ai` external merge (no default network) |
| Internal URL | `http://stats-forecast:8090` (container port, not host remap) |
| Host debug | `curl localhost:${STATS_FORECAST_PORT:-8091}/health` |

**Rejected:** dual-network sidecar — unnecessary when backend is traefik-only (R-0071).

#### CI compose assert (frozen)

Update `scripts/compose-config-check.sh`:

| Check | Before | After |
|-------|--------|-------|
| External `config --services` | `flow-finance-ai grafana` | `flow-finance-ai grafana stats-forecast` |
| Traefik network | flow-finance-ai, grafana | + stats-forecast |

DEC-0056 anti-combination guards (`minimal+external` no firefly-iii) **unchanged**.

### S2 — Sync + API (verify-first)

Existing implementation — **no algorithm changes** expected:

| Component | Path | Contract |
|-----------|------|----------|
| Sync phase | `backend/src/sync/mod.rs` | `forecast_ml` after baseline; phase label "ML forecast…" |
| Health gate | `backend/src/forecast_ml/sidecar.rs` | `health_ok()` GET `/health` before ML pass |
| Skip metadata | `backend/src/forecast_ml/service.rs` | `record_skip_on_baseline` on failure (DEC-0052) |
| Min history | `default.toml` `[forecast_ml]` | `min_monthly_points = 12` unchanged |
| API | `backend/src/api/forecast.rs` | `variant=ml_enhanced` returns bands + series |

**Sidecar SLO (frozen):**

| Layer | Contract |
|-------|----------|
| Runtime | `health_ok()` before ML phase; 60s HTTP timeout |
| Compose healthcheck | `start_period: 30s` — advisory only |
| Cold start | First sync may `sidecar_unavailable` — manual re-sync acceptable (DEC-0052) |

### S3 — UI + Grafana (verify-first)

| Surface | Path | Verify |
|---------|------|--------|
| Forecast Compare | `frontend/src/pages/ForecastPage.tsx` | Baseline + ML overlay when data exists |
| Degraded copy | ForecastPage + meta API | `sidecar_disabled` per DEC-0066 |
| Wealth outlook | `frontend/src/pages/WealthPage.tsx` | Portfolio horizons; signed totals (DEC-0065) |
| Grafana ML | `grafana/.../forecast-horizons.json` | `$forecast_variant=ml_enhanced` panels with data |
| ML off banner | Grafana provisioning | BUG-0009 banner remains when ML disabled |

**No new React/Grafana features** — verification on external profile post-enablement.

### S4 — Runbook + CI (execute)

| Deliverable | Content |
|-------------|---------|
| Runbook § Omniflow ML enablement | Compose union, env vars, health probe, Full sync prerequisite, min history, degraded troubleshooting |
| CI dual guard | `compose-config-check.sh` update + `forecast_ml_integration` retained in `tests/run-tests.sh` |
| User guide | `docs/user-guides/US-0013.md` (operator path) |

### Codebase map (ML enablement slice)

| Path | Role | US-0013 touch |
|------|------|---------------|
| `docker-compose.yml` | Base `stats-forecast` `[full]` | Reference only — no base profile change |
| `docker-compose.external.yml` | Omniflow overlay | **S1** — sidecar + env passthrough |
| `.env.example` | Operator docs | **S1** — `FORECAST_ML_ENABLED`, `STATS_FORECAST_URL` |
| `scripts/compose-config-check.sh` | CI compose assert | **S1/S4** — 3-service + traefik assert |
| `backend/src/config/mod.rs` | Env merge | **Verify** — `FORECAST_ML_ENABLED`, `STATS_FORECAST_URL` |
| `backend/src/sync/mod.rs` | Sync mutex + `forecast_ml` phase | **S2 verify** |
| `backend/src/forecast_ml/{service,sidecar}.rs` | ML orchestration + health | **S2 verify** |
| `backend/src/api/{forecast,wealth}.rs` | ML variant + portfolio API | **S2/S3 verify** |
| `frontend/src/pages/ForecastPage.tsx` | Compare + degraded UX | **S3 verify** |
| `frontend/src/pages/WealthPage.tsx` | Portfolio forecast horizons | **S3 verify** |
| `grafana/provisioning/dashboards/analytics/forecast-horizons.json` | ML panels | **S3 verify** |
| `backend/tests/forecast_ml_integration.rs` | Wiremock sidecar test | **S4 retain** |
| `docs/engineering/runbook.md` | Operator procedures | **S4** — new section |
| `docs/user-guides/US-0013.md` | Operator guide | **S4** — created at architecture |

### Recommended sprint S0014 (slices — sprint-plan materializes tasks)

| Slice | Boundary | Tasks (est.) | Acceptance rows |
|-------|----------|--------------|-----------------|
| **US-0013-S1** | Compose + env + CI assert | ~4 | AC-1 |
| **US-0013-S2** | Sync verify + API `ml_enhanced` | ~2 | AC-2, AC-3, AC-4 |
| **US-0013-S3** | React + Grafana verify | ~3 | AC-5, AC-6, AC-7 |
| **US-0013-S4** | Runbook + integration test | ~2 | AC-8, AC-9 |

**Count:** ~11 tasks (< `SPRINT_MAX_TASKS` 12) → **single sprint S0014**; no split.

**Sequencing:** S1 before S2 (sidecar must start); S2 before S3 (data prerequisite); S4 may parallel S3 after S1 lands.

### Test strategy

| Check | Type | Pass criteria |
|-------|------|---------------|
| Compose assert | `scripts/compose-config-check.sh` | 3 external services; stats-forecast on traefik |
| Integration | `cargo test --test forecast_ml_integration` | Wiremock sidecar + skip metadata |
| API smoke | Manual or integration | `variant=ml_enhanced` non-empty after Full sync |
| UI smoke | Operator V1 | Compare tab + wealth horizons on omniflow |
| Grafana | Panel query | `$forecast_variant=ml_enhanced` returns data |
| Profile guard | compose-config-check | `minimal+external` still excludes firefly-iii |

### Triad check (architecture phase)

| Surface | Check | Result |
|---------|-------|--------|
| `docs/product/backlog.md#US-0013` | Discovery + research resolution linked | pass |
| `docs/product/acceptance.md` § US-0013 | 10 rows unchanged; mapped to S1–S4 | pass |
| `docker-compose.external.yml` + `compose-config-check.sh` | Gap documented in codebase map | pass |
| `backend/src/forecast_ml/` + `sync/mod.rs` | Verify-first paths documented | pass |
| R-0071 | 5/5 discovery questions resolved; DEC-0076 formalized | pass |

`triad_hot_surface`: post-write `--check` required; architecture § US-0013 appended; DEC-0076 formalized; spec-pack + user guide created.

### Decisions (US-0013)

| ID | Topic | Summary |
|----|-------|---------|
| DEC-0076 | External ML compose contract | Overlay additive `external` profile on `stats-forecast`; traefik network; env opt-in; dual CI guard |

Full record: `decisions/DEC-0076.md`

### Risks

| Risk | Mitigation |
|------|------------|
| Cold-start race (first sync skips ML) | Runbook: re-sync after health OK; DEC-0052 skip acceptable |
| CI drift (compose-check not updated with overlay) | Atomic PR: S1 compose + compose-check together |
| Host memory (sidecar RSS on shared omniflow) | Monitor; R-0044 footprint bounded; runbook note |
| `minimal+external` profile regression | DEC-0056 guard unchanged in compose-config-check |
| Short mirror history | `insufficient_history` skip; Full sync prerequisite in runbook |
| FX incomplete crypto portfolio | `portfolio_forecast_low_confidence` banner — not block (R-0034) |

### Acceptance mapping

| Row | Architecture slice | Verify |
|-----|-------------------|--------|
| AC-1 | S1 | External overlay starts sidecar; env documented |
| AC-2 | S2 | Sidecar health gate before ML phase |
| AC-3 | S2 | Sync ML phase + skip metadata; UI phase label |
| AC-4 | S2 | `ml_enhanced` persisted; API variant returns series |
| AC-5 | S3 | Forecast Compare overlay |
| AC-6 | S3 | Wealth ML portfolio overlay; signed totals |
| AC-7 | S3 | Grafana ML panels with data |
| AC-8 | S4 | Runbook omniflow ML section |
| AC-9 | S4 | CI wiremock + compose assert |
| Prerequisite | — | BUG-0010 AA/AB/AC DONE (checked) |

### Next phase

`/sprint-plan` **S0014** — materialize US-0013-S1..S4 tasks from slice table; S1-before-S2 sequencing frozen; then `/plan-verify` → `/execute`.

---

