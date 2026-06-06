use std::collections::{HashMap, HashSet};

use chrono::Utc;
use uuid::Uuid;

use crate::recurrence::{compute_fingerprint, detect_recurrence_groups, RecurrenceConfig, RecurrenceGroup};
use crate::subscriptions::classify::classify_kind;
use crate::subscriptions::price_change::{
    classify_price_change, delta_pct, PriceChangeConfig, PriceChangeKind,
};
use crate::subscriptions::repository::SubscriptionRepository;
use crate::subscriptions::types::PatternRow;

pub struct DetectionPipeline<'a> {
    repo: &'a SubscriptionRepository,
}

impl<'a> DetectionPipeline<'a> {
    pub fn new(repo: &'a SubscriptionRepository) -> Self {
        Self { repo }
    }

    pub async fn run_candidates(
        &self,
        sync_run_id: Uuid,
        rejections: &HashSet<String>,
        confirmed_fps: &HashSet<String>,
    ) -> Result<Vec<(Uuid, RecurrenceGroup, bool)>, sqlx::Error> {
        let config = self.repo.config();
        let txs = self.repo.load_expense_transactions(config.detection_window_days).await?;
        let recurrence_config = RecurrenceConfig {
            high_tolerance_pct: config.confidence_tolerance_pct.high,
            medium_tolerance_pct: config.confidence_tolerance_pct.medium,
            low_tolerance_pct: config.confidence_tolerance_pct.low,
            min_emit_confidence: 60,
        };

        let groups = detect_recurrence_groups(&txs, &recurrence_config);
        let mut new_detections = Vec::new();

        for group in groups {
            let fingerprint = compute_fingerprint(&group.payee_key, group.interval_days, group.median_amount);
            if rejections.contains(&fingerprint) || confirmed_fps.contains(&fingerprint) {
                continue;
            }

            let kind = classify_kind(&group, config);
            let id = self
                .repo
                .upsert_pending_pattern(&group, &fingerprint, kind, sync_run_id)
                .await?;

            let is_new = self
                .repo
                .insert_alert(
                    Some(id),
                    "new_detection",
                    &format!("New recurring pattern: {}", group.display_name),
                    Some(&format!(
                        "Detected {} every {} days at €{:.2} ({}% confidence)",
                        group.display_name,
                        group.interval_days,
                        group.median_amount.abs(),
                        group.confidence_pct
                    )),
                    sync_run_id,
                )
                .await
                .is_ok();

            new_detections.push((id, group, is_new));
        }

        Ok(new_detections)
    }

    pub async fn process_confirmed(
        &self,
        sync_run_id: Uuid,
    ) -> Result<(), sqlx::Error> {
        let patterns = self.repo.list_confirmed_patterns().await?;
        let price_cfg: PriceChangeConfig = self.repo.config().into();
        let today = Utc::now().date_naive();

        for pattern in patterns {
            self.process_confirmed_pattern(&pattern, sync_run_id, &price_cfg, today)
                .await?;
        }

        Ok(())
    }

    async fn process_confirmed_pattern(
        &self,
        pattern: &PatternRow,
        sync_run_id: Uuid,
        price_cfg: &PriceChangeConfig,
        today: chrono::NaiveDate,
    ) -> Result<(), sqlx::Error> {
        let gap_days = (today - pattern.last_seen_at).num_days();
        let inactive_threshold = pattern.interval_days as i64 * 2;
        if gap_days > inactive_threshold {
            self.repo.mark_inactive(pattern.id).await?;
            return Ok(());
        }

        let amount = pattern.amount_f64();
        let events = self.repo.price_history(pattern.id).await?;
        let last_billing = events.iter().filter(|e| e.event_type == "billing").last();

        if let Some(last) = last_billing {
            let prev = last.amount;
            self.repo
                .append_price_event(
                    pattern.id,
                    "billing",
                    amount,
                    Some(prev),
                    Some(delta_pct(prev, amount)),
                    Some(pattern.interval_days),
                    pattern.last_seen_at,
                    sync_run_id,
                )
                .await?;

            match classify_price_change(prev, amount, price_cfg) {
                PriceChangeKind::Increase => {
                    self.repo
                        .append_price_event(
                            pattern.id,
                            "price_increase",
                            amount,
                            Some(prev),
                            Some(delta_pct(prev, amount)),
                            None,
                            pattern.last_seen_at,
                            sync_run_id,
                        )
                        .await?;
                    self.repo
                        .insert_alert(
                            Some(pattern.id),
                            "price_change",
                            &format!("Price increase: {}", pattern.display_name),
                            Some(&format!("Changed from €{:.2} to €{:.2}", prev.abs(), amount.abs())),
                            sync_run_id,
                        )
                        .await?;
                }
                PriceChangeKind::Decrease => {
                    self.repo
                        .append_price_event(
                            pattern.id,
                            "price_decrease",
                            amount,
                            Some(prev),
                            Some(delta_pct(prev, amount)),
                            None,
                            pattern.last_seen_at,
                            sync_run_id,
                        )
                        .await?;
                    self.repo
                        .insert_alert(
                            Some(pattern.id),
                            "price_change",
                            &format!("Price decrease: {}", pattern.display_name),
                            Some(&format!("Changed from €{:.2} to €{:.2}", prev.abs(), amount.abs())),
                            sync_run_id,
                        )
                        .await?;
                }
                _ => {}
            }
        } else {
            self.repo
                .append_price_event(
                    pattern.id,
                    "billing",
                    amount,
                    None,
                    None,
                    Some(pattern.interval_days),
                    pattern.last_seen_at,
                    sync_run_id,
                )
                .await?;
        }

        Ok(())
    }

    pub async fn mark_stale_inactive(
        &self,
        active_fps: &HashSet<String>,
    ) -> Result<(), sqlx::Error> {
        let confirmed = self.repo.list_confirmed_patterns().await?;
        let today = Utc::now().date_naive();
        for pattern in confirmed {
            if !active_fps.contains(&pattern.fingerprint) {
                let gap = (today - pattern.last_seen_at).num_days();
                if gap > pattern.interval_days as i64 * 2 {
                    self.repo.mark_inactive(pattern.id).await?;
                }
            }
        }
        Ok(())
    }
}

pub fn build_active_fingerprint_map(
    groups: &[RecurrenceGroup],
) -> HashMap<String, RecurrenceGroup> {
    groups
        .iter()
        .map(|g| {
            let fp = compute_fingerprint(&g.payee_key, g.interval_days, g.median_amount);
            (fp, g.clone())
        })
        .collect()
}
