-- US-0005 alerts and wealth snapshots

CREATE TYPE alert_type AS ENUM (
    'scarcity',
    'budget_drift',
    'plan_viability'
);

CREATE TYPE alert_severity AS ENUM (
    'info',
    'warning',
    'critical'
);

CREATE TYPE alert_status AS ENUM (
    'active',
    'acknowledged',
    'dismissed',
    'resolved'
);

CREATE TABLE alert_config (
    id                      INT PRIMARY KEY DEFAULT 1 CHECK (id = 1),
    scarcity_threshold_eur  NUMERIC NOT NULL DEFAULT 200.0,
    budget_drift_pct        NUMERIC NOT NULL DEFAULT 20.0,
    updated_at              TIMESTAMPTZ NOT NULL DEFAULT now()
);

INSERT INTO alert_config DEFAULT VALUES;

CREATE TABLE alerts (
    id                      UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    alert_type              alert_type NOT NULL,
    severity                alert_severity NOT NULL,
    status                  alert_status NOT NULL DEFAULT 'active',
    fingerprint             TEXT NOT NULL,
    title                   TEXT NOT NULL,
    message                 TEXT NOT NULL,
    entity_type             TEXT,
    entity_id               TEXT,
    context                 JSONB NOT NULL DEFAULT '{}',
    triggered_at            TIMESTAMPTZ NOT NULL DEFAULT now(),
    acknowledged_at         TIMESTAMPTZ,
    dismissed_at            TIMESTAMPTZ,
    resolved_at             TIMESTAMPTZ,
    sync_run_id             UUID REFERENCES sync_runs(id)
);

CREATE UNIQUE INDEX alerts_active_fingerprint
    ON alerts (fingerprint)
    WHERE status IN ('active', 'acknowledged');

CREATE INDEX alerts_status_triggered
    ON alerts (status, triggered_at DESC);

CREATE INDEX alerts_unread
    ON alerts (status, acknowledged_at)
    WHERE status = 'active' AND acknowledged_at IS NULL;

CREATE TABLE net_worth_snapshots (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    snapshot_date   DATE NOT NULL UNIQUE,
    total_eur       NUMERIC NOT NULL,
    mixed_currency  BOOLEAN NOT NULL DEFAULT false,
    account_count   INT NOT NULL DEFAULT 0,
    payload         JSONB NOT NULL DEFAULT '[]',
    sync_run_id     UUID REFERENCES sync_runs(id),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX net_worth_snapshots_snapshot_date
    ON net_worth_snapshots (snapshot_date DESC);
