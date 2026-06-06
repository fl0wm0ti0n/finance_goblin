//! Pre-migration database bootstrap (US-0012 / DEC-0058).

use std::time::{Duration, Instant};

use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use url::Url;

use crate::config::{maintenance_url_user, redact_database_url, AppConfig};

/// Stable operator log codes (`bootstrap_reason` field).
pub const REASON_STARTED: &str = "database_bootstrap_started";
pub const REASON_CREATED: &str = "database_bootstrap_created";
pub const REASON_SKIPPED_EXISTS: &str = "database_bootstrap_skipped_exists";
pub const REASON_GRANTS_APPLIED: &str = "database_bootstrap_grants_applied";
pub const REASON_EXTENSION_OK: &str = "database_bootstrap_extension_ok";
pub const REASON_FAILED_PRIVILEGE: &str = "database_bootstrap_failed_privilege";
pub const REASON_FAILED_TIMESCALEDB: &str = "database_bootstrap_failed_timescaledb";
pub const REASON_FAILED_CONNECT: &str = "database_bootstrap_failed_connect";

const RUNBOOK_POINTER: &str =
    "See docs/engineering/runbook.md § Omniflow external deploy §1 (TimescaleDB host install).";

#[derive(Debug, thiserror::Error)]
pub enum BootstrapError {
    #[error("{bootstrap_reason}: maintenance database unreachable ({detail})")]
    Connect {
        bootstrap_reason: &'static str,
        detail: String,
    },
    #[error("{bootstrap_reason}: insufficient privilege to create database — set DATABASE_BOOTSTRAP_URL ({detail})")]
    Privilege {
        bootstrap_reason: &'static str,
        detail: String,
    },
    #[error("{bootstrap_reason}: TimescaleDB extension unavailable — {RUNBOOK_POINTER} ({detail})")]
    TimescaleDb {
        bootstrap_reason: &'static str,
        detail: String,
    },
    #[error("database bootstrap failed: {0}")]
    Other(String),
}

impl BootstrapError {
    pub fn bootstrap_reason(&self) -> &'static str {
        match self {
            BootstrapError::Connect { bootstrap_reason, .. } => bootstrap_reason,
            BootstrapError::Privilege { bootstrap_reason, .. } => bootstrap_reason,
            BootstrapError::TimescaleDb { bootstrap_reason, .. } => bootstrap_reason,
            BootstrapError::Other(_) => "database_bootstrap_failed",
        }
    }
}

/// Map SQL/database errors to bootstrap reason codes (unit-testable).
pub fn map_sql_error(err: &sqlx::Error) -> BootstrapError {
    if let sqlx::Error::Database(db) = err {
        if db.code().as_deref() == Some("42501") {
            return BootstrapError::Privilege {
                bootstrap_reason: REASON_FAILED_PRIVILEGE,
                detail: db.message().to_string(),
            };
        }
        let msg = db.message().to_lowercase();
        if msg.contains("timescaledb") || msg.contains("extension") {
            return BootstrapError::TimescaleDb {
                bootstrap_reason: REASON_FAILED_TIMESCALEDB,
                detail: db.message().to_string(),
            };
        }
    }
    let detail = err.to_string();
    if detail.to_lowercase().contains("timescaledb") {
        return BootstrapError::TimescaleDb {
            bootstrap_reason: REASON_FAILED_TIMESCALEDB,
            detail,
        };
    }
    BootstrapError::Other(detail)
}

struct BootstrapLogContext {
    host: String,
    port: u16,
    maintenance_db: String,
    app_db: String,
    app_user: String,
}

impl BootstrapLogContext {
    fn from_config(config: &AppConfig) -> Self {
        let maintenance_url = config.maintenance_database_url();
        let (host, port, maintenance_db) = parse_url_endpoint(&maintenance_url);
        Self {
            host,
            port,
            maintenance_db,
            app_db: config.database.name.clone(),
            app_user: config.database.user.clone(),
        }
    }
}

