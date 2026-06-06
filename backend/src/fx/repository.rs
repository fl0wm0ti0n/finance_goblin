use chrono::NaiveDate;
use sqlx::PgPool;

pub struct FxRepository {
    pool: PgPool,
}

impl FxRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn get_cached_rate(
        &self,
        base: &str,
        quote: &str,
        date: NaiveDate,
    ) -> Result<Option<f64>, sqlx::Error> {
        sqlx::query_scalar(
            r#"
            SELECT rate::float8 FROM fx_rates
            WHERE base = $1 AND quote = $2 AND rate_date = $3
            "#,
        )
        .bind(base)
        .bind(quote)
        .bind(date)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn upsert_rate(
        &self,
        date: NaiveDate,
        base: &str,
        quote: &str,
        rate: f64,
        provider: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO fx_rates (rate_date, base, quote, rate, provider)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (rate_date, base, quote) DO UPDATE SET
                rate = EXCLUDED.rate,
                provider = EXCLUDED.provider
            "#,
        )
        .bind(date)
        .bind(base)
        .bind(quote)
        .bind(rate)
        .bind(provider)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
