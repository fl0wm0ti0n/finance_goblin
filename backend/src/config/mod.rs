use std::collections::HashMap;
use std::path::PathBuf;

use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub firefly: FireflyConfig,
    pub sync: SyncConfig,
    pub oidc: OidcConfig,
    #[serde(default = "default_forecast_config")]
    pub forecast: ForecastConfig,
    #[serde(default = "default_subscriptions_config")]
    pub subscriptions: SubscriptionsConfig,
    #[serde(default = "default_plans_config")]
    pub plans: PlansConfig,
    #[serde(default = "default_alerts_config")]
    pub alerts: AlertsConfig,
    #[serde(default = "default_wealth_config")]
    pub wealth: WealthConfig,
    #[serde(default = "default_ai_config")]
    pub ai: AiConfig,
    #[serde(default = "default_privacy_config")]
    pub privacy: PrivacyConfig,
    #[serde(default = "default_exchanges_config")]
    pub exchanges: ExchangesConfig,
    #[serde(default = "default_portfolio_config")]
    pub portfolio: PortfolioConfig,
    #[serde(default = "default_forecast_ml_config")]
    pub forecast_ml: ForecastMlConfig,
    #[serde(default = "default_analytics_config")]
    pub analytics: AnalyticsConfig,
    /// Optional maintenance URL (`DATABASE_BOOTSTRAP_URL`) — never log verbatim.
    #[serde(default)]
    pub database_bootstrap_url: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AnalyticsConfig {
    #[serde(default = "default_grafana_upstream")]
    pub grafana_upstream: String,
}

fn default_analytics_config() -> AnalyticsConfig {
    AnalyticsConfig {
        grafana_upstream: default_grafana_upstream(),
    }
}

