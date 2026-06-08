use std::sync::Arc;

use chrono::{Datelike, Duration, NaiveDate};

use crate::config::PrivacyConfig;
use crate::db::DbPool;

use super::repository::TransactionsRepository;
use super::types::{
    compute_expense_series_summary, compute_period_status, validate_expense_series_months,
    AggregateFilter, CategoryAggregate, CategoryCatalogItem, CategoryCatalogResponse,
    ExpenseSeriesCategory, ExpenseSeriesMeta, ExpenseSeriesResponse, GroupBy, MirrorDateBounds,
    TransactionAggregates, UNCATEGORIZED_CATEGORY_ID,
};

const RAW_ROW_CAP: i64 = 20;
const RAW_DEFAULT_DAYS: i64 = 30;
const MIN_CATEGORY_SEARCH_LEN: usize = 2;

#[derive(Clone)]
pub struct TransactionsService {
    repo: Arc<TransactionsRepository>,
    privacy: PrivacyConfig,
}

#[derive(Debug, thiserror::Error)]
pub enum TransactionsError {
    #[error("database error: {0}")]
    Db(#[from] sqlx::Error),
    #[error("invalid arguments: {0}")]
    InvalidArgs(String),
    #[error("category not found: {0}")]
    NotFound(String),
}

impl TransactionsService {
    pub fn new(db: DbPool, privacy: PrivacyConfig) -> Self {
        Self {
            repo: Arc::new(TransactionsRepository::new(db.pool().clone())),
            privacy,
        }
    }

    pub async fn aggregates(
        &self,
        filter: AggregateFilter,
    ) -> Result<TransactionAggregates, TransactionsError> {
        if let Some(ref keyword) = filter.category_search {
            let trimmed = keyword.trim();
            if trimmed.len() < MIN_CATEGORY_SEARCH_LEN {
                return Err(TransactionsError::InvalidArgs(format!(
                    "category_search must be at least {MIN_CATEGORY_SEARCH_LEN} characters after trim"
                )));
            }
        }

        let group_by = match filter.group_by {
            GroupBy::Category => "category",
            GroupBy::Month => "month",
        };

        let (min_date, max_date) = self.repo.mirror_date_bounds().await?;
        let mirror_date_bounds = MirrorDateBounds {
            min: min_date.map(|d| d.to_string()),
            max: max_date.map(|d| d.to_string()),
        };

        let search_attempted = filter.category_search.is_some();
        let category_id_ignored =
            search_attempted && filter.category_id.as_ref().is_some_and(|id| !id.is_empty());

        let (category_matches, category_matches_truncated, category_ids_filter) =
            if let Some(ref keyword) = filter.category_search {
                let trimmed = keyword.trim();
                let (matches, truncated) = self.repo.search_categories_by_name(trimmed).await?;
                let ids: Vec<String> = matches.iter().map(|m| m.category_id.clone()).collect();
                let filter = if ids.is_empty() {
                    Some(vec![])
                } else {
                    Some(ids)
                };
                (matches, truncated, filter)
            } else if let Some(ref id) = filter.category_id {
                if id.is_empty() {
                    (vec![], false, None)
                } else {
                    (vec![], false, Some(vec![id.clone()]))
                }
            } else {
                (vec![], false, None)
            };

        let summary = self
            .repo
            .period_summary(filter.period_start, filter.period_end)
            .await?;

        let mut result = TransactionAggregates {
            period_start: filter.period_start.to_string(),
            period_end: filter.period_end.to_string(),
            group_by: group_by.into(),
            total_transaction_count: summary.total_count,
            total_outflow: summary.total_outflow,
            total_inflow: summary.total_inflow,
            uncategorized_transaction_count: summary.uncategorized_count,
            period_status: compute_period_status(
                summary.total_count,
                summary.total_outflow,
                summary.uncategorized_count,
            ),
            mirror_date_bounds,
            category_search: filter
                .category_search
                .as_ref()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty()),
            category_matches,
            category_matches_truncated,
            search_attempted,
            category_id_ignored,
            by_category: None,
            by_month: None,
            raw_rows: None,
        };

