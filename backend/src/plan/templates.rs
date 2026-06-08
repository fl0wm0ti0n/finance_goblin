use uuid::Uuid;

use crate::config::PlansConfig;

use super::types::{
    AdjustmentDirection, AdjustmentFrequency, AdjustmentTarget, PlanAdjustment, PlanTemplate,
    SavingsSuggestion,
};

#[derive(Debug, Clone, Default)]
pub struct TemplateOverrides {
    pub subscription_payee_keys: Vec<String>,
    pub discretionary_cut: bool,
}

pub fn template_defaults(
    template: PlanTemplate,
    config: &PlansConfig,
    overrides: &TemplateOverrides,
) -> Vec<PlanAdjustment> {
    let version_id = Uuid::nil();
    let today = chrono::Utc::now().date_naive();

    match template {
        PlanTemplate::Current => vec![],
        PlanTemplate::Leasing => vec![PlanAdjustment {
            id: Uuid::new_v4(),
            version_id,
            direction: AdjustmentDirection::AddOutflow,
            amount: config.leasing_default_monthly_eur,
            frequency: AdjustmentFrequency::Monthly,
            target_type: AdjustmentTarget::Household,
            target_key: None,
            label: Some("Leasing".into()),
            effective_from: today,
            effective_to: None,
            sort_order: 0,
        }],
        PlanTemplate::SavingsMode => {
            let mut lines = Vec::new();
            let mut order = 0;
            for payee in &overrides.subscription_payee_keys {
                lines.push(PlanAdjustment {
                    id: Uuid::new_v4(),
                    version_id,
                    direction: AdjustmentDirection::RemoveOutflow,
                    amount: 0.0,
                    frequency: AdjustmentFrequency::Monthly,
                    target_type: AdjustmentTarget::Subscription,
                    target_key: Some(payee.clone()),
                    label: Some(format!("Remove {payee}")),
                    effective_from: today,
                    effective_to: None,
                    sort_order: order,
                });
                order += 1;
            }
            if overrides.discretionary_cut {
                lines.push(PlanAdjustment {
                    id: Uuid::new_v4(),
                    version_id,
                    direction: AdjustmentDirection::AddOutflow,
                    amount: config.savings_mode_discretionary_cut_eur,
                    frequency: AdjustmentFrequency::Monthly,
                    target_type: AdjustmentTarget::Household,
                    target_key: None,
                    label: Some("Discretionary cut".into()),
                    effective_from: today,
                    effective_to: None,
                    sort_order: order,
                });
            }
            lines
        }
        PlanTemplate::HousePurchase => vec![PlanAdjustment {
            id: Uuid::new_v4(),
            version_id,
            direction: AdjustmentDirection::AddOutflow,
            amount: config.house_purchase_default_savings_eur,
            frequency: AdjustmentFrequency::Monthly,
            target_type: AdjustmentTarget::CustomLabel,
            target_key: None,
            label: Some("House savings".into()),
            effective_from: today,
            effective_to: None,
            sort_order: 0,
        }],
        PlanTemplate::Custom => vec![],
        PlanTemplate::GoalBalance => vec![],
        PlanTemplate::AllocationTarget => vec![PlanAdjustment {
            id: Uuid::new_v4(),
            version_id,
            direction: AdjustmentDirection::AddOutflow,
            amount: 0.0,
            frequency: AdjustmentFrequency::OneTime,
            target_type: AdjustmentTarget::AllocationTarget,
            target_key: None,
            label: Some(r#"{"kind":"allocation_target","weights":{"etf_traditional_pct":50,"crypto_pct":50,"cash_pct":0}}"#.into()),
            effective_from: today,
            effective_to: None,
            sort_order: 0,
        }],
    }
}

pub fn map_savings_suggestions(
    rows: &[(Uuid, String, String, f64, i32)],
) -> Vec<SavingsSuggestion> {
    rows.iter()
        .map(|(id, payee, name, amount, interval)| SavingsSuggestion {
            pattern_id: id.to_string(),
            payee_key: payee.clone(),
            display_name: name.clone(),
            current_amount: format!("{amount:.2}"),
            interval_days: *interval,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::PlansConfig;

    fn test_config() -> PlansConfig {
        PlansConfig::default()
    }

    #[test]
    fn leasing_template_produces_monthly_outflow() {
        let lines = template_defaults(PlanTemplate::Leasing, &test_config(), &Default::default());
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].amount, 300.0);
        assert_eq!(lines[0].frequency, AdjustmentFrequency::Monthly);
    }

    #[test]
    fn savings_mode_includes_selected_subscriptions() {
        let overrides = TemplateOverrides {
            subscription_payee_keys: vec!["spotify".into()],
            discretionary_cut: true,
        };
        let lines = template_defaults(PlanTemplate::SavingsMode, &test_config(), &overrides);
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].target_type, AdjustmentTarget::Subscription);
    }

    #[test]
    fn current_template_is_empty() {
        let lines = template_defaults(PlanTemplate::Current, &test_config(), &Default::default());
        assert!(lines.is_empty());
    }
}
