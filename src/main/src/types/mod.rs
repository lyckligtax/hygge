use crate::services::auth::{AccountProvider, LoginTokenProvider};
use auth::Authentication;
use axum::extract::FromRef;
use uuid::Uuid;

pub type Auth = Authentication<AccountProvider, LoginTokenProvider>;
pub type RedisPool = deadpool_redis::Pool;
pub type RedisConnection = deadpool_redis::Connection;

#[derive(Clone, FromRef)]
pub struct Services {
    pub auth: Auth,
    pub redis: RedisPool,
}

pub struct UserId(pub Uuid);