        match filter.group_by {
            GroupBy::Category => {
                let rows = self
                    .repo
                    .aggregates_by_category_ids(
                        filter.period_start,
                        filter.period_end,
                        category_ids_filter.as_deref(),
                    )
                    .await?;
                result.by_category = Some(label_uncategorized_categories(rows));
            }
            GroupBy::Month => {
                let rows = self
                    .repo
                    .aggregates_by_month(filter.period_start, filter.period_end)
                    .await?;
                result.by_month = Some(rows);
            }
        }

        if self.privacy.allow_raw_transactions {
            let raw_start = filter
                .period_end
                .checked_sub_signed(Duration::days(RAW_DEFAULT_DAYS))
                .unwrap_or(filter.period_start);
            let rows = self
                .repo
                .raw_rows_capped(raw_start, filter.period_end, RAW_ROW_CAP)
                .await?;
            result.raw_rows = Some(rows);
        }

        Ok(result)
    }

    pub async fn list_categories_catalog(
        &self,
        search: Option<&str>,
    ) -> Result<CategoryCatalogResponse, TransactionsError> {
        if let Some(keyword) = search {
            let trimmed = keyword.trim();
            if trimmed.len() < MIN_CATEGORY_SEARCH_LEN {
                return Err(TransactionsError::InvalidArgs(format!(
                    "search query must be at least {MIN_CATEGORY_SEARCH_LEN} characters after trim"
                )));
            }
        }

        let (rows, truncated) = self
            .repo
            .list_categories_catalog(search.map(str::trim).filter(|s| !s.is_empty()))
            .await?;

        Ok(CategoryCatalogResponse {
            categories: rows
                .into_iter()
                .map(|(id, name)| CategoryCatalogItem { id, name })
                .collect(),
            truncated: truncated.then_some(true),
        })
    }

    pub async fn expense_series(
        &self,
        category_id: &str,
        months: u32,
        end: NaiveDate,
    ) -> Result<ExpenseSeriesResponse, TransactionsError> {
        validate_expense_series_months(months)
            .map_err(TransactionsError::InvalidArgs)?;

        let (period_start, period_end) = expense_series_window(end, months);
        let is_uncategorized = category_id == UNCATEGORIZED_CATEGORY_ID;

        if !is_uncategorized && !self.repo.category_exists(category_id).await? {
            return Err(TransactionsError::NotFound(category_id.to_string()));
        }

        let category_filter = if is_uncategorized {
            ExpenseSeriesCategory::Uncategorized
        } else {
            ExpenseSeriesCategory::MirrorId(category_id)
        };

        let months_data = self
            .repo
            .expense_series_by_month(category_filter, period_start, period_end)
            .await?;

        let transaction_count: i64 = months_data.iter().map(|m| m.transaction_count).sum();
        let summary = compute_expense_series_summary(&months_data);

        let (category_name, category_label, uncategorized) = if is_uncategorized {
            (
                None,
                Some("Uncategorized".to_string()),
                Some(true),
            )
        } else {
            let name = self.repo.category_name(category_id).await?;
            (name.clone(), None, Some(false))
        };

        Ok(ExpenseSeriesResponse {
            category_id: category_id.to_string(),
            category_name,
            category_label,
            uncategorized,
            months: months_data,
            summary,
            meta: ExpenseSeriesMeta {
                period_start: period_start.to_string(),
                period_end: period_end.to_string(),
            },
            transaction_count,
        })
    }
}

pub fn expense_series_window(end: NaiveDate, months: u32) -> (NaiveDate, NaiveDate) {
    let end_month = month_start(end);
    let start_month = subtract_months(end_month, months.saturating_sub(1));
    (start_month, end)
}

fn month_start(date: NaiveDate) -> NaiveDate {
    NaiveDate::from_ymd_opt(date.year(), date.month(), 1).unwrap_or(date)
}

