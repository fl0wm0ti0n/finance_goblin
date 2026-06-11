-- BUG-0020 / DEC-0109: one-time reconcile (DA1) + confirmed display_category backfill (DB1)
-- Idempotent: safe to re-run; clusters with ≤1 active row per contract are no-ops.

BEGIN;

-- === DA1: YouTube confirmed merge (same display_name, interval_matches ±3d) ===
DO $$
DECLARE
  cluster RECORD;
  survivor_id UUID;
  loser RECORD;
  best_payee_key TEXT;
  best_fingerprint TEXT;
  best_last_seen DATE;
  best_amount NUMERIC(18,2);
BEGIN
  FOR cluster IN
    SELECT display_name
    FROM subscription_patterns
    WHERE status = 'confirmed'
    GROUP BY display_name
    HAVING COUNT(*) > 1
       AND MAX(interval_days) - MIN(interval_days) <= 3
  LOOP
    SELECT id INTO survivor_id
    FROM subscription_patterns
    WHERE status = 'confirmed'
      AND display_name = cluster.display_name
    ORDER BY confirmed_at ASC NULLS LAST, created_at ASC
    LIMIT 1;

    FOR loser IN
      SELECT id, payee_key, fingerprint, last_seen_at, current_amount
      FROM subscription_patterns
      WHERE status = 'confirmed'
        AND display_name = cluster.display_name
        AND id <> survivor_id
    LOOP
      INSERT INTO subscription_pattern_transactions (pattern_id, transaction_firefly_id)
      SELECT survivor_id, transaction_firefly_id
      FROM subscription_pattern_transactions
      WHERE pattern_id = loser.id
      ON CONFLICT DO NOTHING;

      DELETE FROM subscription_pattern_transactions
      WHERE pattern_id = loser.id;

      UPDATE subscription_alerts
      SET pattern_id = survivor_id
      WHERE pattern_id = loser.id;

      INSERT INTO subscription_pattern_tags (pattern_id, tag_id)
      SELECT survivor_id, tag_id
      FROM subscription_pattern_tags
      WHERE pattern_id = loser.id
      ON CONFLICT DO NOTHING;

      DELETE FROM subscription_pattern_tags
      WHERE pattern_id = loser.id;

      IF loser.last_seen_at >= (
        SELECT last_seen_at FROM subscription_patterns WHERE id = survivor_id
      ) THEN
        best_payee_key := loser.payee_key;
        best_fingerprint := loser.fingerprint;
        best_last_seen := loser.last_seen_at;
        best_amount := loser.current_amount;
      END IF;

      UPDATE subscription_patterns
      SET fingerprint = 'merged:' || id::text,
          status = 'inactive',
          updated_at = NOW()
      WHERE id = loser.id;
    END LOOP;

    IF best_payee_key IS NOT NULL THEN
      UPDATE subscription_patterns
      SET payee_key = best_payee_key,
          fingerprint = best_fingerprint,
          last_seen_at = GREATEST(last_seen_at, best_last_seen),
          current_amount = best_amount,
          updated_at = NOW()
      WHERE id = survivor_id;
    END IF;
  END LOOP;
END $$;

-- === DA1: Strom pending collapse (same display_name, interval_matches ±3d) ===
DO $$
DECLARE
  cluster RECORD;
  survivor_id UUID;
  loser RECORD;
BEGIN
  FOR cluster IN
    SELECT display_name
    FROM subscription_patterns
    WHERE status = 'pending'
    GROUP BY display_name
    HAVING COUNT(*) > 1
       AND MAX(interval_days) - MIN(interval_days) <= 3
  LOOP
    SELECT id INTO survivor_id
    FROM subscription_patterns
    WHERE status = 'pending'
      AND display_name = cluster.display_name
    ORDER BY last_seen_at DESC, created_at DESC
    LIMIT 1;

    FOR loser IN
      SELECT id
      FROM subscription_patterns
      WHERE status = 'pending'
        AND display_name = cluster.display_name
        AND id <> survivor_id
    LOOP
      INSERT INTO subscription_pattern_transactions (pattern_id, transaction_firefly_id)
      SELECT survivor_id, transaction_firefly_id
      FROM subscription_pattern_transactions
      WHERE pattern_id = loser.id
      ON CONFLICT DO NOTHING;

      DELETE FROM subscription_pattern_transactions
      WHERE pattern_id = loser.id;

      UPDATE subscription_alerts
      SET pattern_id = survivor_id
      WHERE pattern_id = loser.id;

      INSERT INTO subscription_pattern_tags (pattern_id, tag_id)
      SELECT survivor_id, tag_id
      FROM subscription_pattern_tags
      WHERE pattern_id = loser.id
      ON CONFLICT DO NOTHING;

      DELETE FROM subscription_pattern_tags
      WHERE pattern_id = loser.id;

      UPDATE subscription_patterns
      SET status = 'rejected',
          rejected_at = COALESCE(rejected_at, NOW()),
          updated_at = NOW()
      WHERE id = loser.id;
    END LOOP;
  END LOOP;
END $$;

-- === DB1: confirmed display_category_id backfill (DEC-0100 RANK SQL) ===
UPDATE subscription_patterns p
SET display_category_id = sub.category_id,
    updated_at = NOW()
FROM (
  SELECT p2.id AS pattern_id,
         (
           WITH linked AS (
             SELECT t.category_id, t.date
             FROM subscription_pattern_transactions spt
             JOIN transactions t ON t.firefly_id = spt.transaction_firefly_id
             WHERE spt.pattern_id = p2.id AND t.category_id IS NOT NULL
           ),
           ranked AS (
             SELECT category_id,
                    RANK() OVER (ORDER BY COUNT(*) DESC, MAX(date) DESC) AS rnk
             FROM linked
             GROUP BY category_id
           )
           SELECT category_id FROM ranked WHERE rnk = 1 LIMIT 1
         ) AS category_id
  FROM subscription_patterns p2
  WHERE p2.status = 'confirmed'
) sub
WHERE p.id = sub.pattern_id
  AND sub.category_id IS NOT NULL
  AND (p.display_category_id IS DISTINCT FROM sub.category_id);

COMMIT;
