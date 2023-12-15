use crate::services::auth::io_provider::LocalAccountIO;
use crate::services::auth::Auth;
use crate::types::Services;
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::Response;
use axum::{extract::State, response::IntoResponse, routing::post, Json, Router};
use axum_tx_layer::Transaction;
use serde::Deserialize;
use sqlx::Postgres;
use std::sync::Arc;

pub fn auth_router() -> Router<Arc<Services>> {
    Router::new()
        .route("/login", post(login))
        .route("/create_account", post(create))
}

#[axum::debug_handler(state = Arc<Services>)]
async fn login(
    mut auth_client: Auth,
    mut tx: Transaction<sqlx::Transaction<'static, Postgres>>,
    mut redis: Transaction<deadpool_redis::Connection>,
    Json(login_data): Json<LoginPayload>,
) -> Response {
    if let Ok(id) = auth_client
        .login(&login_data.id, &login_data.password, &mut tx, &mut redis)
        .await
    {
        let mut headers = HeaderMap::new();
        headers.insert(header::AUTHORIZATION, id.parse().unwrap());
        (StatusCode::OK, headers).into_response()
    } else {
        (StatusCode::FORBIDDEN).into_response()
    }
}

#[axum::debug_handler(state = Arc<Services>)]
async fn create(
    mut auth_client: Auth,
    mut tx: Transaction<sqlx::Transaction<'static, Postgres>>,
    Json(login_data): Json<LoginPayload>,
) -> Response {
    if let Ok(id) = auth_client
        .create_account(&login_data.id, &login_data.password, &mut tx)
        .await
    {
        (StatusCode::CREATED, id.to_string()).into_response()
    } else {
        (StatusCode::PRECONDITION_FAILED).into_response()
    }
}

#[derive(Debug, Deserialize)]
struct LoginPayload {
    id: String,
    password: String,
}
