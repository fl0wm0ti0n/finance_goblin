-- BUG-0008 / DEC-0071: fingerprint dedup for subscription_alerts
ALTER TABLE subscription_alerts ADD COLUMN IF NOT EXISTS fingerprint TEXT;

-- Backfill new_detection fingerprints
UPDATE subscription_alerts
SET fingerprint = 'sub_alert:new_detection:' || pattern_id::text
WHERE alert_type = 'new_detection'
  AND pattern_id IS NOT NULL
  AND fingerprint IS NULL;

-- Backfill price_change fingerprints (direction from title, amount from body)
UPDATE subscription_alerts
SET fingerprint = 'sub_alert:price_change:' || pattern_id::text || ':' ||
    CASE WHEN title ILIKE '%increase%' THEN 'increase' ELSE 'decrease' END || ':' ||
    COALESCE(
        NULLIF(
            regexp_replace(
                COALESCE(substring(body FROM 'to €([0-9.]+)'), '0'),
                '[^0-9.]',
                '',
                'g'
            ),
            ''
        ),
        '0.00'
    )
WHERE alert_type = 'price_change'
  AND pattern_id IS NOT NULL
  AND fingerprint IS NULL;

-- Backfill interval_change fingerprints
UPDATE subscription_alerts
SET fingerprint = 'sub_alert:interval_change:' || pattern_id::text || ':' ||
    COALESCE((regexp_match(COALESCE(body, ''), '(\d+)\s*days'))[1], '0')
WHERE alert_type = 'interval_change'
  AND pattern_id IS NOT NULL
  AND fingerprint IS NULL;

-- Legacy rows without pattern_id
UPDATE subscription_alerts
SET fingerprint = 'sub_alert:legacy:' || id::text
WHERE fingerprint IS NULL;

-- Dedupe unread duplicates: keep newest per fingerprint, mark-read the rest
WITH ranked AS (
    SELECT id,
           ROW_NUMBER() OVER (PARTITION BY fingerprint ORDER BY created_at DESC) AS rn
    FROM subscription_alerts
    WHERE read_at IS NULL
)
UPDATE subscription_alerts sa
SET read_at = NOW()
FROM ranked r
WHERE sa.id = r.id
  AND r.rn > 1;

ALTER TABLE subscription_alerts ALTER COLUMN fingerprint SET NOT NULL;

CREATE UNIQUE INDEX subscription_alerts_unread_fingerprint
    ON subscription_alerts (fingerprint)
    WHERE read_at IS NULL;
