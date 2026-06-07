-- US-0015 monthly bucket provenance (DEC-0078)

ALTER TABLE forecast_cashflow_monthly
    ADD COLUMN IF NOT EXISTS bucket_sources JSONB,
    ADD COLUMN IF NOT EXISTS ai_mapped BOOLEAN NOT NULL DEFAULT FALSE;
