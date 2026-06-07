# Tasks — Sprint S0014

**Story:** US-0013  
**Task count:** 11 (within SPRINT_MAX_TASKS=12)

| ID | Title | Status | Slice | Acceptance refs |
|----|-------|--------|-------|-----------------|
| T-0144 | External overlay stats-forecast sidecar | open | S1 | AC-1 |
| T-0145 | flow-finance-ai ML env passthrough | open | S1 | AC-1 |
| T-0146 | .env.example omniflow ML block | open | S1 | AC-1 |
| T-0147 | Compose config CI guard update | open | S1 | AC-1 |
| T-0148 | Sync ML phase + health gate verify | open | S2 | AC-2, AC-3 |
| T-0149 | ml_enhanced API persistence verify | open | S2 | AC-3, AC-4 |
| T-0150 | ForecastPage Compare + degraded UX verify | open | S3 | AC-5 |
| T-0151 | WealthPage portfolio-forecast verify | open | S3 | AC-6 |
| T-0152 | Grafana forecast-horizons ML panels verify | open | S3 | AC-7 |
| T-0153 | Runbook Omniflow ML enablement | open | S4 | AC-8 |
| T-0154 | CI dual guard — wiremock + compose assert | open | S4 | AC-9 |

---

## T-0144 — External overlay stats-forecast sidecar

**Status:** open  
**Depends on:** —  
**Decisions:** DEC-0076  
**Architecture slice:** US-0013-S1  
**Research:** R-0071

### Description

Append additive overlay to `docker-compose.external.yml` for existing `stats-forecast` service key:

```yaml
stats-forecast:
  profiles: [external]
  networks:
    traefik:
  ports:
    - "${STATS_FORECAST_PORT:-8091}:8090"
```

**Frozen contracts:**

| Element | Value |
|---------|-------|
| Profile union | Overlay adds `[external]` — Compose merges with base `[full]` → one container |
| Network | **Traefik-only** — matches `flow-finance-ai` external merge |
| Port | Host `${STATS_FORECAST_PORT:-8091}` → container `8090` |
| Rejected | Duplicate `stats-forecast:` service block; changing base `profiles: [full]` |

Do **not** add `postgres` or default network attachment.

### Done when

- [ ] `docker compose -f docker-compose.yml -f docker-compose.external.yml --profile external config --services` includes `stats-forecast`
- [ ] Merged config shows `stats-forecast` on `traefik` network with port remap
- [ ] Base `--profile full` behavior unchanged (port 8090 default)
- [ ] No duplicate service key in overlay file

---

## T-0145 — flow-finance-ai ML env passthrough

**Status:** open  
**Depends on:** T-0144  
**Decisions:** DEC-0076, DEC-0049  
**Architecture slice:** US-0013-S1

### Description

Extend `flow-finance-ai` environment block in `docker-compose.external.yml`:

```yaml
environment:
  FORECAST_ML_ENABLED: ${FORECAST_ML_ENABLED:-false}
  STATS_FORECAST_URL: ${STATS_FORECAST_URL:-http://stats-forecast:8090}
```

**Frozen:**

- Default-off preserves DEC-0049 — operator opts in via `.env`
- Internal URL uses container port `8090`, not host remap
- Verify `backend/src/config/mod.rs` merges env into `[forecast_ml]` (no code change expected unless gap found)

### Done when

- [ ] External merge renders both env vars with interpolation defaults
- [ ] `FORECAST_ML_ENABLED=false` when unset (default-off)
- [ ] `STATS_FORECAST_URL=http://stats-forecast:8090` when unset
- [ ] Config merge path documented or verified in code audit note

---

## T-0146 — .env.example omniflow ML block

**Status:** open  
**Depends on:** T-0144  
**Decisions:** DEC-0076  
**Architecture slice:** US-0013-S1  
**Research:** R-0071

### Description

Extend `.env.example` omniflow / external profile section with ML enablement vars:

| Variable | Document |
|----------|----------|
| `FORECAST_ML_ENABLED` | Opt-in; default `false`; set `true` to enable ML overlay |
| `STATS_FORECAST_URL` | Internal sidecar URL; default `http://stats-forecast:8090` |
| `STATS_FORECAST_PORT` | Host debug port; default `8091` on omniflow (8090 clash note) |

Include compose invocation reminder:

```bash
docker compose -f docker-compose.yml -f docker-compose.external.yml --profile external up -d
```

No literal secrets or host-specific credentials.

### Done when

- [ ] All three variables documented with defaults and purpose
- [ ] Port 8090 clash note present (US-0010 intake)
- [ ] Opt-in semantics clear (DEC-0049 default-off)
- [ ] Compose external invocation documented or cross-linked

---

## T-0147 — Compose config CI guard update

**Status:** open  
**Depends on:** T-0144, T-0145  
**Decisions:** DEC-0076, DEC-0056  
**Architecture slice:** US-0013-S1  
**Research:** R-0053 §7, R-0071

### Description

Update `scripts/compose-config-check.sh`:

| Check | Before | After |
|-------|--------|-------|
| External `config --services` | `flow-finance-ai grafana` | `flow-finance-ai grafana stats-forecast` |
| Traefik network membership | flow-finance-ai, grafana | + stats-forecast |

