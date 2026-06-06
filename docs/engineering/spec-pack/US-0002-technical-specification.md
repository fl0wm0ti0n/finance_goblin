# Technical Specification — US-0002

## Overview

US-0002 adds a **Forecast Engine** to the existing Rust/Axum backend, hypertable migrations on external TimescaleDB, forecast REST endpoints, a React `/forecast` page with ECharts, and two Grafana analytics dashboards. Recompute runs inline after successful Firefly sync (DEC-0010).

**Dependency:** US-0001 mirror tables (`accounts`, `transactions`, …), sync scheduler, Grafana datasource.

## Components

### Forecast Engine (`backend/src/forecast/`)

```
ForecastService::recompute(sync_run_id)
  ├─ insert forecast_computations (status=running)
  ├─ for each asset account:
  │    ├─ recurring::detect_patterns(transactions)
  │    ├─ rolling::variable_residual(transactions, window=90d)
  │    ├─ categories::map_to_buckets(transactions)
  │    └─ project::daily_path(balance, recurring, rolling, horizon=730d)
  ├─ repository::write_daily + write_monthly (batch insert)
  ├─ repository::mark_success(computation_id)
  └─ repository::enforce_retention(retention_count=5)
```

| Submodule | Responsibility |
|-----------|----------------|
| `recurring` | Payee grouping, cadence detection, ±5% amount tolerance (R-0006) |
| `rolling` | 3-month daily rate; 95th percentile cap; sparse fallback |
| `categories` | TOML `[forecast.category_buckets]` mapping |
| `project` | Day-by-day balance; milestones; monthly aggregation |
| `repository` | SQLx inserts to hypertables; latest computation queries; retention |
| `service` | Orchestration, error handling, low_confidence flags |

**Transfer exclusion:** Skip internal transfers when building cashflow projection (detect via Firefly transaction type in payload).

### Database migration `002_forecast_hypertables.sql`

```sql
CREATE TABLE forecast_computations (
  id UUID PRIMARY KEY,
  sync_run_id UUID REFERENCES sync_runs(id),
  computed_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  status TEXT NOT NULL,
  error_message TEXT,
  metadata JSONB DEFAULT '{}'  -- e.g. per-account low_confidence
);

CREATE TABLE forecast_balance_daily (
  ts TIMESTAMPTZ NOT NULL,
  account_id TEXT NOT NULL REFERENCES accounts(firefly_id),
  computation_id UUID NOT NULL REFERENCES forecast_computations(id) ON DELETE CASCADE,
  balance NUMERIC(18,2) NOT NULL
);
SELECT create_hypertable('forecast_balance_daily', 'ts',
  chunk_time_interval => INTERVAL '7 days', if_not_exists => TRUE);

CREATE TABLE forecast_cashflow_monthly (
  ts TIMESTAMPTZ NOT NULL,
  account_id TEXT NOT NULL REFERENCES accounts(firefly_id),
  computation_id UUID NOT NULL REFERENCES forecast_computations(id) ON DELETE CASCADE,
  income NUMERIC(18,2) NOT NULL DEFAULT 0,
  fixed_costs NUMERIC(18,2) NOT NULL DEFAULT 0,
  variable_costs NUMERIC(18,2) NOT NULL DEFAULT 0,
  free_cashflow NUMERIC(18,2) NOT NULL DEFAULT 0
);
SELECT create_hypertable('forecast_cashflow_monthly', 'ts',
  chunk_time_interval => INTERVAL '30 days', if_not_exists => TRUE);

CREATE INDEX idx_forecast_balance_daily_lookup
  ON forecast_balance_daily (account_id, computation_id, ts DESC);
CREATE INDEX idx_forecast_cashflow_monthly_lookup
  ON forecast_cashflow_monthly (account_id, computation_id, ts DESC);
CREATE INDEX idx_forecast_computations_computed_at
  ON forecast_computations (computed_at DESC);
```

### Sync integration

Modify `SyncService::execute_run` (`backend/src/sync/mod.rs`):

