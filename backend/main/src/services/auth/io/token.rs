#[cfg(test)]
use mockall::automock;
use uuid::Uuid;

/// create, verify and revoke tokens
#[cfg_attr(test, automock(type Ctx=();))]
pub trait TokenIO {
    type Ctx;

    async fn create(&mut self, id: &Uuid, ctx: &mut Self::Ctx) -> Result<String, TokenError>;
    async fn revoke(&mut self, token: &str, ctx: &mut Self::Ctx) -> Option<TokenError>;
    async fn revoke_all(&mut self, id: &Uuid, ctx: &mut Self::Ctx) -> Option<TokenError>;
    async fn verify(&self, token: &str, ctx: &mut Self::Ctx) -> Result<Uuid, TokenError>;
}

#[derive(Debug)]
pub enum TokenError {
    IO,
    Invalid,
}
