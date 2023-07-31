use auth::{Account, AccountStatus};

#[derive(sqlx::Type, Debug, Copy, Clone)]
#[sqlx(rename_all = "lowercase")]
pub enum ProviderAccountStatus {
    Active,
    Inactive,
    Removed,
}

impl From<ProviderAccountStatus> for AccountStatus {
    fn from(value: ProviderAccountStatus) -> Self {
        match value {
            ProviderAccountStatus::Active => AccountStatus::Active,
            ProviderAccountStatus::Inactive => AccountStatus::Inactive,
            ProviderAccountStatus::Removed => AccountStatus::Removed,
        }
    }
}

#[derive(sqlx::FromRow, Debug, Clone)]
pub struct ProviderAccount<InternalId, ExternalId> {
    #[sqlx(rename = "id")]
    pub id_internal: InternalId,
    pub id_external: ExternalId,
    pub hash: String,
    pub status: ProviderAccountStatus,
}

impl<InternalId, ExternalId> From<ProviderAccount<InternalId, ExternalId>>
    for Account<InternalId, ExternalId>
{
    fn from(value: ProviderAccount<InternalId, ExternalId>) -> Self {
        Account {
            id_internal: value.id_internal,
            id_external: value.id_external,
            hash: value.hash,
            status: AccountStatus::from(value.status),
        }
    }
}
