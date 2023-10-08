#[cfg(test)]
use mockall::automock;

/// create, verify and revoke tokens
#[cfg_attr(test, automock(type InternalId=u32; type LoginToken=u32; type Ctx=();))]
pub trait TokenIO {
    type InternalId;
    type LoginToken;
    type Ctx;

    async fn create(
        &mut self,
        id: &Self::InternalId,
        ctx: &mut Self::Ctx,
    ) -> Result<Self::LoginToken, TokenError>;
    async fn revoke(&mut self, token: &Self::LoginToken, ctx: &mut Self::Ctx)
        -> Option<TokenError>;
    async fn revoke_all(
        &mut self,
        id: &Self::InternalId,
        ctx: &mut Self::Ctx,
    ) -> Option<TokenError>;
    async fn verify(
        &mut self,
        token: &Self::LoginToken,
        ctx: &mut Self::Ctx,
    ) -> Result<Self::InternalId, TokenError>;
}

#[derive(Debug)]
pub enum TokenError {
    IO,
    Invalid,
}
