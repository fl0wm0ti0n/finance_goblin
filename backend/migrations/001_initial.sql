-- US-0001 mirror schema (relational only; hypertables deferred to US-0002)
CREATE EXTENSION IF NOT EXISTS timescaledb;

CREATE TABLE IF NOT EXISTS sync_runs (
    id UUID PRIMARY KEY,
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    finished_at TIMESTAMPTZ,
    status TEXT NOT NULL DEFAULT 'running',
    trigger TEXT NOT NULL DEFAULT 'scheduled',
    error_message TEXT
);

CREATE INDEX IF NOT EXISTS idx_sync_runs_started_at ON sync_runs (started_at DESC);

CREATE TABLE IF NOT EXISTS sync_cursors (
    entity_type TEXT PRIMARY KEY,
    last_successful_sync_at TIMESTAMPTZ,
    records_synced BIGINT NOT NULL DEFAULT 0,
    last_error TEXT
);

CREATE TABLE IF NOT EXISTS accounts (
    firefly_id TEXT PRIMARY KEY,
    type TEXT,
    name TEXT,
    iban TEXT,
    currency TEXT,
    balance NUMERIC,
    payload JSONB NOT NULL DEFAULT '{}',
    synced_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS transactions (
    firefly_id TEXT PRIMARY KEY,
    account_id TEXT,
    date DATE,
    amount NUMERIC,
    description TEXT,
    category_id TEXT,
    tag_ids JSONB DEFAULT '[]',
    payload JSONB NOT NULL DEFAULT '{}',
    synced_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_transactions_date ON transactions (date);

CREATE TABLE IF NOT EXISTS categories (
    firefly_id TEXT PRIMARY KEY,
    name TEXT,
    payload JSONB NOT NULL DEFAULT '{}',
    synced_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS budgets (
    firefly_id TEXT PRIMARY KEY,
    name TEXT,
    amount NUMERIC,
    period TEXT,
    payload JSONB NOT NULL DEFAULT '{}',
    synced_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS tags (
    firefly_id TEXT PRIMARY KEY,
    tag TEXT,
    payload JSONB NOT NULL DEFAULT '{}',
    synced_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS piggy_banks (
    firefly_id TEXT PRIMARY KEY,
    name TEXT,
    target_amount NUMERIC,
    current_amount NUMERIC,
    payload JSONB NOT NULL DEFAULT '{}',
    synced_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS firefly_request_audit (
    id BIGSERIAL PRIMARY KEY,
    method TEXT NOT NULL,
    path TEXT NOT NULL,
    status_code INT,
    requested_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_firefly_audit_requested_at ON firefly_request_audit (requested_at DESC);
