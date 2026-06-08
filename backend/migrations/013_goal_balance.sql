-- US-0019 goal balance plan template (DEC-0091)

ALTER TYPE plan_template ADD VALUE IF NOT EXISTS 'goal_balance';

ALTER TABLE plans
    ADD COLUMN IF NOT EXISTS target_balance_eur NUMERIC(18, 2) NULL,
    ADD COLUMN IF NOT EXISTS target_date DATE NULL,
    ADD COLUMN IF NOT EXISTS goal_account_id TEXT NULL;
