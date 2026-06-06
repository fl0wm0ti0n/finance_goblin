-- US-0009 forecast ML schema (requires TimescaleDB from migration 001)

ALTER TABLE forecast_computations
  ADD COLUMN IF NOT EXISTS model_kind TEXT NOT NULL DEFAULT 'baseline',
  ADD COLUMN IF NOT EXISTS paired_baseline_id UUID REFERENCES forecast_computations(id);

CREATE INDEX IF NOT EXISTS idx_forecast_computations_kind_computed
  ON forecast_computations (model_kind, computed_at DESC)
  WHERE status = 'success';

ALTER TABLE forecast_balance_daily
  ADD COLUMN IF NOT EXISTS balance_p10 NUMERIC(18,2),
  ADD COLUMN IF NOT EXISTS balance_p90 NUMERIC(18,2);

CREATE TABLE IF NOT EXISTS forecast_portfolio_weekly (
  ts TIMESTAMPTZ NOT NULL,
  computation_id UUID NOT NULL REFERENCES forecast_computations(id) ON DELETE CASCADE,
  value_eur NUMERIC(18,2) NOT NULL,
  value_p10 NUMERIC(18,2),
  value_p90 NUMERIC(18,2)
);

SELECT create_hypertable(
  'forecast_portfolio_weekly',
  'ts',
  chunk_time_interval => INTERVAL '30 days',
  if_not_exists => TRUE
);

CREATE INDEX IF NOT EXISTS idx_forecast_portfolio_weekly_lookup
  ON forecast_portfolio_weekly (computation_id, ts DESC);

CREATE TABLE IF NOT EXISTS plan_risk_scores (
  plan_computation_id UUID PRIMARY KEY REFERENCES plan_computations(id) ON DELETE CASCADE,
  score SMALLINT NOT NULL CHECK (score BETWEEN 0 AND 100),
  band TEXT NOT NULL,
  components JSONB NOT NULL DEFAULT '{}',
  computed_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
