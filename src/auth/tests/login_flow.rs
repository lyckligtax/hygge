#![allow(incomplete_features)]
#![feature(async_fn_in_trait)]

use std::time::Duration;

use auth::AuthProvider;

use crate::fixtures::AccountProvider;
use crate::fixtures::Cache;

mod fixtures;

#[tokio::test]
async fn test_login() {
    let persistence = AccountProvider::default();
    let cache = Cache::default();
    let mut auth = AuthProvider::new(persistence, cache, Duration::from_secs(3600));

    let user_id = 1u32;
    let password = b"test1234";

    let internal_id = auth
        .create_account(user_id, password)
        .await
        .expect("Expected Account to be created");
    let login_token = auth
        .login(user_id, password)
        .await
        .expect("Expected LoginToken");
    let internal_id_verified = auth
        .verify_token(login_token)
        .await
        .expect("Expected InternalId");
    assert_eq!(internal_id, internal_id_verified)
}