fn parse_url_endpoint(raw: &str) -> (String, u16, String) {
    match Url::parse(raw) {
        Ok(u) => {
            let host = u.host_str().unwrap_or("unknown").to_string();
            let port = u.port().unwrap_or(5432);
            let db = u.path().trim_start_matches('/').to_string();
            let maintenance_db = if db.is_empty() {
                "postgres".into()
            } else {
                db
            };
            (host, port, maintenance_db)
        }
        Err(_) => ("unknown".into(), 5432, "postgres".into()),
    }
}

async fn connect_with_startup_retry(
    url: &str,
    config: &AppConfig,
    redacted: &str,
) -> Result<PgPool, BootstrapError> {
    let initial = Duration::from_millis(config.database.startup_retry_initial_ms);
    let max_interval = Duration::from_millis(config.database.startup_retry_max_ms);
    let budget = Duration::from_millis(config.database.startup_retry_total_ms);

    let started = Instant::now();
    let mut delay = initial;
    let mut attempt = 0u32;

    loop {
        attempt += 1;
        match PgPoolOptions::new()
            .max_connections(1)
            .connect(url)
            .await
        {
            Ok(pool) => return Ok(pool),
            Err(err) => {
                let elapsed = started.elapsed();
                if elapsed >= budget {
                    tracing::error!(
                        bootstrap_reason = REASON_FAILED_CONNECT,
                        %err,
                        attempt,
                        url = %redacted,
                        elapsed_ms = elapsed.as_millis(),
                        "maintenance database connection failed after retry budget exhausted"
                    );
                    return Err(BootstrapError::Connect {
                        bootstrap_reason: REASON_FAILED_CONNECT,
                        detail: format!("after ~{}s: {err}", budget.as_secs()),
                    });
                }
                tracing::warn!(
                    bootstrap_reason = REASON_FAILED_CONNECT,
                    %err,
                    attempt,
                    url = %redacted,
                    retry_in_ms = delay.as_millis(),
                    "maintenance database connection failed, retrying"
                );
                tokio::time::sleep(delay).await;
                delay = (delay * 2).min(max_interval);
            }
        }
    }
}

async fn database_exists(pool: &PgPool, name: &str) -> Result<bool, BootstrapError> {
    let row = sqlx::query("SELECT 1 FROM pg_database WHERE datname = $1")
        .bind(name)
        .fetch_optional(pool)
        .await
        .map_err(|e| map_sql_error(&e))?;
    Ok(row.is_some())
}

async fn create_database(
    pool: &PgPool,
    db_name: &str,
    owner: &str,
) -> Result<(), BootstrapError> {
    let sql = format!(
        r#"CREATE DATABASE "{db_name}" OWNER "{owner}""#
    );
    sqlx::query(&sql)
        .execute(pool)
        .await
        .map_err(|e| map_sql_error(&e))?;
    Ok(())
}

async fn timescaledb_ready(pool: &PgPool) -> Result<bool, BootstrapError> {
    let row = sqlx::query("SELECT extversion FROM pg_extension WHERE extname = 'timescaledb'")
        .fetch_optional(pool)
        .await
        .map_err(|e| map_sql_error(&e))?;
    Ok(row.is_some())
}

async fn ensure_timescaledb_extension(pool: &PgPool) -> Result<(), BootstrapError> {
    if timescaledb_ready(pool).await? {
        return Ok(());
    }
    sqlx::query("CREATE EXTENSION IF NOT EXISTS timescaledb")
        .execute(pool)
        .await
        .map_err(|e| map_sql_error(&e))?;
    Ok(())
}

