use chrono::NaiveDate;
use sqlx::PgPool;

use super::types::{
    CategoryAggregate, CategoryMatch, ExpenseSeriesCategory, ExpenseSeriesMonth, MonthAggregate,
    PeriodSummary, RawTransactionRow, CATEGORY_CATALOG_CAP,
};

const CATEGORY_SEARCH_LIMIT: i64 = 10;

pub struct TransactionsRepository {
    pool: PgPool,
}

impl TransactionsRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn period_summary(
        &self,
        start: NaiveDate,
        end: NaiveDate,
    ) -> Result<PeriodSummary, sqlx::Error> {
        let (count, outflow, inflow, uncategorized): (i64, f64, f64, i64) = sqlx::query_as(
            r#"
            SELECT COUNT(*)::bigint,
                   COALESCE(SUM(CASE WHEN amount::float8 < 0 THEN ABS(amount::float8) ELSE 0 END), 0),
                   COALESCE(SUM(CASE WHEN amount::float8 > 0 THEN amount::float8 ELSE 0 END), 0),
                   COUNT(*) FILTER (WHERE category_id IS NULL)::bigint
            FROM transactions
            WHERE date >= $1 AND date <= $2
            "#,
        )
        .bind(start)
        .bind(end)
        .fetch_one(&self.pool)
        .await?;

        Ok(PeriodSummary {
            total_count: count,
            total_outflow: outflow,
            total_inflow: inflow,
            uncategorized_count: uncategorized,
        })
    }

    pub async fn search_categories_by_name(
        &self,
        keyword: &str,
    ) -> Result<(Vec<CategoryMatch>, bool), sqlx::Error> {
        let pattern = format!("%{keyword}%");

        let total: i64 = sqlx::query_scalar(
            "SELECT COUNT(*)::bigint FROM categories WHERE name ILIKE $1",
        )
        .bind(&pattern)
        .fetch_one(&self.pool)
        .await?;

        let truncated = total > CATEGORY_SEARCH_LIMIT;

        let rows: Vec<(String, String)> = sqlx::query_as(
            r#"
            SELECT firefly_id, COALESCE(name, '')
            FROM categories
            WHERE name ILIKE $1
            ORDER BY name ASC
            LIMIT $2
            "#,
        )
        .bind(&pattern)
        .bind(CATEGORY_SEARCH_LIMIT)
        .fetch_all(&self.pool)
        .await?;

        Ok((
            rows.into_iter()
                .map(|(category_id, category_name)| CategoryMatch {
                    category_id,
                    category_name,
                })
                .collect(),
            truncated,
        ))
    }

    pub async fn mirror_date_bounds(
        &self,
    ) -> Result<(Option<NaiveDate>, Option<NaiveDate>), sqlx::Error> {
        sqlx::query_as("SELECT MIN(date), MAX(date) FROM transactions")
            .fetch_one(&self.pool)
            .await
    }

    pub async fn aggregates_by_category(
        &self,
        start: NaiveDate,
        end: NaiveDate,
        category_id: Option<&str>,
    ) -> Result<Vec<CategoryAggregate>, sqlx::Error> {
        let ids = category_id.map(|id| vec![id.to_string()]);
        self.aggregates_by_category_ids(start, end, ids.as_deref())
            .await
    }

    pub async fn aggregates_by_category_ids(
        &self,
        start: NaiveDate,
        end: NaiveDate,
        category_ids: Option<&[String]>,
    ) -> Result<Vec<CategoryAggregate>, sqlx::Error> {
        if matches!(category_ids, Some([])) {
            return Ok(vec![]);
        }

        let rows = if let Some(ids) = category_ids {
            sqlx::query_as::<_, (Option<String>, Option<String>, f64, f64, i64)>(
                r#"
                SELECT t.category_id, c.name,
                       COALESCE(SUM(CASE WHEN t.amount::float8 < 0 THEN ABS(t.amount::float8) ELSE 0 END), 0),
                       COALESCE(SUM(CASE WHEN t.amount::float8 > 0 THEN t.amount::float8 ELSE 0 END), 0),
                       COUNT(*)::bigint
                FROM transactions t
                LEFT JOIN categories c ON c.firefly_id = t.category_id
                WHERE t.date >= $1 AND t.date <= $2 AND t.category_id = ANY($3)
                GROUP BY t.category_id, c.name
                ORDER BY 3 DESC NULLS LAST
                "#,
            )
            .bind(start)
            .bind(end)
            .bind(ids)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as::<_, (Option<String>, Option<String>, f64, f64, i64)>(
                r#"
                SELECT t.category_id, c.name,
                       COALESCE(SUM(CASE WHEN t.amount::float8 < 0 THEN ABS(t.amount::float8) ELSE 0 END), 0),
                       COALESCE(SUM(CASE WHEN t.amount::float8 > 0 THEN t.amount::float8 ELSE 0 END), 0),
                       COUNT(*)::bigint
                FROM transactions t
                LEFT JOIN categories c ON c.firefly_id = t.category_id
                WHERE t.date >= $1 AND t.date <= $2
                GROUP BY t.category_id, c.name
                ORDER BY 3 DESC NULLS LAST
                "#,
            )
            .bind(start)
            .bind(end)
            .fetch_all(&self.pool)
            .await?
        };

        Ok(rows
            .into_iter()
            .map(|(id, name, outflow, inflow, count)| CategoryAggregate {
                category_id: id,
                category_name: name,
                total_outflow: outflow,
                total_inflow: inflow,
                transaction_count: count,
            })
            .collect())
    }

    pub async fn aggregates_by_month(
        &self,
        start: NaiveDate,
        end: NaiveDate,
    ) -> Result<Vec<MonthAggregate>, sqlx::Error> {
        let rows: Vec<(String, f64, f64, i64)> = sqlx::query_as(
            r#"
            SELECT to_char(date_trunc('month', date), 'YYYY-MM') AS month,
                   COALESCE(SUM(CASE WHEN amount::float8 < 0 THEN ABS(amount::float8) ELSE 0 END), 0),
                   COALESCE(SUM(CASE WHEN amount::float8 > 0 THEN amount::float8 ELSE 0 END), 0),
                   COUNT(*)::bigint
            FROM transactions
            WHERE date >= $1 AND date <= $2
            GROUP BY 1
            ORDER BY 1
            "#,
        )
        .bind(start)
        .bind(end)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|(month, outflow, inflow, count)| MonthAggregate {
                month,
                total_outflow: outflow,
                total_inflow: inflow,
                transaction_count: count,
            })
            .collect())
    }

    pub async fn list_categories_catalog(
        &self,
        search: Option<&str>,
    ) -> Result<(Vec<(String, String)>, bool), sqlx::Error> {
        let rows = if let Some(keyword) = search {
            let pattern = format!("%{keyword}%");
            let total: i64 = sqlx::query_scalar(
                "SELECT COUNT(*)::bigint FROM categories WHERE name ILIKE $1",
            )
            .bind(&pattern)
            .fetch_one(&self.pool)
            .await?;

            let truncated = total > CATEGORY_CATALOG_CAP;
            let fetched: Vec<(String, String)> = sqlx::query_as(
                r#"
                SELECT firefly_id, COALESCE(name, '')
                FROM categories
                WHERE name ILIKE $1
                ORDER BY name ASC
                LIMIT $2
                "#,
            )
            .bind(&pattern)
            .bind(CATEGORY_CATALOG_CAP)
            .fetch_all(&self.pool)
            .await?;
            (fetched, truncated)
        } else {
            let total: i64 =
                sqlx::query_scalar("SELECT COUNT(*)::bigint FROM categories")
                    .fetch_one(&self.pool)
                    .await?;
            let truncated = total > CATEGORY_CATALOG_CAP;
            let fetched: Vec<(String, String)> = sqlx::query_as(
                r#"
                SELECT firefly_id, COALESCE(name, '')
                FROM categories
                ORDER BY name ASC
                LIMIT $1
                "#,
            )
            .bind(CATEGORY_CATALOG_CAP)
            .fetch_all(&self.pool)
            .await?;
            (fetched, truncated)
        };

        Ok(rows)
    }

    pub async fn category_exists(&self, firefly_id: &str) -> Result<bool, sqlx::Error> {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*)::bigint FROM categories WHERE firefly_id = $1",
        )
        .bind(firefly_id)
        .fetch_one(&self.pool)
        .await?;
        Ok(count > 0)
    }

    pub async fn category_name(&self, firefly_id: &str) -> Result<Option<String>, sqlx::Error> {
        sqlx::query_scalar("SELECT name FROM categories WHERE firefly_id = $1")
            .bind(firefly_id)
            .fetch_optional(&self.pool)
            .await
    }

    pub async fn expense_series_by_month(
        &self,
        category: ExpenseSeriesCategory<'_>,
        start: NaiveDate,
        end: NaiveDate,
    ) -> Result<Vec<ExpenseSeriesMonth>, sqlx::Error> {
        let rows: Vec<(String, f64, f64, i64)> = match category {
            ExpenseSeriesCategory::Uncategorized => sqlx::query_as(
                r#"
                WITH month_spine AS (
                    SELECT generate_series(
                        date_trunc('month', $1::date),
                        date_trunc('month', $2::date),
                        '1 month'::interval
                    ) AS month_start
                )
                SELECT
                    to_char(m.month_start, 'YYYY-MM') AS month,
                    COALESCE(SUM(CASE WHEN t.amount::float8 < 0 THEN ABS(t.amount::float8) ELSE 0 END), 0),
                    COALESCE(SUM(CASE WHEN t.amount::float8 > 0 THEN t.amount::float8 ELSE 0 END), 0),
                    COUNT(t.*)::bigint
                FROM month_spine m
                LEFT JOIN transactions t
                    ON date_trunc('month', t.date) = m.month_start
                   AND t.date >= $1 AND t.date <= $2
                   AND t.category_id IS NULL
                GROUP BY m.month_start
                ORDER BY m.month_start
                "#,
            )
            .bind(start)
            .bind(end)
            .fetch_all(&self.pool)
            .await?,
            ExpenseSeriesCategory::MirrorId(category_id) => sqlx::query_as(
                r#"
                WITH month_spine AS (
                    SELECT generate_series(
                        date_trunc('month', $1::date),
                        date_trunc('month', $2::date),
                        '1 month'::interval
                    ) AS month_start
                )
                SELECT
                    to_char(m.month_start, 'YYYY-MM') AS month,
                    COALESCE(SUM(CASE WHEN t.amount::float8 < 0 THEN ABS(t.amount::float8) ELSE 0 END), 0),
                    COALESCE(SUM(CASE WHEN t.amount::float8 > 0 THEN t.amount::float8 ELSE 0 END), 0),
                    COUNT(t.*)::bigint
                FROM month_spine m
                LEFT JOIN transactions t
                    ON date_trunc('month', t.date) = m.month_start
                   AND t.date >= $1 AND t.date <= $2
                   AND t.category_id = $3
                GROUP BY m.month_start
                ORDER BY m.month_start
                "#,
            )
            .bind(start)
            .bind(end)
            .bind(category_id)
            .fetch_all(&self.pool)
            .await?,
        };

        Ok(rows
            .into_iter()
            .map(|(month, outflow, inflow, count)| ExpenseSeriesMonth {
                month,
                outflow_eur: outflow,
                inflow_eur: inflow,
                transaction_count: count,
            })
            .collect())
    }

    pub async fn raw_rows_capped(
        &self,
        start: NaiveDate,
        end: NaiveDate,
        limit: i64,
    ) -> Result<Vec<RawTransactionRow>, sqlx::Error> {
        let rows: Vec<(NaiveDate, f64, Option<String>, Option<String>, Option<String>)> =
            sqlx::query_as(
                r#"
                SELECT date, amount::float8, description, category_id, account_id
                FROM transactions
                WHERE date >= $1 AND date <= $2
                ORDER BY date DESC
                LIMIT $3
                "#,
            )
            .bind(start)
            .bind(end)
            .bind(limit)
            .fetch_all(&self.pool)
            .await?;

        Ok(rows
            .into_iter()
            .map(|(date, amount, description, category_id, account_id)| RawTransactionRow {
                date: date.to_string(),
                amount,
                description,
                category_id,
                account_id,
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::transactions::types::{compute_period_status, GroupBy, PeriodStatus};

    #[test]
    fn group_by_serialization() {
        let g = GroupBy::Category;
        assert_eq!(serde_json::to_string(&g).unwrap(), "\"category\"");
    }

    #[test]
    fn period_status_priority_order() {
        assert_eq!(
            compute_period_status(0, 0.0, 0),
            PeriodStatus::NoRowsInPeriod
        );
        assert_eq!(
            compute_period_status(5, 0.0, 5),
            PeriodStatus::RowsZeroOutflow
        );
        assert_eq!(
            compute_period_status(3, 100.0, 3),
            PeriodStatus::RowsUncategorized
        );
        assert_eq!(
            compute_period_status(3, 50.0, 1),
            PeriodStatus::RowsWithOutflow
        );
    }
}
