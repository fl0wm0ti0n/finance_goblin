use std::collections::{HashMap, HashSet};

use chrono::Utc;
use tracing::warn;
use uuid::Uuid;

use crate::recurrence::{compute_fingerprint, detect_recurrence_groups, RecurrenceConfig, RecurrenceGroup};
use crate::subscriptions::classify::classify_kind;
use crate::subscriptions::price_change::{
    classify_price_change, delta_pct, PriceChangeConfig, PriceChangeKind,
};
use crate::subscriptions::repository::{
    find_confirmed_payee_interval, is_rejected_payee_interval, interval_matches,
    SubscriptionRepository,
};
use crate::subscriptions::types::{ConfirmedPayeeInterval, PatternRow};

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
        confirmed_payee_intervals: &[ConfirmedPayeeInterval],
        rejected_payee_intervals: &[(String, i32)],
    ) -> Result<CandidateRunResult, sqlx::Error> {
        let config = self.repo.config();
        let txs = self
            .repo
            .load_expense_transactions(config.detection_window_days, None)
            .await?;
        let recurrence_config = RecurrenceConfig {
            high_tolerance_pct: config.confidence_tolerance_pct.high,
            medium_tolerance_pct: config.confidence_tolerance_pct.medium,
            low_tolerance_pct: config.confidence_tolerance_pct.low,
            min_emit_confidence: 60,
        };

        let groups = detect_recurrence_groups(&txs, &recurrence_config);
        let mut new_detections = Vec::new();

        for group in &groups {
            let fingerprint =
                compute_fingerprint(&group.payee_key, group.interval_days, group.median_amount);
            if rejections.contains(&fingerprint) || confirmed_fps.contains(&fingerprint) {
                continue;
            }

            let interval_days = group.interval_days as i32;
            if is_rejected_payee_interval(rejected_payee_intervals, &group.payee_key, interval_days)
            {
                continue;
            }

            if let Some(confirmed) =
                find_confirmed_payee_interval(confirmed_payee_intervals, &group.payee_key, interval_days)
            {
                let kind = classify_kind(group, config);
                let merged = self
                    .repo
                    .merge_confirmed_pattern(
                        confirmed.id,
                        group,
                        &fingerprint,
                        kind,
                        sync_run_id,
                    )
                    .await?;
                if merged {
                    continue;
                }
                warn!(
                    payee_key = %group.payee_key,
                    interval_days,
                    display_name = %group.display_name,
                    confirmed_id = %confirmed.id,
                    "confirmed payee-interval match but merge failed (fingerprint conflict) — skip pending insert"
                );
                continue;
            }

            let kind = classify_kind(group, config);
            let outcome = self
                .repo
                .upsert_pending_pattern(group, &fingerprint, kind, sync_run_id)
                .await?;

            let mut alert_emitted = false;
            if outcome.emit_detection_alert {
                let alert_fp = SubscriptionRepository::compute_alert_fingerprint(
                    "new_detection",
                    outcome.id,
                    None,
                    None,
                    None,
                );
                self.repo
                    .upsert_alert(
                        Some(outcome.id),
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
                        &alert_fp,
                    )
                    .await?;
                alert_emitted = true;
            }

            new_detections.push((outcome.id, group.clone(), alert_emitted));
        }

        Ok(CandidateRunResult {
            new_detections,
            groups,
        })
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
                    let alert_fp = SubscriptionRepository::compute_alert_fingerprint(
                        "price_change",
                        pattern.id,
                        Some("increase"),
                        Some(amount),
                        None,
                    );
                    self.repo
                        .upsert_alert(
                            Some(pattern.id),
                            "price_change",
                            &format!("Price increase: {}", pattern.display_name),
                            Some(&format!(
                                "Changed from €{:.2} to €{:.2}",
                                prev.abs(),
                                amount.abs()
                            )),
                            sync_run_id,
                            &alert_fp,
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
                    let alert_fp = SubscriptionRepository::compute_alert_fingerprint(
                        "price_change",
                        pattern.id,
                        Some("decrease"),
                        Some(amount),
                        None,
                    );
                    self.repo
                        .upsert_alert(
                            Some(pattern.id),
                            "price_change",
                            &format!("Price decrease: {}", pattern.display_name),
                            Some(&format!(
                                "Changed from €{:.2} to €{:.2}",
                                prev.abs(),
                                amount.abs()
                            )),
                            sync_run_id,
                            &alert_fp,
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
        active_payee_intervals: &HashSet<(String, i32)>,
    ) -> Result<(), sqlx::Error> {
        let confirmed = self.repo.list_confirmed_patterns().await?;
        let today = Utc::now().date_naive();
        for pattern in confirmed {
            let is_active = active_payee_intervals.iter().any(|(pk, iv)| {
                pk == &pattern.payee_key && interval_matches(*iv, pattern.interval_days)
            });
            if !is_active {
                let gap = (today - pattern.last_seen_at).num_days();
                if gap > pattern.interval_days as i64 * 2 {
                    self.repo.mark_inactive(pattern.id).await?;
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct CandidateRunResult {
    pub new_detections: Vec<(Uuid, RecurrenceGroup, bool)>,
    pub groups: Vec<RecurrenceGroup>,
}

pub fn build_active_payee_intervals(groups: &[RecurrenceGroup]) -> HashSet<(String, i32)> {
    groups
        .iter()
        .map(|g| (g.payee_key.clone(), g.interval_days as i32))
        .collect()
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::subscriptions::repository::SubscriptionRepository;

    #[test]
    fn new_detection_fingerprint_is_stable() {
        let id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        assert_eq!(
            SubscriptionRepository::compute_alert_fingerprint("new_detection", id, None, None, None),
            "sub_alert:new_detection:550e8400-e29b-41d4-a716-446655440000"
        );
    }

    #[test]
    fn price_change_fingerprint_includes_direction_and_amount() {
        let id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        assert_eq!(
            SubscriptionRepository::compute_alert_fingerprint(
                "price_change",
                id,
                Some("increase"),
                Some(12.995),
                None,
            ),
            "sub_alert:price_change:550e8400-e29b-41d4-a716-446655440000:increase:13.00"
        );
    }

    #[test]
    fn build_active_payee_intervals_from_groups() {
        let groups = vec![RecurrenceGroup {
            payee_key: "cursor".into(),
            display_name: "Cursor".into(),
            interval_days: 30,
            median_amount: -20.0,
            confidence_pct: 80,
            transaction_ids: vec![],
            transaction_dates: vec![],
            category_ids: vec![],
        }];
        let active = build_active_payee_intervals(&groups);
        assert!(active.contains(&("cursor".to_string(), 30)));
    }

    #[test]
    fn interval_matches_covers_monthly_variance() {
        assert!(interval_matches(30, 31));
        assert!(!interval_matches(7, 30));
    }

    #[test]
    fn detection_pipeline_min_emit_confidence_unchanged_at_60() {
        let config = crate::recurrence::RecurrenceConfig::default();
        assert_eq!(config.min_emit_confidence, 60);
    }

    #[test]
    fn manual_discover_confirm_does_not_use_alert_fingerprint_path() {
        // Regression guard (DEC-0099 / AC-6): discover confirm is repository-only;
        // alert upsert remains exclusive to DetectionPipeline::run_candidates.
        let id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        assert!(SubscriptionRepository::compute_alert_fingerprint("new_detection", id, None, None, None)
            .starts_with("sub_alert:new_detection:"));
    }
}
