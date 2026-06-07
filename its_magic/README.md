# Flow Finance AI

Personal finance analytics for households that sync Firefly III transactions, project
forecasts, embed Grafana dashboards, and expose an AI assistant for spending insights.

## Purpose

Flow Finance AI mirrors your Firefly ledger into a TimescaleDB-backed analytics stack,
runs scheduled sync and exchange portfolio updates, and surfaces wealth, forecast, and
subscription intelligence through a React SPA plus embedded Grafana. Operators deploy with
Docker Compose (greenfield or omniflow external profile); contributors extend the Rust
backend, React frontend, and provisioning under the its-magic phased workflow.

### Product status

Recent closed work (newest first). Full history: [`docs/product/backlog.md`](docs/product/backlog.md).

- US-0017 — README expansion: omniflow smoke curls, troubleshooting symptom table, per-segment Product status maintenance hooks (Q0021)
- BUG-0013 — Omniflow analytics regression: budgets MTD cap, Bitunix crypto pricing, Grafana copy (Q0020)
- US-0015 — AI-assisted forecast bucket mapping: config→rule→LLM cascade, monthly provenance API, AI-mapped badge (S0016)
- US-0014 — Planning UX polish: mutation feedback, onboarding confirmations, operator-visible errors (S0015)
- US-0013 — Production ML hardening: external stats-forecast overlay, opt-in ML sync, Compare + Grafana parity (S0014)
- BUG-0011 — Planning mode fixes: empty-plan UX, overlay-only compare deltas, PVA 200 no_active_plan (Q0019)
- BUG-0008 — Subscription alert dedup, reconciled unread-count API, and detection recall (Q0018)
- US-0016 — Root README living documentation for operators and contributors (S0013)
- BUG-0007 — AI merchant/category discovery fixed via category_search and subscription enumeration (Q0017)
- BUG-0009 — Grafana default account selection and portfolio overview restored (Q0016)

## Quickstart

Copy the environment template and edit values locally — never commit real credentials:

```bash
cp .env.example .env
# Edit .env: database host, Firefly PAT (when syncing), OIDC URLs, Grafana root URL, etc.
```

Choose a Compose entry profile (see comments at the top of `.env.example`):

| Profile | When to use | Command |
|---------|-------------|---------|
| **Minimal** | App + Grafana only; external or host-managed Firefly/Postgres | `docker compose --profile minimal up --build` |
| **Bundled Firefly** | Greenfield stack with Firefly III in Compose | `docker compose --profile minimal --profile bundled-firefly up --build` |
| **External omniflow** | Shared homelab host on Traefik (`financegnome.omniflow.cc`) | `docker compose -f docker-compose.yml -f docker-compose.external.yml --profile external up -d` |

**External omniflow notes:** set `DATABASE_HOST=postgres` (not `host.docker.internal`) and
configure `FIREFLY_PERSONAL_ACCESS_TOKEN` from Firefly → Profile → API tokens. See
[`docs/engineering/runbook.md`](docs/engineering/runbook.md) § Omniflow for mis-host and PAT
troubleshooting.

After containers are healthy, open the SPA (default `http://localhost:8080` or your Traefik
host), complete OIDC login when enabled, and trigger an initial Firefly sync from the Sync page.

## Examples

**Check sync status and entity counts:**

```bash
curl -s http://localhost:8080/api/v1/sync/status | jq .
curl -s http://localhost:8080/api/v1/sync/entities | jq .
```

**Trigger a manual full sync** (requires valid Firefly PAT and running backend):

```bash
curl -s -X POST http://localhost:8080/api/v1/sync/trigger \
  -H 'Content-Type: application/json' \
  -d '{"mode":"full"}' | jq .
```

**Embedded analytics dashboards** (six SPA routes; Grafana proxied at `/analytics/grafana/`):

| Route | Focus |
|-------|--------|
| `/analytics/cashflow` | Cashflow time series |
| `/analytics/subscriptions` | Recurring subscription patterns |
| `/analytics/budgets` | Budget vs actual |
| `/analytics/portfolio` | Cross-account overview and allocation |
| `/analytics/forecast-horizons` | Forecast milestones and ML overlay |
| `/analytics/platform-health` | Stack health panels |

**AI assistant:** open Chat in the SPA and ask about spending categories, subscriptions, or
account totals — responses use aggregate tool data (`allow_raw_transactions=false` by default).

### Omniflow smoke (external profile)

Production host: `https://financegnome.omniflow.cc` (override with `TRAEFIK_HOST` in operator
`.env`). **Edge auth:** Traefik `auth` basic-auth on public routes — use placeholder
`-u '<basic-auth-user>:<pass>'`; never commit credentials. **API auth:** `/api/v1/*` routes
require an OIDC session in the SPA or `AUTH_DEV_BYPASS=true` on the external profile; see
[`docs/engineering/runbook.md`](docs/engineering/runbook.md) § Omniflow AC-6 for the full matrix.

