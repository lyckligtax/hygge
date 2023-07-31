use crate::services::auth::Auth;
use axum::extract::FromRef;
use uuid::Uuid;

pub type RedisPool = deadpool_redis::Pool;
pub type RedisConnection = deadpool_redis::Connection;

#[derive(Clone, FromRef)]
pub struct Services {
    pub auth: Auth,
    pub redis: RedisPool,
}

pub struct UserId(pub Uuid);
