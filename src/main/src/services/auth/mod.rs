mod account;
mod login_token;
mod permission;

use auth::Authentication;
use std::time::Duration;

pub type Auth = Authentication<account::Provider, login_token::Provider, permission::Provider>;

pub fn create_auth_service(login_ttl: Duration) -> Auth {
    Authentication::new(
        account::Provider::new(),
        login_token::Provider::new(login_ttl),
        permission::Provider::new(),
    )
}
