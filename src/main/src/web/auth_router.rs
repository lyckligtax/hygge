use crate::{Auth, GlobalState};
use axum::{extract::State, response::IntoResponse, routing::post, Json, Router};
use axum_tx_layer::Transaction;
use serde::Deserialize;
use sqlx::Postgres;

pub fn auth_router() -> Router<GlobalState> {
    Router::new()
        .route("/login", post(login))
        .route("/create_account", post(create))
}

#[axum::debug_handler(state = GlobalState)]
async fn login(
    State(mut auth): State<Auth>,
    Transaction(mut tx): Transaction<sqlx::Transaction<'static, Postgres>>,
    Transaction(mut redis): Transaction<r2d2::PooledConnection<redis::Client>>,
    Json(login_data): Json<LoginPayload>,
) -> impl IntoResponse {
    let id = auth
        .login(
            &login_data.external_id,
            &login_data.password,
            &mut tx,
            &mut redis,
        )
        .await
        .expect("TODO: panic message");

    id.to_string().into_response()
}

#[axum::debug_handler(state = GlobalState)]
async fn create(
    State(mut auth): State<Auth>,
    Transaction(mut tx): Transaction<sqlx::Transaction<'static, Postgres>>,
    Json(login_data): Json<LoginPayload>,
) -> String {
    let id = auth
        .create_account(&login_data.external_id, &login_data.password, &mut tx)
        .await
        .expect("TODO: panic message");

    id.to_string()
}

#[derive(Debug, Deserialize)]
struct LoginPayload {
    external_id: String,
    password: String,
}
