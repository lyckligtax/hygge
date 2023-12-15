use crate::services::auth::io::TokenIO;
use crate::services::auth::io_provider::LocalTokenIO;
use crate::types::RedisPool;
use crate::UserId;
use axum::async_trait;
use axum::extract::{FromRequestParts, State};
use axum::http::request::Parts;
use axum::http::{header, Request, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};

#[derive(Clone)]
pub struct AuthService {
    pub auth: LocalTokenIO,
    pub redis: RedisPool,
}
pub async fn auth_layer<B>(
    State(mut auth): State<AuthService>,
    mut request: Request<B>,
    next: Next<B>,
) -> Response {
    let Some(header) = request.headers().get(header::AUTHORIZATION) else {
        return StatusCode::UNAUTHORIZED.into_response();
    };
    let Ok(token) = header.to_str() else {
        return StatusCode::UNAUTHORIZED.into_response();
    };
    let Ok(mut conn) = auth.redis.get().await else {
        return StatusCode::UNAUTHORIZED.into_response();
    };
    let Ok(internal_id) = auth.auth.verify(&token.to_string(), &mut conn).await else {
        return StatusCode::UNAUTHORIZED.into_response();
    };

    // insert InternalId to be extractable later on
    if request
        .extensions_mut()
        .insert(UserId(internal_id))
        .is_some()
    {
        // inserted twice
        return StatusCode::UNAUTHORIZED.into_response();
    };

    next.run(request).await
}

#[async_trait]
impl<S: Sized> FromRequestParts<S> for UserId {
    type Rejection = ();

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let user_id = parts.extensions.get::<UserId>().ok_or(())?;

        Ok(*user_id)
    }
}
