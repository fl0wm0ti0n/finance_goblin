-- US-0004 plan engine (requires TimescaleDB from migration 002)

DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_extension WHERE extname = 'timescaledb') THEN
        RAISE EXCEPTION 'TimescaleDB extension is required for plan_daily_cashflow hypertable.';
    END IF;
END $$;

CREATE TYPE plan_template AS ENUM (
    'current',
    'leasing',
    'savings_mode',
    'house_purchase',
    'custom'
);

CREATE TYPE plan_adjustment_direction AS ENUM (
    'add_outflow',
    'remove_outflow',
    'add_inflow',
    'remove_inflow'
);

CREATE TYPE plan_adjustment_frequency AS ENUM (
    'monthly',
    'weekly',
    'quarterly',
    'one_time'
);

CREATE TYPE plan_adjustment_target AS ENUM (
    'household',
    'subscription',
    'category',
    'custom_label'
);

CREATE TABLE plans (
    id                UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name              TEXT NOT NULL,
    template          plan_template NOT NULL DEFAULT 'custom',
    is_active         BOOLEAN NOT NULL DEFAULT false,
    created_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at        TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE UNIQUE INDEX plans_one_active ON plans (is_active) WHERE is_active = true;

CREATE TABLE plan_versions (
    id                        UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    plan_id                   UUID NOT NULL REFERENCES plans(id) ON DELETE CASCADE,
    version_number            INT NOT NULL CHECK (version_number BETWEEN 1 AND 3),
    is_latest                 BOOLEAN NOT NULL DEFAULT true,
    frozen_at                 TIMESTAMPTZ,
    baseline_computation_id   UUID REFERENCES forecast_computations(id),
    created_at                TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at                TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (plan_id, version_number)
);

CREATE UNIQUE INDEX plan_versions_one_latest ON plan_versions (plan_id) WHERE is_latest = true;

CREATE TABLE plan_adjustments (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    version_id      UUID NOT NULL REFERENCES plan_versions(id) ON DELETE CASCADE,
    direction       plan_adjustment_direction NOT NULL,
    amount          NUMERIC(18, 2) NOT NULL,
    frequency       plan_adjustment_frequency NOT NULL,
    target_type     plan_adjustment_target NOT NULL,
    target_key      TEXT,
    label           TEXT,
    effective_from  DATE NOT NULL DEFAULT CURRENT_DATE,
    effective_to    DATE,
    sort_order      INT NOT NULL DEFAULT 0,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_plan_adjustments_version ON plan_adjustments (version_id, sort_order);

CREATE TABLE plan_computations (
    id                        UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    version_id                UUID NOT NULL REFERENCES plan_versions(id) ON DELETE CASCADE,
    forecast_computation_id   UUID NOT NULL REFERENCES forecast_computations(id),
    computed_at               TIMESTAMPTZ NOT NULL DEFAULT now(),
    status                    TEXT NOT NULL,
    error_message             TEXT,
    metadata                  JSONB NOT NULL DEFAULT '{}'
);

CREATE INDEX idx_plan_computations_version ON plan_computations (version_id, computed_at DESC);

CREATE TABLE plan_daily_cashflow (
    ts                TIMESTAMPTZ NOT NULL,
    version_id        UUID NOT NULL REFERENCES plan_versions(id) ON DELETE CASCADE,
    computation_id    UUID NOT NULL REFERENCES plan_computations(id) ON DELETE CASCADE,
    planned_net       NUMERIC(18, 2) NOT NULL,
    planned_balance   NUMERIC(18, 2)
);

SELECT create_hypertable(
    'plan_daily_cashflow',
    'ts',
    chunk_time_interval => INTERVAL '7 days',
    if_not_exists => TRUE
);

CREATE INDEX idx_plan_daily_cashflow_lookup
    ON plan_daily_cashflow (version_id, computation_id, ts DESC);