**Operator gates** (run before attributing flat analytics to code defects):
**BACKEND_FRONTEND_DEPLOY** → **GRAFANA_PROVISIONING_RELOAD** → **FULL_FIREFLY_SYNC** + forecast
recompute. Detail: [runbook §23 BUG-0013](docs/engineering/runbook.md#23-bug-0013-hotfix--omniflow-analytics-regression-q0020--released-2026-06-09).

```bash
OMNI=https://financegnome.omniflow.cc
AUTH='-u <basic-auth-user>:<pass>'   # Traefik edge only — replace placeholders

# Health
curl -sf "$OMNI/health" $AUTH

# Sync status + entity counts
curl -s "$OMNI/api/v1/sync/status" | jq .
curl -s "$OMNI/api/v1/sync/entities" | jq .

# Manual Full sync (requires Firefly PAT + running backend)
curl -s -X POST "$OMNI/api/v1/sync/trigger" \
  -H 'Content-Type: application/json' \
  -d '{"mode":"full"}' | jq .

# Forecast recompute signal (after Full sync from SPA or trigger above)
curl -s "$OMNI/api/v1/forecast/meta" | jq '.last_computed_at, .computation_id'

# Exchange / crypto sanity
curl -s "$OMNI/api/v1/wealth" | jq '.crypto.subtotal_eur, .total_eur'

# Grafana embed proxy health
curl -s -o /dev/null -w '%{http_code}\n' "$OMNI/analytics/grafana/api/health" $AUTH
```

**Six analytics routes** (prefix with `$OMNI` for omniflow smoke):

| Route | Smoke focus |
|-------|-------------|
| `/analytics/cashflow` | Baseline balances acct **114** (not **116**) |
| `/analytics/subscriptions` | Price-changes panel or documented empty-state |
| `/analytics/budgets` | MTD planned/actual plausible post-DEC-0079 |
| `/analytics/portfolio` | Crypto stat non-zero post-DEC-0080 |
| `/analytics/forecast-horizons` | Baseline + optional ML banner |
| `/analytics/platform-health` | Stack health |

## Limitations

- **Firefly read-only:** Flow Finance AI syncs from Firefly; it does not write categories or
  transactions back to Firefly.
- **PostgreSQL + TimescaleDB required:** the app expects a Timescale-enabled database; shared
  hosts may need `DATABASE_BOOTSTRAP_URL` when the app role lacks `CREATEDB`.
- **Omniflow external profile:** do not combine `external` with `minimal` / `bundled-firefly`
  profiles; `DATABASE_HOST=host.docker.internal` breaks API health on the Traefik network.
- **ML forecast panels:** when ML forecasting is not enabled (see US-0013), forecast-horizons
  dashboards show an honest "ML unavailable" banner — baseline statistical forecasts still apply.
- **OIDC and edge auth:** production omniflow deploys often use Traefik basic-auth plus optional
  OIDC; misconfigured redirect URIs or empty PATs produce fail-fast sync errors, not silent empty data.
- **Exchange and crypto:** futures/margin exposure depends on configured read-only exchange keys;
  not all venues or account types are supported.

### Troubleshooting

Before attributing analytics failures to application code, complete the operator gate sequence:
**BACKEND_FRONTEND_DEPLOY** → **GRAFANA_PROVISIONING_RELOAD** → **FULL_FIREFLY_SYNC** + forecast
recompute. Row-level probes and deploy commands:
[runbook §23 BUG-0013](docs/engineering/runbook.md#23-bug-0013-hotfix--omniflow-analytics-regression-q0020--released-2026-06-09).

| Symptom | Likely cause | Operator action |
|---------|--------------|-----------------|
| All analytics panels flat **0 €** after deploy | Stale image / gates skipped | Run full gate sequence above |
| Budgets MTD **−€150K** planned, **€0** actual | Pre-DEC-0079 MTD SQL artifact | Deploy DEC-0079 build + Grafana reload |
| Crypto **€0** in wealth/portfolio | Pre-DEC-0080 pricing or exchanges-only sync | Deploy DEC-0080 build + Full sync; probe `crypto.subtotal_eur` |
| Forecast **0 €** on default panels | Wrong `$account_id` or no recompute | Full sync; verify acct **114** (not **116**) |
| **ML unavailable** banner on forecast-horizons | ML overlay off (US-0013 / DEC-0049) | **Expected** — baseline statistical forecast still applies; not data-missing |
| Grafana **Failed to fetch** (browser console) | Embed annotation cancel / WS edge | curl ds/query **200**; Traefik session; do not Save dashboard overrides |

**Distinction:** empty Grafana SQL panels **after** the gate sequence indicate a data or deploy
defect; an **ML unavailable** banner is honest degraded mode when the stats-forecast sidecar is
disabled — not missing ledger data.

## Related documentation

| Resource | Description |
|----------|-------------|
| [`docs/user-guides/`](docs/user-guides/) | Per-feature operator guides (USER_GUIDE_MODE) |
| [`docs/engineering/runbook.md`](docs/engineering/runbook.md) | Canonical operator commands, smoke checks, and TEST_COMMAND |
| [`docs/engineering/architecture.md`](docs/engineering/architecture.md) | Story-level contracts and module boundaries |
| [`docs/engineering/decisions.md`](docs/engineering/decisions.md) | Compact decision index |
| [`docs/engineering/spec-pack/`](docs/engineering/spec-pack/) | Design concepts, CRS, and technical specifications (SPEC_PACK_MODE) |

**Compose entry commands** (same as [Quickstart](#quickstart)):

```bash
# Minimal — app + Grafana
docker compose --profile minimal up --build

# Greenfield with bundled Firefly III
docker compose --profile minimal --profile bundled-firefly up --build

# Omniflow external (two-file merge)
docker compose -f docker-compose.yml -f docker-compose.external.yml --profile external up -d
```

## Contributing

Contributor workflow, quality gates, and developer-channel documentation live in
[`docs/developer/README.md`](docs/developer/README.md).
