-- BUG-0015 Q0023 AU2 — payee+status lookup for confirm inheritance (DEC-0085)
CREATE INDEX IF NOT EXISTS idx_subscription_patterns_payee_status
    ON subscription_patterns (payee_key, status);
