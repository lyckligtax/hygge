#[derive(Debug, Copy, Clone, sqlx::Type)]
#[sqlx(type_name = "account_status")]
#[sqlx(rename_all = "lowercase")]
pub enum AccountStatus {
    Active,
    Inactive,
    Removed,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Account<InternalId, ExternalId> {
    pub id: InternalId,
    pub id_external: ExternalId,
    pub hash: String,
    pub status: AccountStatus,
}
