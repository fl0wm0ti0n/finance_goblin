use std::sync::Arc;

use uuid::Uuid;

use crate::exchanges::repository::ExchangeRepository;

pub struct BaselineService {
    repo: Arc<ExchangeRepository>,
}

impl BaselineService {
    pub fn new(repo: Arc<ExchangeRepository>) -> Self {
        Self { repo }
    }

    pub async fn capture_if_missing(
        &self,
        exchange_id: &str,
        total_eur: f64,
        run_id: Uuid,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO portfolio_baselines (exchange_id, baseline_eur, sync_run_id)
            SELECT $1, $2, $3
            WHERE NOT EXISTS (SELECT 1 FROM portfolio_baselines WHERE exchange_id = $1)
            "#,
        )
        .bind(exchange_id)
        .bind(total_eur)
        .bind(run_id)
        .execute(self.repo.pool())
        .await?;
        Ok(())
    }

    pub async fn total_baseline_eur(&self) -> Result<f64, sqlx::Error> {
        let sum: Option<f64> = sqlx::query_scalar(
            "SELECT SUM(baseline_eur)::float8 FROM portfolio_baselines",
        )
        .fetch_one(self.repo.pool())
        .await?;
        Ok(sum.unwrap_or(0.0))
    }
}
