use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::Mutex;
use uuid::Uuid;

use crate::ai::audit::{AuditRepository, AuditRow};
use crate::ai::orchestrator::AiOrchestrator;
use crate::ai::privacy::PrivacyLayer;
use crate::ai::provider::{build_provider, AiProvider, ProviderError};
use crate::ai::registry::ToolRegistry;
use crate::ai::types::ToolContext;
use crate::alerts::AlertService;
use crate::config::{AiConfig, AiPublicSettings, AppConfig, PrivacyConfig};
use crate::db::DbPool;
use crate::exchanges::repository::ExchangeRepository;
use crate::forecast::ForecastService;
use crate::plan::PlanService;
use crate::portfolio::PortfolioEngine;
use crate::subscriptions::SubscriptionService;
use crate::transactions::TransactionsService;
use crate::wealth::WealthService;

#[derive(Clone)]
pub struct AiService {
    pub config: AiConfig,
    provider: Arc<dyn AiProvider>,
    pub privacy_config: PrivacyConfig,
    pub privacy_layer: Arc<PrivacyLayer>,
    pub registry: Arc<ToolRegistry>,
    pub audit: Arc<AuditRepository>,
    pub orchestrator: Arc<AiOrchestrator>,
    rate_limits: Arc<Mutex<HashMap<String, Vec<Instant>>>>,
    transactions: TransactionsService,
    subscriptions: SubscriptionService,
    forecast: ForecastService,
    plans: PlanService,
    wealth: WealthService,
    alerts: AlertService,
    exchange_repo: Option<Arc<ExchangeRepository>>,
    portfolio: Option<PortfolioEngine>,
}

impl AiService {
    pub fn new(
        db: DbPool,
        app_config: &AppConfig,
        transactions: TransactionsService,
        subscriptions: SubscriptionService,
        forecast: ForecastService,
        plans: PlanService,
        wealth: WealthService,
        alerts: AlertService,
        exchange_repo: Option<Arc<ExchangeRepository>>,
        portfolio: Option<PortfolioEngine>,
    ) -> Result<Self, ProviderError> {
        let provider = build_provider(&app_config.ai)?;
        let privacy_layer = Arc::new(PrivacyLayer::new(app_config.privacy.clone()));
        let registry = Arc::new(ToolRegistry::build());
        let audit = Arc::new(AuditRepository::new(db.pool().clone()));
        let orchestrator = Arc::new(AiOrchestrator::new(
            app_config.ai.clone(),
            registry.clone(),
            privacy_layer.clone(),
            audit.clone(),
        ));

        Ok(Self {
            config: app_config.ai.clone(),
            provider,
            privacy_config: app_config.privacy.clone(),
            privacy_layer,
            registry,
            audit,
            orchestrator,
            rate_limits: Arc::new(Mutex::new(HashMap::new())),
            transactions,
            subscriptions,
            forecast,
            plans,
            wealth,
            alerts,
            exchange_repo,
            portfolio,
        })
    }

    pub fn provider(&self) -> Arc<dyn AiProvider> {
        self.provider.clone()
    }

    pub fn ai_public_settings(&self) -> AiPublicSettings {
        let p = self.provider.as_ref();
        self.config.public_settings(p.display_label(), p.base_url(), p.is_local())
    }

    pub async fn run_audit_retention(&self) -> Result<(), sqlx::Error> {
        self.audit
            .purge_expired(self.config.audit_retention_days, self.config.audit_max_rows)
            .await
    }

    pub fn tool_context(&self, user_subject: &str, session_id: Uuid) -> ToolContext {
        ToolContext {
            transactions: self.transactions.clone(),
            subscriptions: self.subscriptions.clone(),
            forecast: self.forecast.clone(),
            plans: self.plans.clone(),
            wealth: self.wealth.clone(),
            alerts: self.alerts.clone(),
            exchange_repo: self.exchange_repo.clone(),
            portfolio: self.portfolio.clone(),
            privacy: self.privacy_config.clone(),
            user_subject: user_subject.to_string(),
            session_id,
        }
    }

    pub async fn check_rate_limit(&self, user_subject: &str) -> bool {
        let limit = self.config.rate_limit_per_min as usize;
        let window = Duration::from_secs(60);
        let now = Instant::now();
        let mut map = self.rate_limits.lock().await;
        let entries = map.entry(user_subject.to_string()).or_default();
        entries.retain(|t| now.duration_since(*t) < window);
        if entries.len() >= limit {
            return false;
        }
        entries.push(now);
        true
    }

    pub async fn list_audit(&self, limit: i64, offset: i64) -> Result<Vec<AuditRow>, sqlx::Error> {
        self.audit.list(limit, offset).await
    }
}
