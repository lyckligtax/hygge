use axum::extract::FromRef;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

pub type RedisPool = deadpool_redis::Pool;
pub type RedisConnection = deadpool_redis::Connection;

#[derive(Clone, FromRef)]
pub struct Services {
    pub token_provider: Arc<RwLock<crate::services::auth::io_provider::LocalTokenIO>>,
    pub account_provider: Arc<RwLock<crate::services::auth::io_provider::LocalAccountIO>>,
    pub redis: RedisPool,
}

#[derive(Copy, Clone, Debug)]
pub struct UserId(pub Uuid);
