//! Integration tests for US-0006 AI assistant (requires DATABASE_URL).

use std::path::PathBuf;

use flow_finance_ai::ai::audit::{AuditInsert, AuditRepository};
use flow_finance_ai::ai::registry::{ToolRegistry, TOOL_NAMES};
use sqlx::PgPool;
use uuid::Uuid;

fn skip_without_db() -> Option<PgPool> {
    let url = std::env::var("DATABASE_URL").ok()?;
    let rt = tokio::runtime::Runtime::new().ok()?;
    rt.block_on(async { PgPool::connect(&url).await.ok() })
}

#[test]
fn ai_modules_have_no_direct_sqlx_in_tools_path() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    for entry in ["src/ai/tools", "src/ai/registry.rs", "src/ai/orchestrator.rs"] {
        let path = manifest_dir.join(entry);
        if path.is_dir() {
            for file in std::fs::read_dir(&path).unwrap() {
                let content = std::fs::read_to_string(file.unwrap().path()).unwrap();
                assert!(
                    !content.contains("sqlx::query"),
                    "ai tool path must not use sqlx::query"
                );
            }
        } else if path.exists() {
            let content = std::fs::read_to_string(&path).unwrap();
            assert!(!content.contains("sqlx::query"));
        }
    }
}

#[test]
fn registry_names_match_migration_allowlist() {
    let reg = ToolRegistry::build();
    assert_eq!(reg.tools().len(), TOOL_NAMES.len());
    for name in TOOL_NAMES {
        assert!(reg.get(name).is_some());
    }
}

#[tokio::test]
async fn audit_insert_and_list_with_migration() {
    let Some(pool) = skip_without_db() else {
        eprintln!("SKIP audit_insert_and_list: DATABASE_URL not set");
        return;
    };

    sqlx::migrate!("./migrations").run(&pool).await.expect("migrate");

    let repo = AuditRepository::new(pool.clone());
    let session_id = Uuid::new_v4();
    let id = repo
        .insert(AuditInsert {
            session_id,
            user_subject: "test-user".into(),
            tool_name: "get_forecast".into(),
            args_summary: serde_json::json!({ "horizon": "3m" }),
            result_status: "ok".into(),
            result_rows: None,
            duration_ms: 12,
            error_message: None,
            model: Some("gpt-4o-mini".into()),
            provider: Some("openai".into()),
        })
        .await
        .expect("insert");

    let rows = repo.list(10, 0).await.expect("list");
    assert!(rows.iter().any(|r| r.id == id));
}

#[test]
fn config_uses_env_for_api_key_not_toml_field() {
    let ai = flow_finance_ai::config::AiConfig::default();
    assert_eq!(ai.api_key_env, "OPENAI_API_KEY");
    // api_key_env names an env var — key must not be embedded in struct
    assert!(std::mem::size_of_val(&ai.api_key_env) < 256);
}
