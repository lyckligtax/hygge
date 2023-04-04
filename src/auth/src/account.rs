#[derive(Debug, Copy, Clone)]
pub enum AccountStatus {
    Ok,
    Disabled,
    Removed,
}

#[derive(Debug, Clone)]
pub struct Account<InternalId, ExternalId> {
    pub id_internal: InternalId,
    pub id_external: ExternalId,
    pub password_hash: String,
    pub status: AccountStatus,
}