fn default_grafana_upstream() -> String {
    "http://grafana:3000".into()
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    pub mode: String,
    pub host: String,
    pub port: u16,
    pub name: String,
    pub user: String,
    #[serde(default)]
    pub password: String,
    pub max_connections: u32,
    pub startup_retry_initial_ms: u64,
    pub startup_retry_max_ms: u64,
    pub startup_retry_total_ms: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FireflyConfig {
    pub base_url: String,
    #[serde(default)]
    pub personal_access_token: String,
    pub page_limit: u32,
    pub audit_enabled: bool,
}

impl FireflyConfig {
    /// True when a non-empty PAT is loaded (env or config file).
    pub fn pat_configured(&self) -> bool {
        !self.personal_access_token.trim().is_empty()
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct SyncConfig {
    pub interval_seconds: u64,
    pub overlap_days: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OidcConfig {
    #[serde(default)]
    pub issuer_url: String,
    #[serde(default)]
    pub audience: String,
    #[serde(default)]
    pub dev_bypass: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ForecastConfig {
    #[serde(default = "default_rolling_window_days")]
    pub rolling_window_days: i64,
    #[serde(default = "default_sparse_history_days")]
    pub sparse_history_days: i64,
    #[serde(default = "default_retention_count")]
    pub retention_count: u32,
    #[serde(default = "default_recurring_tolerance")]
    pub recurring_amount_tolerance_pct: f64,
    #[serde(default = "default_category_buckets")]
    pub category_buckets: HashMap<String, String>,
    #[serde(default = "default_ai_bucket_min_confidence")]
    pub ai_bucket_min_confidence: f64,
}

fn default_forecast_config() -> ForecastConfig {
    ForecastConfig {
        rolling_window_days: default_rolling_window_days(),
        sparse_history_days: default_sparse_history_days(),
        retention_count: default_retention_count(),
        recurring_amount_tolerance_pct: default_recurring_tolerance(),
        category_buckets: default_category_buckets(),
        ai_bucket_min_confidence: default_ai_bucket_min_confidence(),
    }
}

fn default_ai_bucket_min_confidence() -> f64 {
    0.75
}

fn default_rolling_window_days() -> i64 {
    90
}

fn default_sparse_history_days() -> i64 {
    90
}

fn default_retention_count() -> u32 {
    5
}

fn default_recurring_tolerance() -> f64 {
    5.0
}

#[derive(Debug, Clone, Deserialize)]
pub struct SubscriptionsConfig {
    #[serde(default = "default_detection_window_days")]
    pub detection_window_days: i64,
    #[serde(default = "default_full_rescan_interval_days")]
    pub full_rescan_interval_days: i64,
    #[serde(default = "default_price_change_min_eur")]
    pub price_change_min_eur: f64,
    #[serde(default = "default_price_change_min_pct")]
    pub price_change_min_pct: f64,
    #[serde(default = "default_inactive_grace_days")]
    pub inactive_grace_days: i64,
    #[serde(default = "default_standing_order_categories")]
    pub standing_order_category_patterns: Vec<String>,
    #[serde(default)]
    pub standing_order_payee_patterns: Vec<String>,
    #[serde(default = "default_confidence_tolerance")]
    pub confidence_tolerance_pct: ConfidenceTolerance,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ConfidenceTolerance {
    #[serde(default = "default_high_tolerance")]
    pub high: f64,
    #[serde(default = "default_medium_tolerance")]
    pub medium: f64,
    #[serde(default = "default_low_tolerance")]
    pub low: f64,
}

impl Default for SubscriptionsConfig {
    fn default() -> Self {
        default_subscriptions_config()
    }
}

fn default_subscriptions_config() -> SubscriptionsConfig {
    SubscriptionsConfig {
        detection_window_days: default_detection_window_days(),
        full_rescan_interval_days: default_full_rescan_interval_days(),
        price_change_min_eur: default_price_change_min_eur(),
        price_change_min_pct: default_price_change_min_pct(),
        inactive_grace_days: default_inactive_grace_days(),
        standing_order_category_patterns: default_standing_order_categories(),
        standing_order_payee_patterns: vec![],
        confidence_tolerance_pct: default_confidence_tolerance(),
    }
}

fn default_detection_window_days() -> i64 {
    730
}

fn default_full_rescan_interval_days() -> i64 {
    7
}

fn default_price_change_min_eur() -> f64 {
    1.0
}

fn default_price_change_min_pct() -> f64 {
    5.0
}

fn default_inactive_grace_days() -> i64 {
    5
}

fn default_high_tolerance() -> f64 {
    5.0
}

fn default_medium_tolerance() -> f64 {
    10.0
}

fn default_low_tolerance() -> f64 {
    15.0
}

fn default_confidence_tolerance() -> ConfidenceTolerance {
    ConfidenceTolerance {
        high: default_high_tolerance(),
        medium: default_medium_tolerance(),
        low: default_low_tolerance(),
    }
}

fn default_standing_order_categories() -> Vec<String> {
    vec![
        "rent".into(),
        "miete".into(),
        "insurance".into(),
        "versicherung".into(),
        "utilities".into(),
        "nebenkosten".into(),
        "loan".into(),
        "darlehen".into(),
    ]
}

#[derive(Debug, Clone, Deserialize)]
pub struct PlansConfig {
    #[serde(default = "default_leasing_monthly")]
    pub leasing_default_monthly_eur: f64,
    #[serde(default = "default_house_savings")]
    pub house_purchase_default_savings_eur: f64,
    #[serde(default = "default_discretionary_cut")]
    pub savings_mode_discretionary_cut_eur: f64,
    #[serde(default = "default_max_versions")]
    pub max_versions_per_plan: u32,
    #[serde(default = "default_plan_retention")]
    pub computation_retention_per_version: u32,
    #[serde(default = "default_reporting_currency")]
    pub reporting_currency: String,
}

impl Default for PlansConfig {
    fn default() -> Self {
        default_plans_config()
    }
}

fn default_plans_config() -> PlansConfig {
    PlansConfig {
        leasing_default_monthly_eur: default_leasing_monthly(),
        house_purchase_default_savings_eur: default_house_savings(),
        savings_mode_discretionary_cut_eur: default_discretionary_cut(),
        max_versions_per_plan: default_max_versions(),
        computation_retention_per_version: default_plan_retention(),
        reporting_currency: default_reporting_currency(),
    }
}

fn default_leasing_monthly() -> f64 {
    300.0
}

fn default_house_savings() -> f64 {
    500.0
}

fn default_discretionary_cut() -> f64 {
    100.0
}

fn default_max_versions() -> u32 {
    3
}

fn default_plan_retention() -> u32 {
    3
}

fn default_reporting_currency() -> String {
    "EUR".into()
}

#[derive(Debug, Clone, Deserialize)]
pub struct AlertsConfig {
    #[serde(default = "default_scarcity_threshold")]
    pub scarcity_threshold_eur: f64,
    #[serde(default = "default_budget_drift_pct")]
    pub budget_drift_pct: f64,
    #[serde(default = "default_reporting_currency")]
    pub reporting_currency: String,
}

impl Default for AlertsConfig {
    fn default() -> Self {
        default_alerts_config()
    }
}

fn default_alerts_config() -> AlertsConfig {
    AlertsConfig {
        scarcity_threshold_eur: default_scarcity_threshold(),
        budget_drift_pct: default_budget_drift_pct(),
        reporting_currency: default_reporting_currency(),
    }
}

fn default_scarcity_threshold() -> f64 {
    200.0
}

fn default_budget_drift_pct() -> f64 {
    20.0
}

#[derive(Debug, Clone, Deserialize)]
pub struct WealthConfig {
    #[serde(default = "default_snapshot_retention")]
    pub snapshot_retention_days: u32,
}

impl Default for WealthConfig {
    fn default() -> Self {
        default_wealth_config()
    }
}

fn default_wealth_config() -> WealthConfig {
    WealthConfig {
        snapshot_retention_days: default_snapshot_retention(),
    }
}

fn default_snapshot_retention() -> u32 {
    365
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ExchangesConfig {
    #[serde(default = "default_exchanges_enabled")]
    pub enabled: bool,
    #[serde(default = "default_exchange_interval")]
    pub interval_seconds: u64,
    #[serde(default = "default_binance_config")]
    pub binance: ExchangeInstanceConfig,
    #[serde(default = "default_bybit_config")]
    pub bybit: ExchangeInstanceConfig,
    #[serde(default = "default_bitunix_config")]
    pub bitunix: BitunixConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ExchangeInstanceConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub api_key_env: String,
    #[serde(default)]
    pub api_secret_env: String,
    #[serde(default = "default_binance_url")]
    pub base_url: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BitunixConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub api_key_env: String,
    #[serde(default)]
    pub api_secret_env: String,
    #[serde(default = "default_bitunix_url")]
    pub spot_base_url: String,
    #[serde(default = "default_bitunix_futures_url")]
    pub futures_base_url: String,
    #[serde(default)]
    pub enabled_futures: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PortfolioConfig {
    #[serde(default = "default_trade_retention_days")]
    pub trade_retention_days: u32,
    #[serde(default = "default_frankfurter_url")]
    pub frankfurter_base_url: String,
}

impl Default for ExchangesConfig {
    fn default() -> Self {
        default_exchanges_config()
    }
}

impl Default for PortfolioConfig {
    fn default() -> Self {
        default_portfolio_config()
    }
}

fn default_exchanges_config() -> ExchangesConfig {
    ExchangesConfig {
        enabled: default_exchanges_enabled(),
        interval_seconds: default_exchange_interval(),
        binance: default_binance_config(),
        bybit: default_bybit_config(),
        bitunix: default_bitunix_config(),
    }
}

fn default_portfolio_config() -> PortfolioConfig {
    PortfolioConfig {
        trade_retention_days: default_trade_retention_days(),
        frankfurter_base_url: default_frankfurter_url(),
    }
}

fn default_exchanges_enabled() -> bool {
    true
}

fn default_exchange_interval() -> u64 {
    3600
}

fn default_binance_config() -> ExchangeInstanceConfig {
    ExchangeInstanceConfig {
        enabled: false,
        api_key_env: "BINANCE_API_KEY".into(),
        api_secret_env: "BINANCE_API_SECRET".into(),
        base_url: default_binance_url(),
    }
}

fn default_bybit_config() -> ExchangeInstanceConfig {
    ExchangeInstanceConfig {
        enabled: false,
        api_key_env: "BYBIT_API_KEY".into(),
        api_secret_env: "BYBIT_API_SECRET".into(),
        base_url: "https://api.bybit.com".into(),
    }
}

fn default_bitunix_config() -> BitunixConfig {
    BitunixConfig {
        enabled: false,
        api_key_env: "BITUNIX_API_KEY".into(),
        api_secret_env: "BITUNIX_API_SECRET".into(),
        spot_base_url: default_bitunix_url(),
        futures_base_url: default_bitunix_futures_url(),
        enabled_futures: false,
    }
}

fn default_binance_url() -> String {
    "https://api.binance.com".into()
}

fn default_bitunix_url() -> String {
    "https://openapi.bitunix.com".into()
}

fn default_bitunix_futures_url() -> String {
    "https://fapi.bitunix.com".into()
}

fn default_trade_retention_days() -> u32 {
    730
}

fn default_frankfurter_url() -> String {
    "https://api.frankfurter.dev".into()
}

impl ExchangeInstanceConfig {
    pub fn credentials(&self) -> Option<(String, String)> {
        let key = std::env::var(&self.api_key_env).ok().filter(|k| !k.is_empty())?;
        let secret = std::env::var(&self.api_secret_env)
            .ok()
            .filter(|s| !s.is_empty())?;
        Some((key, secret))
    }

    pub fn configured(&self) -> bool {
        self.credentials().is_some()
    }

    pub fn effective_enabled(&self) -> bool {
        self.configured() || self.enabled
    }
}

impl BitunixConfig {
    pub fn credentials(&self) -> Option<(String, String)> {
        let key = std::env::var(&self.api_key_env).ok().filter(|k| !k.is_empty())?;
        let secret = std::env::var(&self.api_secret_env)
            .ok()
            .filter(|s| !s.is_empty())?;
        Some((key, secret))
    }

    pub fn configured(&self) -> bool {
        self.credentials().is_some()
    }

    pub fn effective_enabled(&self) -> bool {
        self.configured() || self.enabled
    }

    pub fn effective_enabled_futures(&self) -> bool {
        if let Ok(v) = std::env::var("BITUNIX_ENABLED_FUTURES") {
            let lower = v.to_ascii_lowercase();
            if matches!(lower.as_str(), "false" | "0" | "no") {
                return false;
            }
            if matches!(lower.as_str(), "true" | "1" | "yes") {
                return true;
            }
        }
        if self.enabled_futures {
            return true;
        }
        self.effective_enabled() && self.credentials().is_some()
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ExchangeSettingsView {
    pub enabled: bool,
    pub api_key_env: String,
    pub api_secret_env: String,
    pub configured: bool,
    pub base_url: Option<String>,
    pub spot_base_url: Option<String>,
    pub futures_base_url: Option<String>,
    pub enabled_futures: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ExchangesSettingsView {
    pub enabled: bool,
    pub interval_seconds: u64,
    pub binance: ExchangeSettingsView,
    pub bybit: ExchangeSettingsView,
    pub bitunix: ExchangeSettingsView,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PortfolioSettingsView {
    pub trade_retention_days: u32,
    pub frankfurter_base_url: String,
}

impl ExchangesConfig {
    pub fn settings_view(&self) -> ExchangesSettingsView {
        ExchangesSettingsView {
            enabled: self.enabled,
            interval_seconds: self.interval_seconds,
            binance: ExchangeSettingsView {
                enabled: self.binance.effective_enabled(),
                api_key_env: self.binance.api_key_env.clone(),
                api_secret_env: self.binance.api_secret_env.clone(),
                configured: self.binance.configured(),
                base_url: Some(self.binance.base_url.clone()),
                spot_base_url: None,
                futures_base_url: None,
                enabled_futures: None,
            },
            bybit: ExchangeSettingsView {
                enabled: self.bybit.effective_enabled(),
                api_key_env: self.bybit.api_key_env.clone(),
                api_secret_env: self.bybit.api_secret_env.clone(),
                configured: self.bybit.configured(),
                base_url: Some(self.bybit.base_url.clone()),
                spot_base_url: None,
                futures_base_url: None,
                enabled_futures: None,
            },
            bitunix: ExchangeSettingsView {
                enabled: self.bitunix.effective_enabled(),
                api_key_env: self.bitunix.api_key_env.clone(),
                api_secret_env: self.bitunix.api_secret_env.clone(),
                configured: self.bitunix.configured(),
                base_url: None,
                spot_base_url: Some(self.bitunix.spot_base_url.clone()),
                futures_base_url: Some(self.bitunix.futures_base_url.clone()),
                enabled_futures: Some(self.bitunix.effective_enabled_futures()),
            },
        }
    }
}

impl PortfolioConfig {
    pub fn settings_view(&self) -> PortfolioSettingsView {
        PortfolioSettingsView {
            trade_retention_days: self.trade_retention_days,
            frankfurter_base_url: self.frankfurter_base_url.clone(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AiConfig {
    #[serde(default = "default_ai_provider")]
    pub provider: String,
    #[serde(default)]
    pub base_url: String,
    #[serde(default = "default_ai_model")]
    pub model: String,
    #[serde(default = "default_api_key_env")]
    pub api_key_env: String,
    #[serde(default)]
    pub temperature: Option<f32>,
    #[serde(default = "default_local_tool_nudge_retry")]
    pub local_tool_nudge_retry: bool,
    #[serde(default = "default_max_tool_rounds")]
    pub max_tool_rounds: u32,
    #[serde(default = "default_max_completion_tokens")]
    pub max_completion_tokens: u32,
    #[serde(default = "default_max_tool_result_bytes")]
    pub max_tool_result_bytes: usize,
    #[serde(default = "default_request_timeout_secs")]
    pub request_timeout_secs: u64,
    #[serde(default = "default_rate_limit_per_min")]
    pub rate_limit_per_min: u32,
    #[serde(default = "default_audit_retention_days")]
    pub audit_retention_days: u32,
    #[serde(default = "default_audit_max_rows")]
    pub audit_max_rows: u32,
}

/// Settings API view — no secrets.
#[derive(Debug, Clone, Serialize)]
pub struct AiPublicSettings {
    pub provider: String,
    pub provider_label: String,
    pub base_url: String,
    pub model: String,
    pub is_local: bool,
    pub provider_configured: bool,
}

impl Default for AiConfig {
    fn default() -> Self {
        default_ai_config()
    }
}

fn default_ai_config() -> AiConfig {
    AiConfig {
        provider: default_ai_provider(),
        base_url: String::new(),
        model: default_ai_model(),
        api_key_env: default_api_key_env(),
        temperature: None,
        local_tool_nudge_retry: default_local_tool_nudge_retry(),
        max_tool_rounds: default_max_tool_rounds(),
        max_completion_tokens: default_max_completion_tokens(),
        max_tool_result_bytes: default_max_tool_result_bytes(),
        request_timeout_secs: default_request_timeout_secs(),
        rate_limit_per_min: default_rate_limit_per_min(),
        audit_retention_days: default_audit_retention_days(),
        audit_max_rows: default_audit_max_rows(),
    }
}

fn default_local_tool_nudge_retry() -> bool {
    true
}

fn default_ai_provider() -> String {
    "openai".into()
}

fn default_ai_model() -> String {
    "gpt-4o-mini".into()
}

fn default_api_key_env() -> String {
    "OPENAI_API_KEY".into()
}

fn default_max_tool_rounds() -> u32 {
    5
}

fn default_max_completion_tokens() -> u32 {
    1024
}

fn default_max_tool_result_bytes() -> usize {
    8192
}

fn default_request_timeout_secs() -> u64 {
    60
}

fn default_rate_limit_per_min() -> u32 {
    20
}

fn default_audit_retention_days() -> u32 {
    90
}

fn default_audit_max_rows() -> u32 {
    500
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PrivacyConfig {
    #[serde(default)]
    pub allow_raw_transactions: bool,
    #[serde(default = "default_true")]
    pub redact_iban: bool,
    #[serde(default = "default_true")]
    pub redact_counterparties: bool,
}

impl Default for PrivacyConfig {
    fn default() -> Self {
        default_privacy_config()
    }
}

fn default_true() -> bool {
    true
}

fn default_privacy_config() -> PrivacyConfig {
    PrivacyConfig {
        allow_raw_transactions: false,
        redact_iban: true,
        redact_counterparties: true,
    }
}

impl AiConfig {
    pub fn api_key(&self) -> Option<String> {
        std::env::var(&self.api_key_env).ok().filter(|k| !k.is_empty())
    }

    pub fn openai_api_key(&self) -> Option<String> {
        self.api_key()
    }

    pub fn effective_base_url(&self) -> Option<String> {
        if self.base_url.trim().is_empty() {
            None
        } else {
            Some(self.base_url.trim_end_matches('/').to_string())
        }
    }

    pub fn provider_configured(&self) -> bool {
        match self.provider.trim().to_lowercase().as_str() {
            "openai" => self.api_key().is_some(),
            "ollama" => true,
            "openai_compatible" => self.effective_base_url().is_some(),
            _ => false,
        }
    }

    pub fn openai_configured(&self) -> bool {
        self.provider_configured()
    }

    pub fn public_settings(&self, label: &str, base_url: &str, is_local: bool) -> AiPublicSettings {
        AiPublicSettings {
            provider: self.provider.clone(),
            provider_label: label.to_string(),
            base_url: base_url.to_string(),
            model: self.model.clone(),
            is_local,
            provider_configured: self.provider_configured(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ForecastMlConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_sidecar_url")]
    pub sidecar_url: String,
    #[serde(default = "default_min_monthly_points")]
    pub min_monthly_points: u32,
    #[serde(default = "default_min_portfolio_weeks")]
    pub min_portfolio_weeks: u32,
    #[serde(default = "default_interval_level")]
    pub interval_level: u32,
    #[serde(default = "default_sidecar_timeout_secs")]
    pub sidecar_timeout_secs: u64,
    #[serde(default = "default_mstl_min_months")]
    pub mstl_min_months: u32,
    #[serde(default = "default_mstl_seasonal_strength_threshold")]
    pub mstl_seasonal_strength_threshold: f64,
    #[serde(default = "default_wmape_low_confidence_threshold")]
    pub wmape_low_confidence_threshold: f64,
}

impl Default for ForecastMlConfig {
    fn default() -> Self {
        default_forecast_ml_config()
    }
}

fn default_forecast_ml_config() -> ForecastMlConfig {
    ForecastMlConfig {
        enabled: false,
        sidecar_url: default_sidecar_url(),
        min_monthly_points: default_min_monthly_points(),
        min_portfolio_weeks: default_min_portfolio_weeks(),
        interval_level: default_interval_level(),
        sidecar_timeout_secs: default_sidecar_timeout_secs(),
        mstl_min_months: default_mstl_min_months(),
        mstl_seasonal_strength_threshold: default_mstl_seasonal_strength_threshold(),
        wmape_low_confidence_threshold: default_wmape_low_confidence_threshold(),
    }
}

fn default_sidecar_url() -> String {
    "http://stats-forecast:8090".into()
}

fn default_min_monthly_points() -> u32 {
    12
}

fn default_min_portfolio_weeks() -> u32 {
    8
}

fn default_interval_level() -> u32 {
    90
}

fn default_sidecar_timeout_secs() -> u64 {
    60
}

fn default_mstl_min_months() -> u32 {
    24
}

fn default_mstl_seasonal_strength_threshold() -> f64 {
    0.35
}

fn default_wmape_low_confidence_threshold() -> f64 {
    0.35
}

fn default_category_buckets() -> HashMap<String, String> {
    let mut m = HashMap::new();
    m.insert("salary".into(), "income".into());
    m.insert("payroll".into(), "income".into());
    m.insert("rent".into(), "fixed".into());
    m.insert("mortgage".into(), "fixed".into());
    m.insert("insurance".into(), "fixed".into());
    m.insert("utilities".into(), "fixed".into());
    m
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("database.mode must be 'external', got '{0}'")]
    InvalidDatabaseMode(String),
    #[error("missing required environment variable: {0}")]
    MissingEnv(String),
    #[error("config load failed: {0}")]
    Load(#[from] config::ConfigError),
    #[error("invalid GRAFANA_UPSTREAM: {0}")]
    InvalidGrafanaUpstream(String),
    #[error("invalid DATABASE_NAME '{0}': must match ^[a-zA-Z_][a-zA-Z0-9_]*$")]
    InvalidDatabaseName(String),
}

impl AppConfig {
    pub fn load() -> Result<Self, ConfigError> {
        let config_path = std::env::var("APP_CONFIG")
            .unwrap_or_else(|_| "config/default.toml".to_string());

        let mut builder = config::Config::builder()
            .add_source(config::File::from(PathBuf::from(&config_path)).required(false))
            .add_source(config::Environment::default().separator("__"));

        // Overlay standard env vars
        if let Ok(host) = std::env::var("DATABASE_HOST") {
            builder = builder.set_override("database.host", host)?;
        }
        if let Ok(port) = std::env::var("DATABASE_PORT") {
            builder = builder.set_override(
                "database.port",
                port.parse::<i64>()
                    .map_err(|e| ConfigError::MissingEnv(format!("DATABASE_PORT invalid: {e}")))?,
            )?;
        }
        if let Ok(name) = std::env::var("DATABASE_NAME") {
            builder = builder.set_override("database.name", name)?;
        }
        if let Ok(user) = std::env::var("DATABASE_USER") {
            builder = builder.set_override("database.user", user)?;
        }
        if let Ok(password) = std::env::var("DATABASE_PASSWORD") {
            builder = builder.set_override("database.password", password)?;
        }
        if let Ok(url) = std::env::var("FIREFLY_BASE_URL") {
            builder = builder.set_override("firefly.base_url", url)?;
        }
        if let Ok(pat) = std::env::var("FIREFLY_PERSONAL_ACCESS_TOKEN") {
            if !pat.trim().is_empty() {
                builder = builder.set_override("firefly.personal_access_token", pat)?;
            }
        }
        if let Ok(issuer) = std::env::var("OIDC_ISSUER_URL") {
            builder = builder.set_override("oidc.issuer_url", issuer)?;
        }
        if let Ok(aud) = std::env::var("OIDC_AUDIENCE") {
            builder = builder.set_override("oidc.audience", aud)?;
        }
        if let Ok(interval) = std::env::var("SYNC_INTERVAL_SECONDS") {
            builder = builder.set_override(
                "sync.interval_seconds",
                interval
                    .parse::<i64>()
                    .map_err(|e| ConfigError::MissingEnv(format!("SYNC_INTERVAL_SECONDS invalid: {e}")))?,
            )?;
        }
        if let Ok(bypass) = std::env::var("AUTH_DEV_BYPASS") {
            let enabled = bypass.eq_ignore_ascii_case("true") || bypass == "1";
            builder = builder.set_override("oidc.dev_bypass", enabled)?;
        }

        let mut cfg: AppConfig = builder.build()?.try_deserialize()?;

        if let Ok(p) = std::env::var("AI_PROVIDER") {
            cfg.ai.provider = p;
        }
        if let Ok(u) = std::env::var("AI_BASE_URL") {
            cfg.ai.base_url = u;
        }
        if let Ok(m) = std::env::var("AI_MODEL") {
            cfg.ai.model = m;
        }
        if let Ok(v) = std::env::var("FORECAST_ML_ENABLED") {
            cfg.forecast_ml.enabled =
                v.eq_ignore_ascii_case("true") || v == "1";
        }
        if let Ok(url) = std::env::var("STATS_FORECAST_URL") {
            cfg.forecast_ml.sidecar_url = url;
        }
        if let Ok(url) = std::env::var("GRAFANA_UPSTREAM") {
            cfg.analytics.grafana_upstream = url;
        }
        if let Ok(url) = std::env::var("DATABASE_BOOTSTRAP_URL") {
            let trimmed = url.trim();
            if !trimmed.is_empty() {
                cfg.database_bootstrap_url = Some(trimmed.to_string());
            }
        }

        cfg.validate()?;
        Ok(cfg)
    }

    /// Parsed Grafana upstream URL (validated at startup).
    pub fn grafana_upstream_url(&self) -> Result<url::Url, ConfigError> {
        validate_grafana_upstream(&self.analytics.grafana_upstream)
    }

    fn validate(&self) -> Result<(), ConfigError> {
        if self.database.mode != "external" {
            return Err(ConfigError::InvalidDatabaseMode(self.database.mode.clone()));
        }

        if self.database.password.is_empty() {
            return Err(ConfigError::MissingEnv(
                "DATABASE_PASSWORD (external PostgreSQL password required)".into(),
            ));
        }

        validate_database_name(&self.database.name)?;
        validate_grafana_upstream(&self.analytics.grafana_upstream)?;
        Ok(())
    }

    /// Maintenance connection URL: `DATABASE_BOOTSTRAP_URL` or derived `…/postgres`.
    pub fn maintenance_database_url(&self) -> String {
        if let Some(ref url) = self.database_bootstrap_url {
            return url.clone();
        }
        format!(
            "postgres://{}:{}@{}:{}/postgres",
            urlencoding::encode(&self.database.user),
            urlencoding::encode(&self.database.password),
            self.database.host,
            self.database.port
        )
    }

    /// App database URL using maintenance credentials (extension step).
    pub fn app_database_maintenance_url(&self) -> String {
        if let Some(ref bootstrap) = self.database_bootstrap_url {
            return replace_database_in_url(bootstrap, &self.database.name);
        }
        self.database_url()
    }

    pub fn database_url(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            urlencoding::encode(&self.database.user),
            urlencoding::encode(&self.database.password),
            self.database.host,
            self.database.port,
            self.database.name
        )
    }
}

/// PostgreSQL identifier allowlist for `DATABASE_NAME` (DEC-0058).
pub fn validate_database_name(name: &str) -> Result<(), ConfigError> {
    static RE: std::sync::OnceLock<Regex> = std::sync::OnceLock::new();
    let re = RE.get_or_init(|| Regex::new(r"^[a-zA-Z_][a-zA-Z0-9_]*$").expect("database name regex"));
    if name.is_empty() || !re.is_match(name) {
        return Err(ConfigError::InvalidDatabaseName(name.to_string()));
    }
    Ok(())
}

/// Redact password (and userinfo) from a database URL for logs/tests.
pub fn redact_database_url(raw: &str) -> String {
    match url::Url::parse(raw) {
        Ok(parsed) => {
            let host = parsed.host_str().unwrap_or("unknown");
            let port = parsed
                .port()
                .map(|p| format!(":{p}"))
                .unwrap_or_default();
            let path = parsed.path();
            format!("postgres://[REDACTED]@[{host}{port}{path}")
        }
        Err(_) => "[invalid-url]".into(),
    }
}

fn replace_database_in_url(bootstrap: &str, db_name: &str) -> String {
    match url::Url::parse(bootstrap) {
        Ok(mut parsed) => {
            parsed.set_path(&format!("/{db_name}"));
            parsed.to_string()
        }
        Err(_) => bootstrap.to_string(),
    }
}

/// Maintenance URL user (for OWNER vs bootstrap user logging).
pub fn maintenance_url_user(raw: &str) -> Option<String> {
    url::Url::parse(raw)
        .ok()
        .and_then(|u| {
            let user = u.username();
            if user.is_empty() {
                None
            } else {
                Some(user.to_string())
            }
        })
}

// Minimal url encoding without extra dep in validate path — add urlencoding crate
/// Allowlisted hosts: `grafana` (Docker), `localhost` / `127.0.0.1` (dev). SSRF guard (DEC-0057).
pub fn validate_grafana_upstream(raw: &str) -> Result<url::Url, ConfigError> {
    let parsed = url::Url::parse(raw)
        .map_err(|e| ConfigError::InvalidGrafanaUpstream(format!("invalid URL: {e}")))?;
    let scheme = parsed.scheme();
    if scheme != "http" && scheme != "https" {
        return Err(ConfigError::InvalidGrafanaUpstream(format!(
            "scheme must be http or https, got '{scheme}'"
        )));
    }
    let host = parsed.host_str().ok_or_else(|| {
        ConfigError::InvalidGrafanaUpstream("missing host".into())
    })?;
    let allowed = matches!(host, "grafana" | "localhost" | "127.0.0.1");
    if !allowed {
        return Err(ConfigError::InvalidGrafanaUpstream(format!(
            "host '{host}' not in allowlist (grafana, localhost, 127.0.0.1)"
        )));
    }
    if parsed.port().is_none() && (scheme == "http" || scheme == "https") {
        // default ports are fine
    }
    Ok(parsed)
}

mod urlencoding {
    pub fn encode(input: &str) -> String {
        input
            .chars()
            .map(|c| match c {
                'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
                _ => format!("%{:02X}", c as u8),
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_non_external_database_mode() {
        let cfg = AppConfig {
            server: ServerConfig {
                host: "0.0.0.0".into(),
                port: 8080,
            },
            database: DatabaseConfig {
                mode: "embedded".into(),
                host: "localhost".into(),
                port: 5432,
                name: "test".into(),
                user: "u".into(),
                password: "p".into(),
                max_connections: 5,
                startup_retry_initial_ms: 1000,
                startup_retry_max_ms: 8000,
                startup_retry_total_ms: 60000,
            },
            firefly: FireflyConfig {
                base_url: "http://localhost".into(),
                personal_access_token: String::new(),
                page_limit: 500,
                audit_enabled: true,
            },
            sync: SyncConfig {
                interval_seconds: 3600,
                overlap_days: 7,
            },
            oidc: OidcConfig {
                issuer_url: String::new(),
                audience: String::new(),
                dev_bypass: false,
            },
            forecast: default_forecast_config(),
            subscriptions: default_subscriptions_config(),
            plans: default_plans_config(),
            alerts: default_alerts_config(),
            wealth: default_wealth_config(),
            ai: default_ai_config(),
            privacy: default_privacy_config(),
            exchanges: default_exchanges_config(),
            portfolio: default_portfolio_config(),
            forecast_ml: default_forecast_ml_config(),
            analytics: default_analytics_config(),
            database_bootstrap_url: None,
        };
        assert!(matches!(
            cfg.validate(),
            Err(ConfigError::InvalidDatabaseMode(_))
        ));
    }

    #[test]
    fn grafana_upstream_allowlist_accepts_docker_and_dev() {
        assert!(validate_grafana_upstream("http://grafana:3000").is_ok());
        assert!(validate_grafana_upstream("http://localhost:3000").is_ok());
        assert!(validate_grafana_upstream("http://127.0.0.1:3000").is_ok());
    }

    #[test]
    fn grafana_upstream_rejects_external_host() {
        assert!(matches!(
            validate_grafana_upstream("http://evil.example.com:3000"),
            Err(ConfigError::InvalidGrafanaUpstream(_))
        ));
    }

    #[test]
    fn grafana_upstream_rejects_invalid_scheme() {
        assert!(matches!(
            validate_grafana_upstream("ftp://grafana:3000"),
            Err(ConfigError::InvalidGrafanaUpstream(_))
        ));
    }

    #[test]
    fn database_name_allowlist_accepts_default() {
        assert!(validate_database_name("flow_finance_ai").is_ok());
        assert!(validate_database_name("_test_db1").is_ok());
    }

    #[test]
    fn database_name_allowlist_rejects_invalid() {
        assert!(matches!(
            validate_database_name(""),
            Err(ConfigError::InvalidDatabaseName(_))
        ));
        assert!(matches!(
            validate_database_name("flow-finance"),
            Err(ConfigError::InvalidDatabaseName(_))
        ));
        assert!(matches!(
            validate_database_name("123bad"),
            Err(ConfigError::InvalidDatabaseName(_))
        ));
    }

    #[test]
    fn maintenance_url_prefers_bootstrap_url() {
        let cfg = AppConfig {
            database_bootstrap_url: Some("postgres://admin:secret@db:5432/postgres".into()),
            ..minimal_app_config()
        };
        assert_eq!(
            cfg.maintenance_database_url(),
            "postgres://admin:secret@db:5432/postgres"
        );
    }

    #[test]
    fn maintenance_url_derived_from_database_fields() {
        let cfg = minimal_app_config();
        assert_eq!(
            cfg.maintenance_database_url(),
            "postgres://finance:pass%40word@postgres:5432/postgres"
        );
    }

    #[test]
    fn app_database_maintenance_url_replaces_path() {
        let cfg = AppConfig {
            database_bootstrap_url: Some("postgres://admin:secret@db:5432/postgres".into()),
            ..minimal_app_config()
        };
        assert_eq!(
            cfg.app_database_maintenance_url(),
            "postgres://admin:secret@db:5432/flow_finance_ai"
        );
    }

    #[test]
    fn redact_database_url_hides_credentials() {
        let redacted = redact_database_url("postgres://user:pass@host:5432/db");
        assert!(!redacted.contains("pass"));
        assert!(!redacted.contains("user"));
        assert!(redacted.contains("[REDACTED]"));
    }

    #[test]
    fn firefly_pat_configured_false_when_token_empty() {
        let cfg = FireflyConfig {
            base_url: "http://firefly:8080".into(),
            personal_access_token: "   ".into(),
            page_limit: 500,
            audit_enabled: true,
        };
        assert!(!cfg.pat_configured());
    }

    #[test]
    fn firefly_pat_configured_true_when_token_non_empty() {
        let cfg = FireflyConfig {
            base_url: "http://firefly:8080".into(),
            personal_access_token: "pat-value".into(),
            page_limit: 500,
            audit_enabled: true,
        };
        assert!(cfg.pat_configured());
    }

    #[test]
    fn exchange_effective_enabled_when_configured_without_toml_enabled() {
        let prev_key = std::env::var("BINANCE_API_KEY").ok();
        let prev_secret = std::env::var("BINANCE_API_SECRET").ok();
        std::env::set_var("BINANCE_API_KEY", "k");
        std::env::set_var("BINANCE_API_SECRET", "s");
        let ex = ExchangeInstanceConfig {
            enabled: false,
            api_key_env: "BINANCE_API_KEY".into(),
            api_secret_env: "BINANCE_API_SECRET".into(),
            base_url: default_binance_url(),
        };
        assert!(ex.configured());
        assert!(ex.effective_enabled());
        restore_env("BINANCE_API_KEY", prev_key);
        restore_env("BINANCE_API_SECRET", prev_secret);
    }

    #[test]
    fn settings_view_uses_effective_enabled_for_bitunix() {
        let prev_key = std::env::var("BITUNIX_API_KEY").ok();
        let prev_secret = std::env::var("BITUNIX_API_SECRET").ok();
        std::env::set_var("BITUNIX_API_KEY", "k");
        std::env::set_var("BITUNIX_API_SECRET", "s");
        let cfg = ExchangesConfig {
            enabled: true,
            interval_seconds: 3600,
            binance: default_binance_config(),
            bybit: default_bybit_config(),
            bitunix: BitunixConfig {
                enabled: false,
                api_key_env: "BITUNIX_API_KEY".into(),
                api_secret_env: "BITUNIX_API_SECRET".into(),
                spot_base_url: default_bitunix_url(),
                futures_base_url: default_bitunix_futures_url(),
                enabled_futures: false,
            },
        };
        let view = cfg.settings_view();
        assert!(view.bitunix.configured);
        assert!(view.bitunix.enabled);
        assert!(view.bitunix.enabled_futures.unwrap());
        restore_env("BITUNIX_API_KEY", prev_key);
        restore_env("BITUNIX_API_SECRET", prev_secret);
    }

    #[test]
    fn effective_enabled_futures_auto_when_creds_present() {
        let prev_key = std::env::var("BITUNIX_API_KEY").ok();
        let prev_secret = std::env::var("BITUNIX_API_SECRET").ok();
        let prev_futures = std::env::var("BITUNIX_ENABLED_FUTURES").ok();
        std::env::remove_var("BITUNIX_ENABLED_FUTURES");
        std::env::set_var("BITUNIX_API_KEY", "k");
        std::env::set_var("BITUNIX_API_SECRET", "s");
        let cfg = BitunixConfig {
            enabled: false,
            api_key_env: "BITUNIX_API_KEY".into(),
            api_secret_env: "BITUNIX_API_SECRET".into(),
            spot_base_url: default_bitunix_url(),
            futures_base_url: default_bitunix_futures_url(),
            enabled_futures: false,
        };
        assert!(cfg.effective_enabled_futures());
        restore_env("BITUNIX_API_KEY", prev_key);
        restore_env("BITUNIX_API_SECRET", prev_secret);
        restore_env("BITUNIX_ENABLED_FUTURES", prev_futures);
    }

    #[test]
    fn effective_enabled_futures_env_false_opt_out() {
        let prev_key = std::env::var("BITUNIX_API_KEY").ok();
        let prev_secret = std::env::var("BITUNIX_API_SECRET").ok();
        let prev_futures = std::env::var("BITUNIX_ENABLED_FUTURES").ok();
        std::env::set_var("BITUNIX_API_KEY", "k");
        std::env::set_var("BITUNIX_API_SECRET", "s");
        std::env::set_var("BITUNIX_ENABLED_FUTURES", "false");
        let cfg = BitunixConfig {
            enabled: false,
            api_key_env: "BITUNIX_API_KEY".into(),
            api_secret_env: "BITUNIX_API_SECRET".into(),
            spot_base_url: default_bitunix_url(),
            futures_base_url: default_bitunix_futures_url(),
            enabled_futures: false,
        };
        assert!(!cfg.effective_enabled_futures());
        restore_env("BITUNIX_API_KEY", prev_key);
        restore_env("BITUNIX_API_SECRET", prev_secret);
        restore_env("BITUNIX_ENABLED_FUTURES", prev_futures);
    }

    fn restore_env(key: &str, prev: Option<String>) {
        match prev {
            Some(v) => std::env::set_var(key, v),
            None => std::env::remove_var(key),
        }
    }

    fn minimal_app_config() -> AppConfig {
        AppConfig {
            server: ServerConfig {
                host: "0.0.0.0".into(),
                port: 8080,
            },
            database: DatabaseConfig {
                mode: "external".into(),
                host: "postgres".into(),
                port: 5432,
                name: "flow_finance_ai".into(),
                user: "finance".into(),
                password: "pass@word".into(),
                max_connections: 5,
                startup_retry_initial_ms: 1000,
                startup_retry_max_ms: 8000,
                startup_retry_total_ms: 60000,
            },
            firefly: FireflyConfig {
                base_url: "http://localhost".into(),
                personal_access_token: String::new(),
                page_limit: 500,
                audit_enabled: true,
            },
            sync: SyncConfig {
                interval_seconds: 3600,
                overlap_days: 7,
            },
            oidc: OidcConfig {
                issuer_url: String::new(),
                audience: String::new(),
                dev_bypass: false,
            },
            forecast: default_forecast_config(),
            subscriptions: default_subscriptions_config(),
            plans: default_plans_config(),
            alerts: default_alerts_config(),
            wealth: default_wealth_config(),
            ai: default_ai_config(),
            privacy: default_privacy_config(),
            exchanges: default_exchanges_config(),
            portfolio: default_portfolio_config(),
            forecast_ml: default_forecast_ml_config(),
            analytics: default_analytics_config(),
            database_bootstrap_url: None,
        }
    }
}