**Unchanged:** DEC-0056 anti-combination guards — `minimal+external` must still exclude `firefly-iii`.

Land atomically with T-0144 overlay in same PR.

### Done when

- [ ] Script passes locally after overlay changes
- [ ] External-only merge asserts exactly 3 services
- [ ] Traefik network assert includes `stats-forecast`
- [ ] `minimal+external` guard still passes (no firefly-iii regression)

---

## T-0148 — Sync ML phase + health gate verify

**Status:** open  
**Depends on:** T-0147  
**Decisions:** DEC-0052, DEC-0066  
**Architecture slice:** US-0013-S2  
**Research:** R-0044, R-0071

### Description

Verify-first audit of existing sync ML pipeline — **no algorithm changes** unless gap found:

| Component | Path | Contract |
|-----------|------|----------|
| Sync phase | `backend/src/sync/mod.rs` | `forecast_ml` runs after baseline; phase label "ML forecast…" |
| Health gate | `backend/src/forecast_ml/sidecar.rs` | `health_ok()` GET `/health` before ML pass |
| Skip metadata | `backend/src/forecast_ml/service.rs` | `record_skip_on_baseline` on failure (DEC-0052) |
| Sidecar SLO | Runtime | 60s HTTP timeout; compose healthcheck advisory (30s start_period) |

If gaps found, minimal fix only — document in task completion note.

Optional: add/adjust unit test for health gate skip path if coverage gap identified.

### Done when

- [ ] Code audit confirms health gate before ML phase
- [ ] Skip metadata path documented or fixed
- [ ] Sync status UI shows "ML forecast…" when phase active (verify in code or smoke)
- [ ] Cold-start skip behavior documented (DEC-0052 acceptable)

---

## T-0149 — ml_enhanced API persistence verify

**Status:** open  
**Depends on:** T-0148  
**Decisions:** DEC-0050, DEC-0051  
**Architecture slice:** US-0013-S2  
**Research:** R-0045

### Description

Verify-first audit of ML persistence and API:

| Component | Path | Contract |
|-----------|------|----------|
| Min history | `default.toml` `[forecast_ml]` | `min_monthly_points = 12` unchanged |
| Persistence | forecast computation store | `model_kind=ml_enhanced` with p10/p90 bands |
| API | `backend/src/api/forecast.rs` | `variant=ml_enhanced` returns 6–24 month series |

Confirm `insufficient_history` skip path when mirror < 12 monthly points.

If integration test gap found, extend `forecast_ml_integration.rs` or add focused unit test — prefer existing wiremock fixture (T-0154).

### Done when

- [ ] `min_monthly_points=12` confirmed unchanged
- [ ] API variant contract verified in code
- [ ] Persistence schema/path documented
- [ ] Skip path for short history documented

---

## T-0150 — ForecastPage Compare + degraded UX verify

**Status:** open  
**Depends on:** T-0149  
**Decisions:** DEC-0066  
**Architecture slice:** US-0013-S3

### Description

Verify-first audit of React forecast ML UX — **no new features** unless gap found:

| Surface | Path | Verify |
|---------|------|--------|
| Compare control | `frontend/src/pages/ForecastPage.tsx` | Baseline + ML overlay when data exists |
| Degraded copy | ForecastPage + meta API | `sidecar_disabled` per DEC-0066 — not generic skip message |
| Three-state UX | ForecastPage | ML not enabled vs skipped vs available |

Document omniflow smoke steps for verify-work UAT.

### Done when

- [ ] Compare segmented control renders baseline + ML when `ml_enhanced` data present
- [ ] `sidecar_disabled` copy path confirmed (DEC-0066)
- [ ] No generic "ML forecast unavailable" when reason is `sidecar_disabled`
- [ ] UAT smoke steps documented in `sprints/S0014/uat.md`

---

## T-0151 — WealthPage portfolio-forecast verify

**Status:** open  
**Depends on:** T-0149  
**Decisions:** DEC-0065  
**Architecture slice:** US-0013-S3  
**Research:** R-0034

### Description

Verify-first audit of wealth ML integration:

| Surface | Path | Verify |
|---------|------|--------|
| Portfolio outlook | `frontend/src/pages/WealthPage.tsx` | ML portfolio horizons when data present |
| Signed totals | Wealth API/UI | DEC-0065 negative asset visibility |
| FX banner | WealthPage | `portfolio_forecast_low_confidence` when FX incomplete (R-0034) |

No new React features unless gap found.

### Done when

- [ ] Portfolio forecast horizons render when ML data present
- [ ] Signed totals and account breakdown paths verified
- [ ] FX incomplete banner path confirmed (warning, not hard block)
- [ ] UAT smoke steps documented

---

## T-0152 — Grafana forecast-horizons ML panels verify

**Status:** open  
**Depends on:** T-0149  
**Decisions:** DEC-0049, DEC-0068  
**Architecture slice:** US-0013-S3

### Description

Verify Grafana ML panel queries in `grafana/provisioning/dashboards/analytics/forecast-horizons.json`:

