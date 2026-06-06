-- US-0007 exchange portfolio schema

CREATE TABLE exchange_connections (
    id                TEXT PRIMARY KEY,
    enabled           BOOLEAN NOT NULL DEFAULT false,
    connection_state  TEXT NOT NULL DEFAULT 'not_configured',
    last_sync_at      TIMESTAMPTZ,
    last_error        TEXT,
    payload           JSONB NOT NULL DEFAULT '{}'
);

CREATE TABLE exchange_sync_state (
    exchange_id       TEXT PRIMARY KEY REFERENCES exchange_connections(id) ON DELETE CASCADE,
    watermarks        JSONB NOT NULL DEFAULT '{}',
    updated_at        TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE exchange_holdings (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    exchange_id         TEXT NOT NULL REFERENCES exchange_connections(id) ON DELETE CASCADE,
    asset               TEXT NOT NULL,
    quantity            NUMERIC(24, 8) NOT NULL,
    market_value_eur    NUMERIC(18, 2),
    unrealized_pnl_eur  NUMERIC(18, 2),
    avg_cost_eur        NUMERIC(18, 8),
    product_type        TEXT NOT NULL DEFAULT 'spot',
    synced_at           TIMESTAMPTZ NOT NULL DEFAULT now(),
    payload             JSONB NOT NULL DEFAULT '{}',
    UNIQUE (exchange_id, asset, product_type)
);

CREATE INDEX idx_exchange_holdings_exchange ON exchange_holdings (exchange_id);

CREATE TABLE exchange_trades (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    exchange_id     TEXT NOT NULL REFERENCES exchange_connections(id) ON DELETE CASCADE,
    external_id     TEXT NOT NULL,
    symbol          TEXT NOT NULL,
    side            TEXT NOT NULL,
    quantity        NUMERIC(24, 8) NOT NULL,
    price           NUMERIC(24, 8) NOT NULL,
    fee             NUMERIC(18, 8),
    fee_asset       TEXT,
    realized_pnl    NUMERIC(18, 8),
    executed_at     TIMESTAMPTZ NOT NULL,
    payload         JSONB NOT NULL DEFAULT '{}',
    UNIQUE (exchange_id, external_id)
);

CREATE INDEX idx_exchange_trades_exchange_time ON exchange_trades (exchange_id, executed_at DESC);

CREATE TABLE exchange_transfers (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    exchange_id     TEXT NOT NULL REFERENCES exchange_connections(id) ON DELETE CASCADE,
    external_id     TEXT NOT NULL,
    transfer_type   TEXT NOT NULL,
    asset           TEXT NOT NULL,
    quantity        NUMERIC(24, 8) NOT NULL,
    status          TEXT NOT NULL DEFAULT 'completed',
    executed_at     TIMESTAMPTZ NOT NULL,
    payload         JSONB NOT NULL DEFAULT '{}',
    UNIQUE (exchange_id, external_id)
);

CREATE TABLE exchange_funding_events (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    exchange_id     TEXT NOT NULL REFERENCES exchange_connections(id) ON DELETE CASCADE,
    external_id     TEXT NOT NULL,
    symbol          TEXT,
    amount            NUMERIC(18, 8) NOT NULL,
    asset           TEXT NOT NULL,
    event_type      TEXT NOT NULL DEFAULT 'funding_fee',
    executed_at     TIMESTAMPTZ NOT NULL,
    payload         JSONB NOT NULL DEFAULT '{}',
    UNIQUE (exchange_id, external_id)
);

CREATE TABLE portfolio_pnl_snapshots (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    snapshot_date       DATE NOT NULL,
    sync_run_id         UUID REFERENCES sync_runs(id),
    realized_pnl_eur    NUMERIC(18, 2) NOT NULL DEFAULT 0,
    unrealized_pnl_eur  NUMERIC(18, 2) NOT NULL DEFAULT 0,
    total_return_pct    NUMERIC(8, 4),
    crypto_value_eur    NUMERIC(18, 2) NOT NULL DEFAULT 0,
    payload             JSONB NOT NULL DEFAULT '{}',
    UNIQUE (snapshot_date)
);

CREATE TABLE portfolio_baselines (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    exchange_id     TEXT NOT NULL REFERENCES exchange_connections(id) ON DELETE CASCADE,
    baseline_eur    NUMERIC(18, 2) NOT NULL,
    captured_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    sync_run_id     UUID REFERENCES sync_runs(id),
    UNIQUE (exchange_id)
);

CREATE TABLE fx_rates (
    rate_date   DATE NOT NULL,
    base        TEXT NOT NULL,
    quote       TEXT NOT NULL DEFAULT 'EUR',
    rate        NUMERIC(18, 8) NOT NULL,
    provider    TEXT NOT NULL DEFAULT 'frankfurter',
    PRIMARY KEY (rate_date, base, quote)
);

ALTER TABLE net_worth_snapshots
    ADD COLUMN IF NOT EXISTS crypto_value_eur NUMERIC(18, 2) NOT NULL DEFAULT 0,
    ADD COLUMN IF NOT EXISTS firefly_value_eur NUMERIC(18, 2) NOT NULL DEFAULT 0,
    ADD COLUMN IF NOT EXISTS total_return_pct NUMERIC(8, 4);

ALTER TYPE plan_adjustment_target ADD VALUE IF NOT EXISTS 'allocation_target';

ALTER TYPE plan_template ADD VALUE IF NOT EXISTS 'allocation_target';

INSERT INTO exchange_connections (id) VALUES ('binance'), ('bybit'), ('bitunix')
ON CONFLICT DO NOTHING;
