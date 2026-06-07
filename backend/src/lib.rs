pub mod ai;
pub mod alerts;
pub mod analytics;
pub mod api;
pub mod audit;
pub mod auth;
pub mod config;
pub mod db;
pub mod exchanges;
pub mod firefly;
pub mod forecast;
pub mod forecast_ml;
pub mod fx;
pub mod health;
pub mod plan;
pub mod portfolio;
pub mod recurrence;
pub mod subscriptions;
pub mod sync;
pub mod transactions;
pub mod wealth;

use std::net::SocketAddr;
use std::sync::Arc;

use axum::Router;
use tokio::signal;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::ai::AiService;
use crate::alerts::AlertService;
use crate::config::AppConfig;
use crate::exchanges::ExchangeService;
use crate::fx::FxService;
use crate::portfolio::PortfolioEngine;
use crate::transactions::TransactionsService;
use crate::db::DbPool;
use crate::forecast::ForecastService;
use crate::forecast_ml::ForecastMlService;
use crate::plan::PlanService;
use crate::plan::risk::PlanRiskService;
use crate::subscriptions::SubscriptionService;
use crate::sync::SyncService;
use crate::wealth::WealthService;

pub struct AppState {
    pub config: AppConfig,
    pub db: DbPool,
    pub sync: SyncService,
    pub forecast: ForecastService,
    pub forecast_ml: ForecastMlService,
    pub plan_risk: PlanRiskService,
    pub portfolio_forecast: crate::wealth::portfolio_forecast::PortfolioForecastService,
    pub subscriptions: SubscriptionService,
    pub plans: PlanService,
    pub wealth: WealthService,
    pub alerts: AlertService,
    pub ai: AiService,
    pub transactions: TransactionsService,
    pub fx: FxService,
    pub portfolio: PortfolioEngine,
    pub exchanges: ExchangeService,
}

pub async fn run() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
            "flow_finance_ai=info,tower_http=info".into()
        }))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = AppConfig::load()?;
    tracing::info!(
        database_mode = %config.database.mode,
        firefly_base = %config.firefly.base_url,
        "starting flow-finance-ai"
    );

    DbPool::ensure_database(&config).await?;
    let db = DbPool::connect_with_retry(&config).await?;
    db.run_migrations().await?;

    let forecast = ForecastService::new(db.clone(), config.forecast.clone());
    let forecast_repo = forecast.repository_arc();
    let forecast_ml = ForecastMlService::new(forecast_repo.clone(), config.forecast_ml.clone());
    let plan_risk = PlanRiskService::new(
        db.pool().clone(),
        forecast_repo.clone(),
        config.alerts.clone(),
        config.forecast_ml.clone(),
    );
    let portfolio_forecast =
        crate::wealth::portfolio_forecast::PortfolioForecastService::new(forecast_repo.clone());
    let subscriptions = SubscriptionService::new(db.clone(), config.subscriptions.clone());
    let plans = PlanService::new(db.clone(), config.plans.clone(), forecast.clone());
    forecast.attach_plan_service(plans.clone()).await;

    let wealth = WealthService::new(
        db.clone(),
        config.wealth.clone(),
        config.alerts.reporting_currency.clone(),
    );
    let fx = FxService::new(db.clone(), config.portfolio.clone());
    let portfolio = PortfolioEngine::new(db.clone(), config.portfolio.clone());
    let exchanges = ExchangeService::new(
        db.clone(),
        config.exchanges.clone(),
        fx.clone(),
        portfolio.clone(),
    );
    exchanges.mirror_enabled_at_startup().await?;
    portfolio
        .run_trade_retention(exchanges.repository().as_ref())
        .await?;

    let alerts = AlertService::new(
        db.clone(),
        config.alerts.clone(),
        wealth.clone(),
        Some(plans.clone()),
    );
    alerts.mirror_config_at_startup().await?;

    let transactions =
        TransactionsService::new(db.clone(), config.privacy.clone());
    let ai = AiService::new(
        db.clone(),
        &config,
        transactions.clone(),
        subscriptions.clone(),
        forecast.clone(),
        plans.clone(),
        wealth.clone(),
        alerts.clone(),
        Some(exchanges.repository()),
        Some(portfolio.clone()),
    )
    .map_err(|e| anyhow::anyhow!("AI provider startup failed: {e}"))?;
    forecast
        .attach_bucket_inference(ai.provider(), config.privacy.clone())
        .await;
    ai.run_audit_retention().await?;

    let sync = SyncService::new(
        config.clone(),
        db.clone(),
        forecast.clone(),
        forecast_ml.clone(),
        plan_risk.clone(),
        subscriptions.clone(),
        alerts.clone(),
        wealth.clone(),
        exchanges.clone(),
        portfolio.clone(),
    );
    sync.start_scheduler().await?;

    let state = Arc::new(AppState {
        config: config.clone(),
        db,
        sync,
        forecast,
        forecast_ml,
        plan_risk,
        portfolio_forecast,
        subscriptions,
        plans,
        wealth,
        alerts,
        ai,
        transactions,
        fx,
        portfolio,
        exchanges,
    });

    let app = build_router(state.clone());

    let addr = SocketAddr::from(([0, 0, 0, 0], config.server.port));
    tracing::info!(%addr, "listening");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    tracing::info!("shutdown complete");
    Ok(())
}

pub fn build_router(state: Arc<AppState>) -> Router {
    let api_router = api::routes(state.clone());
    let health_router = health::routes(state.clone());
    let grafana_upstream = state.config.grafana_upstream_url().expect("validated at startup");
    let analytics_router = analytics::grafana_routes(grafana_upstream);

    let static_dir = std::path::Path::new("/app/static");
    let dev_static = std::path::Path::new("frontend/dist");

    let mut router = Router::new()
        .merge(health_router)
        .merge(analytics_router)
        .merge(api_router)
        .layer(tower_http::trace::TraceLayer::new_for_http())
        .layer(tower_http::cors::CorsLayer::permissive());

    if static_dir.exists() {
        router = router.fallback_service(tower_http::services::ServeDir::new(static_dir));
    } else if dev_static.exists() {
        router = router.fallback_service(tower_http::services::ServeDir::new(dev_static));
    }

    router
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
