#[derive(Debug, Copy, Clone)]
pub enum AccountStatus {
    Active,
    Inactive,
    Removed,
}

#[derive(Debug, Clone)]
pub struct Account<InternalId, ExternalId> {
    pub id_internal: InternalId,
    pub id_external: ExternalId,
    pub hash: String,
    pub status: AccountStatus,
}
