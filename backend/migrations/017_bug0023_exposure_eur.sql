-- BUG-0023: display-only linear exposure in EUR (DEC-0064 / DEC-0081)

ALTER TABLE exchange_holdings
    ADD COLUMN IF NOT EXISTS exposure_eur NUMERIC(18, 2);
