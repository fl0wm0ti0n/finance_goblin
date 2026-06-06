use sqlx::PgPool;

pub async fn record_request(
    pool: &PgPool,
    method: &str,
    path: &str,
    status_code: Option<i32>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO firefly_request_audit (method, path, status_code) VALUES ($1, $2, $3)",
    )
    .bind(method)
    .bind(path)
    .bind(status_code)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn has_non_get_requests(pool: &PgPool) -> Result<bool, sqlx::Error> {
    let row = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM firefly_request_audit WHERE method <> 'GET'",
    )
    .fetch_one(pool)
    .await?;
    Ok(row > 0)
}