| Check | Contract |
|-------|----------|
| `$forecast_variant=ml_enhanced` | Panels return data when computations exist |
| ML off banner | BUG-0009 banner remains when ML disabled |
| Default variant | `$forecast_variant` default stays `baseline` (DEC-0068 boundary) |

Provisioning-only unless query bug found.

### Done when

- [ ] ML panel queries use `model_kind='ml_enhanced'` correctly
- [ ] Variable default is `baseline`
- [ ] BUG-0009 empty-state banner still present when ML off
- [ ] Post-enablement smoke step documented for UAT

---

## T-0153 — Runbook Omniflow ML enablement

**Status:** open  
**Depends on:** T-0146  
**Decisions:** DEC-0076  
**Architecture slice:** US-0013-S4  
**Research:** R-0071

### Description

Add subsection **`Omniflow ML enablement (US-0013)`** to `docs/engineering/runbook.md` covering:

| Topic | Content |
|-------|---------|
| Compose profile union | Base + external overlay; `--profile external` invocation |
| Env vars | `FORECAST_ML_ENABLED`, `STATS_FORECAST_URL`, `STATS_FORECAST_PORT` |
| Health probe | `curl localhost:${STATS_FORECAST_PORT:-8091}/health`; runtime gate before ML phase |
| Min history | ≥12 monthly points; Full sync prerequisite |
| Degraded mode | `sidecar_disabled`, `sidecar_unavailable`, `insufficient_history` troubleshooting |
| Cold start | First sync may skip — re-sync after health OK |
| Memory | Sidecar RSS note (R-0044) |

Cross-link `docs/user-guides/US-0013.md`.

### Done when

- [ ] Runbook subsection exists with all topics above
- [ ] User guide cross-linked
- [ ] No secrets or host-specific credentials
- [ ] Full sync + recompute steps documented

---

## T-0154 — CI dual guard — wiremock + compose assert

**Status:** open  
**Depends on:** T-0147, T-0149  
**Decisions:** DEC-0076  
**Architecture slice:** US-0013-S4  
**Research:** R-0071

### Description

Confirm dual CI guard per DEC-0076:

| Guard | Path | Contract |
|-------|------|----------|
| Compose assert | `scripts/compose-config-check.sh` | 3-service external set (T-0147) |
| Wiremock integration | `backend/tests/forecast_ml_integration.rs` | Sidecar invoke + overlay persist |
| Test runner | `tests/run-tests.sh` | Both guards invoked |

Retain existing wiremock test — extend only if T-0148/T-0149 identified coverage gap.

Local verification:

```bash
scripts/compose-config-check.sh
cargo test --test forecast_ml_integration
```

### Done when

- [ ] `compose-config-check.sh` passes in CI path
- [ ] `forecast_ml_integration` passes locally
- [ ] Both wired in `tests/run-tests.sh`
- [ ] No production secrets required for CI path

---

## Execution order (recommended)

1. **S1 compose:** T-0144 → T-0145 → T-0146 → T-0147
2. **S2 verify:** T-0148 → T-0149
3. **S3 verify (parallel after S2):** T-0150 ∥ T-0151 ∥ T-0152
4. **S4 docs/CI:** T-0153 (after T-0146) ∥ T-0154 (after T-0147)
5. **Operator:** BACKEND_COMPOSE_DEPLOY → Full sync → verify-work (UAT)

```text
T-0144 → T-0145 → T-0146 → T-0147
              ↓
         T-0148 → T-0149 → (T-0150 + T-0151 + T-0152)
              ↓
         T-0153 + T-0154
              ↓
    Operator: deploy + Full sync + UAT smoke
```

## Acceptance coverage map

| AC | Tasks | Notes |
|----|-------|-------|
| AC-1 | T-0144, T-0145, T-0146, T-0147 | External overlay + env documented |
| AC-2 | T-0148 | Sidecar health gate before ML phase |
| AC-3 | T-0148, T-0149 | Sync ML phase + skip metadata |
| AC-4 | T-0149 | `ml_enhanced` persisted; API variant |
| AC-5 | T-0150 | Forecast Compare overlay |
| AC-6 | T-0151 | Wealth ML portfolio overlay |
| AC-7 | T-0152 | Grafana ML panels with data |
| AC-8 | T-0153 | Runbook omniflow ML section |
| AC-9 | T-0154 | CI wiremock + compose assert |
| Prerequisite | — | BUG-0010 AA/AB/AC DONE (checked at intake) |

## Operator gate

| Gate | Before | Action |
|------|--------|--------|
| **BACKEND_COMPOSE_DEPLOY** | UAT omniflow smoke | Deploy S1 overlay + env; restart `flow-finance-ai` + `stats-forecast`; set `FORECAST_ML_ENABLED=true`; run Full Firefly sync |

## Split decision

- **Why 11 tasks:** Architecture slices S1(4) + S2(2) + S3(3) + S4(2) = 11; under `SPRINT_MAX_TASKS=12`.
- **Why no V1 task:** Operator smoke captured in UAT placeholder; reduces task count while preserving verify-work gate.
- **Why not split:** S1→S2→S3 dependency chain; incomplete ML path if split across sprints.
