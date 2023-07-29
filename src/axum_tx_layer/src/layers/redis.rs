use crate::lazy::Lazy;
use crate::slot::Slot;
use crate::{Pool, Transaction, TxPool, TxSlot};
use axum::extract::{FromRequestParts, State};
use axum::http::request::Parts;
use axum::http::Request;
use axum::middleware::Next;
use axum::response::Response;
use axum::{async_trait, http};

type RedisTransaction = r2d2::PooledConnection<redis::Client>;

#[async_trait]
impl TxPool for Pool<r2d2::Pool<redis::Client>> {
    type Tx = RedisTransaction;

    async fn begin(&mut self) -> Option<Self::Tx> {
        todo!()
    }
}

pub async fn tx_layer<B>(
    State(pool): State<r2d2::Pool<redis::Client>>,
    mut request: Request<B>,
    next: Next<B>,
) -> Response {
    let transaction =
        TxSlot::<RedisTransaction>::bind(request.extensions_mut(), Pool(pool.clone()));
    let res = next.run(request).await;

    if !res.status().is_server_error() && !res.status().is_client_error() {
        let _ = transaction.commit().await;
    } else {
        let _ = transaction.rollback().await;
    }
    res
}

impl TxSlot<RedisTransaction> {
    pub(crate) fn bind(
        extensions: &mut http::Extensions,
        pool: Pool<r2d2::Pool<redis::Client>>,
    ) -> Self {
        let (slot, tx) = Slot::new_leased(None);
        extensions.insert(Lazy { pool, tx });
        Self(slot)
    }

    pub(crate) async fn commit(self) -> Result<(), ()> {
        if let Some(mut _tx) = self.0.into_inner().flatten().and_then(Slot::into_inner) {
            todo!("Committing Redis Tx...");
        } else {
            Ok(())
        }
    }

    pub(crate) async fn rollback(self) -> Result<(), ()> {
        if let Some(mut _tx) = self.0.into_inner().flatten().and_then(Slot::into_inner) {
            todo!("Rolling back Redis Tx...");
        } else {
            Ok(())
        }
    }
}

#[async_trait]
impl<S: Sized> FromRequestParts<S> for Transaction<RedisTransaction> {
    type Rejection = ();

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let ext: &mut Lazy<Pool<r2d2::Pool<redis::Client>>> =
            parts.extensions.get_mut().ok_or(())?;
        let tx = ext.get_or_begin().await?;

        Ok(Self(tx))
    }
}