/// Idempotent database + TimescaleDB extension bootstrap before app pool connect.
pub async fn ensure_database(config: &AppConfig) -> Result<(), BootstrapError> {
    let ctx = BootstrapLogContext::from_config(config);
    let maintenance_url = config.maintenance_database_url();
    let redacted_maintenance = redact_database_url(&maintenance_url);

    let maintenance_pool =
        connect_with_startup_retry(&maintenance_url, config, &redacted_maintenance).await?;

    tracing::info!(
        bootstrap_reason = REASON_STARTED,
        host = %ctx.host,
        port = ctx.port,
        maintenance_db = %ctx.maintenance_db,
        app_db = %ctx.app_db,
        app_user = %ctx.app_user,
        "database bootstrap started"
    );

    let exists = database_exists(&maintenance_pool, &config.database.name).await?;
    let maintenance_user = maintenance_url_user(&maintenance_url);
    let created = if exists {
        tracing::info!(
            bootstrap_reason = REASON_SKIPPED_EXISTS,
            app_db = %ctx.app_db,
            "application database already exists"
        );
        false
    } else {
        create_database(
            &maintenance_pool,
            &config.database.name,
            &config.database.user,
        )
        .await?;
        tracing::info!(
            bootstrap_reason = REASON_CREATED,
            app_db = %ctx.app_db,
            app_user = %ctx.app_user,
            "created application database"
        );
        if maintenance_user
            .as_ref()
            .is_some_and(|u| u != &config.database.user)
        {
            tracing::info!(
                bootstrap_reason = REASON_GRANTS_APPLIED,
                app_db = %ctx.app_db,
                app_user = %ctx.app_user,
                "database owner set via CREATE DATABASE OWNER"
            );
        }
        true
    };

    let _ = created;
    maintenance_pool.close().await;

    let app_maint_url = config.app_database_maintenance_url();
    let redacted_app = redact_database_url(&app_maint_url);
    let app_pool = connect_with_startup_retry(&app_maint_url, config, &redacted_app).await?;

    ensure_timescaledb_extension(&app_pool).await?;
    tracing::info!(
        bootstrap_reason = REASON_EXTENSION_OK,
        app_db = %ctx.app_db,
        "TimescaleDB extension present or created"
    );
    app_pool.close().await;

    Ok(())
}

impl super::DbPool {
    pub async fn ensure_database(config: &AppConfig) -> Result<(), BootstrapError> {
        ensure_database(config).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn privilege_error_reason_code() {
        let err = BootstrapError::Privilege {
            bootstrap_reason: REASON_FAILED_PRIVILEGE,
            detail: "permission denied".into(),
        };
        assert_eq!(err.bootstrap_reason(), REASON_FAILED_PRIVILEGE);
    }

    #[test]
    fn reason_codes_are_stable() {
        assert_eq!(REASON_STARTED, "database_bootstrap_started");
        assert_eq!(REASON_FAILED_TIMESCALEDB, "database_bootstrap_failed_timescaledb");
    }

    #[test]
    fn bootstrap_error_display_omits_password() {
        let err = BootstrapError::Connect {
            bootstrap_reason: REASON_FAILED_CONNECT,
            detail: "connection refused".into(),
        };
        let msg = err.to_string();
        assert!(!msg.contains("secret"));
        assert!(msg.contains(REASON_FAILED_CONNECT));
    }

    #[test]
    fn timescaledb_error_includes_runbook_pointer() {
        let err = BootstrapError::TimescaleDb {
            bootstrap_reason: REASON_FAILED_TIMESCALEDB,
            detail: "extension not found".into(),
        };
        assert!(err.to_string().contains("runbook"));
    }

    #[test]
    fn map_sql_error_detects_timescaledb_message() {
        let err = sqlx::Error::Configuration("timescaledb control file missing".into());
        match map_sql_error(&err) {
            BootstrapError::TimescaleDb { bootstrap_reason, .. } => {
                assert_eq!(bootstrap_reason, REASON_FAILED_TIMESCALEDB);
            }
            other => panic!("expected TimescaleDb, got {other:?}"),
        }
    }
}
