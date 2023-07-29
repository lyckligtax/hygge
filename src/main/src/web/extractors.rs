use crate::{GlobalState, UserId};
use async_trait::async_trait;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::http::{header, StatusCode};
use std::str::FromStr;
use uuid::Uuid;

#[async_trait]
impl FromRequestParts<GlobalState> for UserId {
    type Rejection = StatusCode;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &GlobalState,
    ) -> Result<Self, Self::Rejection> {
        let Some(value) = parts.headers.get(header::AUTHORIZATION) else {
            return Err(StatusCode::UNAUTHORIZED);
        };

        let Ok(internal_id_str) = value.to_str() else {
            return Err(StatusCode::UNAUTHORIZED);
        };

        let Ok(login_token) = Uuid::from_str(internal_id_str) else {
            return Err(StatusCode::UNAUTHORIZED);
        };
        let mut conn = state.redis.get().unwrap();
        if let Ok(internal_id) = state.auth.verify_token(&login_token, &mut conn).await {
            Ok(UserId(internal_id))
        } else {
            Err(StatusCode::UNAUTHORIZED)
        }
    }
}
