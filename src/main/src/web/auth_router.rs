use crate::services::auth::Auth;
use crate::types::Services;
use axum::http::StatusCode;
use axum::response::Response;
use axum::{extract::State, response::IntoResponse, routing::post, Json, Router};
use axum_tx_layer::Transaction;
use serde::Deserialize;
use sqlx::Postgres;

pub fn auth_router() -> Router<Services> {
    Router::new()
        .route("/login", post(login))
        .route("/create_account", post(create))
}

#[axum::debug_handler(state = Services)]
async fn login(
    State(mut auth): State<Auth>,
    mut tx: Transaction<sqlx::Transaction<'static, Postgres>>,
    mut redis: Transaction<deadpool_redis::Connection>,
    Json(login_data): Json<LoginPayload>,
) -> Response {
    if let Ok(id) = auth
        .login(
            &login_data.external_id,
            &login_data.password,
            &mut tx,
            &mut redis,
        )
        .await
    {
        (StatusCode::OK, id.to_string()).into_response()
    } else {
        (StatusCode::FORBIDDEN).into_response()
    }
}

#[axum::debug_handler(state = Services)]
async fn create(
    State(mut auth): State<Auth>,
    mut tx: Transaction<sqlx::Transaction<'static, Postgres>>,
    Json(login_data): Json<LoginPayload>,
) -> Response {
    if let Ok(id) = auth
        .create_account(
            &login_data.external_id,
            &login_data.password,
            None,
            &mut tx,
            &mut (),
        )
        .await
    {
        (StatusCode::CREATED, id.to_string()).into_response()
    } else {
        (StatusCode::PRECONDITION_FAILED).into_response()
    }
}

#[derive(Debug, Deserialize)]
struct LoginPayload {
    external_id: String,
    password: String,
}
