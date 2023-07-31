use deadpool_redis::redis::{cmd, AsyncCommands};
use std::str::FromStr;
use std::time::Duration;
use uuid::Uuid;

use crate::types::RedisConnection;
use auth::{AuthenticationError, LoginTokenIO};

#[derive(Default, Clone)]
pub struct LoginTokenProvider {
    ttl: u64,
}

impl LoginTokenIO for LoginTokenProvider {
    type InternalId = Uuid;
    type LoginToken = Uuid;
    type LoginCtx = RedisConnection;

    async fn insert(
        &mut self,
        internal_id: &Self::InternalId,
        ctx: &mut Self::LoginCtx,
    ) -> Result<Self::LoginToken, AuthenticationError> {
        let login_token = Uuid::new_v4();
        if redis::pipe()
            .cmd("SET")
            .arg(LoginTokenProvider::create_key(&login_token))
            .arg(internal_id.to_string())
            .arg("EX")
            .arg(self.ttl)
            .ignore()
            .cmd("SET")
            .arg(LoginTokenProvider::create_reverse_key(
                internal_id,
                &login_token,
            ))
            .arg(login_token.to_string())
            .arg("EX")
            .arg(self.ttl)
            .ignore()
            .query_async::<_, ()>(ctx)
            .await
            .is_ok()
        {
            Ok(login_token)
        } else {
            Err(AuthenticationError::IO)
        }
    }

    async fn remove(
        &mut self,
        login_token: &Self::LoginToken,
        ctx: &mut Self::LoginCtx,
    ) -> Result<(), AuthenticationError> {
        if ctx
            .del::<_, ()>(LoginTokenProvider::create_key(login_token))
            .await
            .is_ok()
        {
            Ok(())
        } else {
            Err(AuthenticationError::IO)
        }
    }

    async fn remove_all(
        &mut self,
        internal_id: &Self::InternalId,
        ctx: &mut Self::LoginCtx,
    ) -> Result<(), AuthenticationError> {
        let Ok(mut iter) = redis::cmd("SCAN")
            .arg("MATCH")
            .arg(LoginTokenProvider::create_reverse_key_match(internal_id))
            .cursor_arg(0)
            .clone()
            .iter_async::<String>(ctx)
            .await
        else {
            return Err(AuthenticationError::IO);
        };

        let mut keys_to_remove = &mut cmd("DEL");
        while let Some(key) = iter.next_item().await {
            let p = key.split(':').last().expect("expected to get uuid str");
            let login_token = Uuid::from_str(p).expect("expected to get uuid");
            keys_to_remove = keys_to_remove.arg(key);
            keys_to_remove = keys_to_remove.arg(LoginTokenProvider::create_key(&login_token));
        }

        drop(iter);

        if keys_to_remove.query_async::<_, ()>(ctx).await.is_ok() {
            Ok(())
        } else {
            Err(AuthenticationError::IO)
        }
    }

    async fn get(
        &self,
        login_token: &Self::LoginToken,
        ctx: &mut Self::LoginCtx,
    ) -> Option<Self::InternalId> {
        if let Ok(res) = ctx
            .get::<_, String>(LoginTokenProvider::create_key(login_token))
            .await
        {
            if let Ok(internal_id) = Uuid::from_str(&res) {
                return Some(internal_id);
            }
        }
        None
    }
}

impl LoginTokenProvider {
    pub fn new(ttl: Duration) -> Self {
        Self { ttl: ttl.as_secs() }
    }
    fn create_key(login_token: &<LoginTokenProvider as LoginTokenIO>::LoginToken) -> String {
        format!("auth:login:{login_token}")
    }
    fn create_reverse_key(
        internal_id: &<LoginTokenProvider as LoginTokenIO>::InternalId,
        login_token: &<LoginTokenProvider as LoginTokenIO>::LoginToken,
    ) -> String {
        format!("auth:login:reverse:{internal_id}:{login_token}")
    }
    fn create_reverse_key_match(
        internal_id: &<LoginTokenProvider as LoginTokenIO>::InternalId,
    ) -> String {
        format!("*auth:login:reverse:{internal_id}:*")
    }
}