```rust
// After successful Firefly sync:
if let Err(e) = self.forecast.recompute(run_id).await {
    tracing::warn!(?e, "forecast recompute failed; serving stale snapshot");
}
// Then clear active_run mutex
```

`AppState` adds `forecast: ForecastService`.

### Backend API routes

| Method | Path | Handler |
|--------|------|---------|
| GET | `/api/v1/forecast/meta` | Latest computation metadata |
| GET | `/api/v1/forecast/accounts` | Asset accounts list |
| GET | `/api/v1/forecast/daily` | Daily series + milestones |
| GET | `/api/v1/forecast/monthly` | Monthly cashflow series |
| GET | `/api/v1/forecast/long-term` | Horizon-filtered balance path |
| GET | `/api/v1/forecast/aggregate` | Optional cross-account sum |

All routes: JWT middleware (existing), 404 if no successful computation yet.

### Frontend (`frontend/src/pages/ForecastPage.tsx`)

| Element | Detail |
|---------|--------|
| Route | `/forecast` — enable nav in `AppLayout.tsx` |
| Account | shadcn Select → `GET /forecast/accounts` |
| Horizons | Tabs: Daily \| Monthly \| Long-term |
| Long-term pills | 3, 6, 12, 24 months |
| Charts | ECharts: line (daily/long-term), grouped bar (monthly) |
| Meta | "Last computed" from `/forecast/meta` |
| Empty state | No transactions synced |

Dependencies: `echarts`, `echarts-for-react` (or project-standard wrapper).

### Grafana provisioning

| File | Change |
|------|--------|
| `grafana/provisioning/datasources/postgres.yaml` | Add `uid: FlowFinancePostgreSQL` |
| `grafana/provisioning/dashboards/analytics/cashflow.json` | Dashboard 1, uid `cashflow` |
| `grafana/provisioning/dashboards/analytics/forecast-horizons.json` | Dashboard 5, uid `forecast-horizons` |
| `grafana/provisioning/dashboards/dashboards.yaml` | Add Analytics folder provider or subpath |

Panel queries use latest successful `computation_id` subquery (R-0008). Scarcity threshold: static €200 (DEC-0012).

### Config (`config/default.toml` + env)

```toml
[forecast]
rolling_window_days = 90
sparse_history_days = 90
retention_count = 5
recurring_amount_tolerance_pct = 5.0
```

## Interfaces

### Forecast API response shapes (conceptual)

**GET /api/v1/forecast/meta**

```json
{
  "computation_id": "uuid",
  "computed_at": "2026-05-31T12:00:00Z",
  "stale": false,
  "low_confidence": false,
  "sync_run_id": "uuid"
}
```

**GET /api/v1/forecast/daily?account_id=1**

```json
{
  "milestones": {
    "tomorrow": "1234.56",
    "next_week": "1200.00",
    "month_end": "1150.00"
  },
  "series": [{ "date": "2026-06-01", "balance": "1234.56" }]
}
```

**GET /api/v1/forecast/monthly?account_id=1**

```json
{
  "series": [{
    "month": "2026-06-01",
    "income": "3000.00",
    "fixed_costs": "1200.00",
    "variable_costs": "800.00",
    "free_cashflow": "1000.00"
  }]
}
```

## Non-functional

| Attribute | Target |
|-----------|--------|
| Recompute | Complete within sync mutex; log duration per account |
| API latency | Read-only DB queries; <500ms typical for single account |
| Storage | Bounded by retention_count × accounts × ~730 daily points |
| Accuracy | `low_confidence` when <90 days history; no silent failure |
| Security | JWT on all forecast routes; no Firefly writes |
| Observability | Trace forecast recompute; optional metric on Platform Health |
| Testing | Unit tests for recurring/rolling/project; integration test with fixture transactions |

## Dependencies

- **Upstream:** US-0001 (released) — mirror schema, sync, Grafana datasource
- **Downstream:** US-0003 (subscription-adjusted forecasts), US-0006 (`get_forecast` tool), US-0009 (ML overlay on Dashboard 5)

**References:** R-0006, R-0007, R-0008, DEC-0007–DEC-0012, `docs/engineering/architecture.md#us-0002`.
