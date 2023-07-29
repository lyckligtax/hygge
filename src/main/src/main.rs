#![allow(incomplete_features)]
#![feature(async_fn_in_trait)]

use ::auth::Authentication;
use axum::extract::FromRef;
use axum::middleware::{from_extractor_with_state, from_fn_with_state};
use axum::routing::get;
use axum::Router;
use dotenvy::dotenv;
use sqlx::postgres::PgPoolOptions;
use std::env;
use std::net::SocketAddr;
use std::time::Duration;
use uuid::Uuid;
mod auth;
mod helper;
mod web;

use crate::auth::{AccountProvider, LoginTokenProvider};
use crate::helper::{create_redis_pool, RedisPool};

type Auth = Authentication<AccountProvider, LoginTokenProvider>;

#[derive(Clone, FromRef)]
pub struct GlobalState {
    pub auth: Auth,
    pub redis: RedisPool,
}

// An extractor that performs authorization.
pub struct UserId(pub Uuid);

#[tokio::main]
async fn main() {
    dotenv().expect("Expected to read .env file");

    // postgres
    let db_connection_string = env::var("DATABASE_URL").expect("expected DATABASE_URL");
    let Ok(db_pool) = PgPoolOptions::new()
        .max_connections(100)
        .connect((db_connection_string).as_ref())
        .await
    else {
        return;
    };

    let accounts = AccountProvider::default();
    let login_tokens = LoginTokenProvider::new(Duration::from_secs(60 * 60));
    let redis_pool = create_redis_pool("redis://127.0.0.1/0").unwrap();
    let global_state = GlobalState {
        auth: Authentication::new(accounts, login_tokens),
        redis: create_redis_pool("redis://127.0.0.1/0").unwrap(),
    };
    let app = Router::new()
        .route("/", get(|| async { "Hello" }))
        .route_layer(from_extractor_with_state::<UserId, GlobalState>(
            global_state.clone(),
        ))
        .merge(
            // unauthenticated
            Router::new().nest("/auth", web::auth_router()),
        )
        .with_state(global_state)
        .layer(from_fn_with_state(db_pool, axum_tx_layer::sqlx::tx_layer))
        .layer(from_fn_with_state(
            redis_pool,
            axum_tx_layer::redis::tx_layer,
        ));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
