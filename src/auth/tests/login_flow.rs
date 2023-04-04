mod account;
mod cache;

use crate::account::AccountProvider;
use crate::cache::Cache;
use auth::AuthProvider;
use std::time::Duration;

#[test]
fn test_login() {
    let persistence = AccountProvider::default();
    let cache = Cache::default();
    let mut auth = AuthProvider::new(persistence, cache, Duration::from_secs(3600));

    let user_id = 1u32;
    let password = b"test1234";

    let internal_id = auth
        .create_account(user_id, password)
        .expect("Expected Account to be created");
    let login_token = auth.login(user_id, password).expect("Expected LoginToken");
    let internal_id_verified = auth.verify_token(login_token).expect("Expected InternalId");
    assert_eq!(internal_id, internal_id_verified)
}
