#![allow(incomplete_features)]
#![feature(async_fn_in_trait)]
#![feature(let_chains)]

use axum::middleware::from_fn_with_state;
use axum::routing::get;
use axum::Router;
use dotenvy::dotenv;
use std::env;
use std::net::{IpAddr, SocketAddr};
use std::time::Duration;
mod helper;
mod services;
mod types;
mod web;

use crate::services::auth::create_auth_service;
use crate::web::auth_layer::AuthService;
use helper::{create_postgres_pool, create_redis_pool};
use types::{Services, UserId};

#[tokio::main]
async fn main() {
    // build config
    dotenv().expect("Expected to read .env file"); //TODO: make these into a singleton to use everywhere
    let postgres_url = env::var("DATABASE_URL").unwrap();
    let redis_url = env::var("REDIS_URL").unwrap();
    let hygge_url = env::var("HYGGE_URL").unwrap().parse::<IpAddr>().unwrap();

    // build connections
    let pg_pool = create_postgres_pool(&postgres_url).await.unwrap();
    let redis_pool = create_redis_pool(&redis_url).unwrap();

    let auth = create_auth_service(Duration::from_secs(60 * 60), "test1234");
    // build services
    let global_state = Services {
        auth: auth.clone(),
        redis: redis_pool.clone(),
    };

    // build axum router
    let app = Router::new()
        .route("/", get(|| async { "Hello" }))
        .layer(from_fn_with_state(
            AuthService {
                auth: auth.clone(),
                redis: redis_pool.clone(),
            },
            web::auth_layer::auth_layer,
        ))
        .merge(
            // unauthenticated
            Router::new().nest("/auth", web::auth_router()),
        )
        .with_state(global_state)
        .layer(from_fn_with_state(pg_pool, axum_tx_layer::sqlx::tx_layer))
        .layer(from_fn_with_state(
            redis_pool,
            axum_tx_layer::redis::tx_layer,
        ));

    // start server
    let addr = SocketAddr::new(hygge_url, 3000);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
