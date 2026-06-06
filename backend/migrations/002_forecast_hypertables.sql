-- US-0002 forecast hypertables (requires TimescaleDB extension from migration 001)

DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_extension WHERE extname = 'timescaledb') THEN
        RAISE EXCEPTION 'TimescaleDB extension is required for forecast hypertables. Run: CREATE EXTENSION IF NOT EXISTS timescaledb;';
    END IF;
END $$;

CREATE TABLE IF NOT EXISTS forecast_computations (
    id UUID PRIMARY KEY,
    sync_run_id UUID REFERENCES sync_runs(id),
    computed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    status TEXT NOT NULL,
    error_message TEXT,
    metadata JSONB NOT NULL DEFAULT '{}'
);

CREATE INDEX IF NOT EXISTS idx_forecast_computations_computed_at
    ON forecast_computations (computed_at DESC);

CREATE TABLE IF NOT EXISTS forecast_balance_daily (
    ts TIMESTAMPTZ NOT NULL,
    account_id TEXT NOT NULL REFERENCES accounts(firefly_id),
    computation_id UUID NOT NULL REFERENCES forecast_computations(id) ON DELETE CASCADE,
    balance NUMERIC(18, 2) NOT NULL
);

SELECT create_hypertable(
    'forecast_balance_daily',
    'ts',
    chunk_time_interval => INTERVAL '7 days',
    if_not_exists => TRUE
);

CREATE TABLE IF NOT EXISTS forecast_cashflow_monthly (
    ts TIMESTAMPTZ NOT NULL,
    account_id TEXT NOT NULL REFERENCES accounts(firefly_id),
    computation_id UUID NOT NULL REFERENCES forecast_computations(id) ON DELETE CASCADE,
    income NUMERIC(18, 2) NOT NULL DEFAULT 0,
    fixed_costs NUMERIC(18, 2) NOT NULL DEFAULT 0,
    variable_costs NUMERIC(18, 2) NOT NULL DEFAULT 0,
    free_cashflow NUMERIC(18, 2) NOT NULL DEFAULT 0
);

SELECT create_hypertable(
    'forecast_cashflow_monthly',
    'ts',
    chunk_time_interval => INTERVAL '30 days',
    if_not_exists => TRUE
);

CREATE INDEX IF NOT EXISTS idx_forecast_balance_daily_lookup
    ON forecast_balance_daily (computation_id, account_id, ts DESC);

CREATE INDEX IF NOT EXISTS idx_forecast_cashflow_monthly_lookup
    ON forecast_cashflow_monthly (computation_id, account_id, ts DESC);
