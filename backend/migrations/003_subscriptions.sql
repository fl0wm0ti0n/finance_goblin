-- US-0003 subscription intelligence (relational only)
CREATE TYPE subscription_status AS ENUM ('pending', 'confirmed', 'rejected', 'inactive');
CREATE TYPE subscription_kind AS ENUM ('subscription', 'standing_order');
CREATE TYPE subscription_alert_type AS ENUM ('new_detection', 'price_change', 'interval_change');

CREATE TABLE subscription_patterns (
    id                UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    fingerprint       TEXT NOT NULL UNIQUE,
    status            subscription_status NOT NULL DEFAULT 'pending',
    kind              subscription_kind NOT NULL DEFAULT 'subscription',
    payee_key         TEXT NOT NULL,
    display_name      TEXT NOT NULL,
    interval_days     INT NOT NULL,
    current_amount    NUMERIC(18,2) NOT NULL,
    confidence_pct    SMALLINT NOT NULL CHECK (confidence_pct IN (60, 80, 95)),
    first_seen_at     DATE NOT NULL,
    last_seen_at      DATE NOT NULL,
    confirmed_at      TIMESTAMPTZ,
    rejected_at       TIMESTAMPTZ,
    detection_run_id  UUID,
    created_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at        TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE subscription_pattern_transactions (
    pattern_id            UUID NOT NULL REFERENCES subscription_patterns(id) ON DELETE CASCADE,
    transaction_firefly_id TEXT NOT NULL REFERENCES transactions(firefly_id) ON DELETE CASCADE,
    PRIMARY KEY (pattern_id, transaction_firefly_id)
);

CREATE TABLE subscription_price_events (
    id                UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    pattern_id        UUID NOT NULL REFERENCES subscription_patterns(id) ON DELETE CASCADE,
    event_type        TEXT NOT NULL CHECK (event_type IN ('billing', 'price_increase', 'price_decrease', 'interval_change')),
    amount            NUMERIC(18,2) NOT NULL,
    previous_amount   NUMERIC(18,2),
    delta_pct         NUMERIC(8,2),
    interval_days     INT,
    occurred_at       DATE NOT NULL,
    sync_run_id       UUID,
    created_at        TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE subscription_rejections (
    fingerprint       TEXT PRIMARY KEY,
    rejected_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
    reason            TEXT
);

CREATE TABLE subscription_alerts (
    id                UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    pattern_id        UUID REFERENCES subscription_patterns(id) ON DELETE SET NULL,
    alert_type        subscription_alert_type NOT NULL,
    title             TEXT NOT NULL,
    body              TEXT,
    read_at           TIMESTAMPTZ,
    sync_run_id       UUID,
    created_at        TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_subscription_patterns_status_kind ON subscription_patterns (status, kind);
CREATE INDEX idx_subscription_patterns_last_seen ON subscription_patterns (last_seen_at DESC);
CREATE INDEX idx_subscription_price_events_pattern ON subscription_price_events (pattern_id, occurred_at DESC);
CREATE INDEX idx_subscription_alerts_read ON subscription_alerts (read_at, created_at DESC);