fn subtract_months(date: NaiveDate, months: u32) -> NaiveDate {
    let mut y = date.year() as i32;
    let mut m = date.month() as i32 - months as i32;
    while m < 1 {
        m += 12;
        y -= 1;
    }
    NaiveDate::from_ymd_opt(y, m as u32, 1).unwrap_or(date)
}

fn label_uncategorized_categories(rows: Vec<CategoryAggregate>) -> Vec<CategoryAggregate> {
    rows.into_iter()
        .map(|mut row| {
            if row.category_id.is_none() {
                row.category_name = Some("Uncategorized".into());
            }
            row
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use crate::transactions::types::{compute_period_status, PeriodStatus};

    #[test]
    fn raw_cap_constant() {
        assert_eq!(RAW_ROW_CAP, 20);
    }

    #[test]
    fn aggregate_filter_defaults_period() {
        let start = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2026, 1, 31).unwrap();
        let f = AggregateFilter {
            period_start: start,
            period_end: end,
            category_id: None,
            category_search: None,
            group_by: GroupBy::Category,
        };
        assert_eq!(f.period_start, start);
    }

    #[test]
    fn label_uncategorized_categories_maps_null_name() {
        let rows = vec![
            CategoryAggregate {
                category_id: None,
                category_name: None,
                total_outflow: 10.0,
                total_inflow: 0.0,
                transaction_count: 1,
            },
            CategoryAggregate {
                category_id: Some("cat-1".into()),
                category_name: Some("Food".into()),
                total_outflow: 5.0,
                total_inflow: 0.0,
                transaction_count: 1,
            },
        ];
        let labeled = label_uncategorized_categories(rows);
        assert_eq!(labeled[0].category_name.as_deref(), Some("Uncategorized"));
        assert_eq!(labeled[1].category_name.as_deref(), Some("Food"));
    }

    #[test]
    fn aggregate_json_includes_summary_fields_without_raw_rows() {
        let agg = TransactionAggregates {
            period_start: "2026-01-01".into(),
            period_end: "2026-01-31".into(),
            group_by: "category".into(),
            total_transaction_count: 2,
            total_outflow: 50.0,
            total_inflow: 0.0,
            uncategorized_transaction_count: 0,
            period_status: PeriodStatus::RowsWithOutflow,
            mirror_date_bounds: MirrorDateBounds {
                min: Some("2025-06-05".into()),
                max: Some("2026-05-22".into()),
            },
            category_search: None,
            category_matches: vec![],
            category_matches_truncated: false,
            search_attempted: false,
            category_id_ignored: false,
            by_category: Some(vec![]),
            by_month: None,
            raw_rows: None,
        };
        let json = serde_json::to_value(&agg).unwrap();
        assert_eq!(json["total_transaction_count"], 2);
        assert_eq!(json["period_status"], "rows_with_outflow");
        assert!(json.get("raw_rows").unwrap().is_null());
        assert_eq!(json["mirror_date_bounds"]["min"], "2025-06-05");
        assert!(json["category_matches"].is_array());
    }

    #[test]
    fn compute_period_status_matches_service_priority() {
        assert_eq!(
            compute_period_status(0, 0.0, 0),
            PeriodStatus::NoRowsInPeriod
        );
    }

    #[test]
    fn expense_series_window_twelve_months() {
        let end = NaiveDate::from_ymd_opt(2026, 6, 15).unwrap();
        let (start, end_out) = super::expense_series_window(end, 12);
        assert_eq!(start, NaiveDate::from_ymd_opt(2025, 7, 1).unwrap());
        assert_eq!(end_out, end);
    }

    #[test]
    fn expense_series_window_rejects_over_cap_via_types() {
        use crate::transactions::types::{validate_expense_series_months, EXPENSE_SERIES_MAX_MONTHS};
        assert!(validate_expense_series_months(24).is_ok());
        assert!(validate_expense_series_months(EXPENSE_SERIES_MAX_MONTHS + 1).is_err());
    }
}
