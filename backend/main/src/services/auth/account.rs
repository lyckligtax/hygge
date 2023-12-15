use uuid::Uuid;

#[derive(Debug, Copy, Clone, sqlx::Type)]
#[sqlx(type_name = "account_status")]
#[sqlx(rename_all = "lowercase")]
pub enum AccountStatus {
    Active,
    Inactive,
    Removed,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Account {
    pub id: Uuid,
    pub id_external: String,
    pub hash: String,
    pub status: AccountStatus,
}
