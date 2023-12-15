mod account;
mod client;
mod error;
pub(crate) mod io;
pub mod io_provider;

use crate::services::auth::client::AuthenticationClient;
use crate::services::auth::io_provider::{LocalAccountIO, LocalTokenIO};
use sqlx::{Postgres, Transaction};

pub type Auth = AuthenticationClient<
    Transaction<'static, Postgres>,
    RedisConnection,
    LocalAccountIO,
    LocalTokenIO,
>;
use crate::types::RedisConnection;
pub use io_provider::*;
