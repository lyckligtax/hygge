use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

pub fn create_redis_pool(url: &str) -> Option<deadpool_redis::Pool> {
    let cfg = deadpool_redis::Config::from_url(url);
    cfg.create_pool(Some(deadpool_redis::Runtime::Tokio1)).ok()
}

pub async fn create_postgres_pool(url: &str) -> Option<PgPool> {
    PgPoolOptions::new()
        .max_connections(100)
        .connect(url)
        .await
        .ok()
}
