mod account;
mod account_provider;
mod token_provider;

use auth::Authentication;
use std::time::Duration;

pub type Auth = Authentication<account_provider::Provider, token_provider::Provider>;

pub fn create_auth_service(login_ttl: Duration, key: &str) -> Auth {
    Authentication::new(
        account_provider::Provider::new(),
        token_provider::Provider::new(login_ttl, key),
    )
}
