use crate::services::auth::io::{TokenError, TokenIO};
use deadpool_redis::redis::AsyncCommands;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::ops::Add;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use uuid::Uuid;

use crate::types::RedisConnection;
#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    id: Uuid,
    user_id: Uuid,
    //TODO: extend according to docs
    exp: u64,
}

#[derive(Clone)]
pub struct LocalTokenIO {
    ttl: Duration,
    enc_key: EncodingKey,
    dec_key: DecodingKey,
    header: Header,
    validation: Validation,
}

impl TokenIO for LocalTokenIO {
    type Ctx = RedisConnection;

    async fn create(
        &mut self,
        internal_id: &Uuid,
        _ctx: &mut Self::Ctx, //TODO: remove from trait because we only ever use it without?
    ) -> Result<String, TokenError> {
        let Ok(expiry) = SystemTime::now().add(self.ttl).duration_since(UNIX_EPOCH) else {
            return Err(TokenError::Invalid);
        };
        let claim = Claims {
            id: Uuid::new_v4(),
            user_id: *internal_id,
            exp: expiry.as_secs(),
        };
        encode(&self.header, &claim, &self.enc_key).or(Err(TokenError::Invalid))
    }

    async fn revoke(&mut self, token: &str, ctx: &mut Self::Ctx) -> Option<TokenError> {
        //TODO move decode and encode into own functions
        let Ok(token) = decode::<Claims>(token, &self.dec_key, &self.validation) else {
            // Token is invalid
            // - corrupt
            // - unable to deserialize claim
            // - expired
            return Some(TokenError::Invalid);
        };

        //TODO: calculate correct ttl left

        if redis::cmd("SET")
            .arg(token.claims.id.to_string())
            .arg(1)
            .arg("EX")
            .arg(self.ttl.as_secs())
            .query_async::<Self::Ctx, ()>(ctx)
            .await
            .is_ok()
        {
            None
        } else {
            Some(TokenError::IO)
        }
    }

    async fn revoke_all(&mut self, id: &Uuid, ctx: &mut Self::Ctx) -> Option<TokenError> {
        let Ok(expiry) = SystemTime::now().duration_since(UNIX_EPOCH) else {
            return Some(TokenError::IO);
        };

        if redis::cmd("SET")
            .arg(id.to_string())
            .arg(expiry.as_secs())
            // we do not know when the last token had been issued
            // wait until all tokens must have been expired
            .arg("EX")
            .arg(self.ttl.as_secs())
            .query_async::<Self::Ctx, ()>(ctx)
            .await
            .is_ok()
        {
            None
        } else {
            Some(TokenError::IO)
        }
    }

    async fn verify(&self, token: &str, ctx: &mut Self::Ctx) -> Result<Uuid, TokenError> {
        let Ok(token) = decode::<Claims>(token, &self.dec_key, &self.validation) else {
            // Token is invalid
            // - corrupt
            // - unable to deserialize claim
            // - expired
            return Err(TokenError::Invalid);
        };

        if let Ok(a) = ctx.exists::<String, i32>(token.claims.id.to_string()).await && a == 1 {
            // token with id has been revoked 
            return Err(TokenError::Invalid);
        }

        if let Ok(revoked_at) = ctx
            .get::<String, usize>(token.claims.user_id.to_string())
            .await
        {
            // all tokens for this user issued before revoked_at have been revoked
            let user_tokens_revoked_at: SystemTime =
                UNIX_EPOCH + Duration::from_secs(revoked_at as u64);
            let current_token_issued_at: SystemTime =
                UNIX_EPOCH + (Duration::from_secs(token.claims.exp) - self.ttl);
            if user_tokens_revoked_at >= current_token_issued_at {
                // token has been issued before all user tokens have been revoked
                return Err(TokenError::Invalid);
            }
        }

        Ok(token.claims.user_id)
    }
}

impl LocalTokenIO {
    pub fn new(ttl: Duration, key: &str) -> Self {
        Self {
            ttl,
            enc_key: EncodingKey::from_secret(key.as_ref()),
            dec_key: DecodingKey::from_secret(key.as_ref()),
            header: Header::new(Algorithm::HS256),
            validation: Validation::default(),
        }
    }
}

#[cfg(test)]
mod tests {

    #[tokio::test]
    async fn test_create_token() {}
}
