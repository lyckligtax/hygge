use crate::lazy::Lazy;
use crate::slot::Slot;
use crate::{Pool, TxPool, TxSlot};
use axum::extract::{FromRequestParts, State};
use axum::http::request::Parts;
use axum::http::Request;
use axum::middleware::Next;
use axum::response::Response;
use axum::{async_trait, http};
use sqlx::{PgPool, Postgres, Transaction};

type SqlxTransaction = Transaction<'static, Postgres>;

#[async_trait]
impl TxPool for Pool<PgPool> {
    type Tx = SqlxTransaction;

    async fn begin(&mut self) -> Option<Self::Tx> {
        self.0.begin().await.ok()
    }
}

pub async fn tx_layer<B>(
    State(pool): State<PgPool>,
    mut request: Request<B>,
    next: Next<B>,
) -> Response {
    let transaction = TxSlot::<SqlxTransaction>::bind(request.extensions_mut(), Pool(pool.clone()));
    let res = next.run(request).await;

    if !res.status().is_server_error() && !res.status().is_client_error() {
        let _ = transaction.commit().await;
    } else {
        let _ = transaction.rollback().await;
    }
    res
}

impl TxSlot<SqlxTransaction> {
    pub(crate) fn bind(extensions: &mut http::Extensions, pool: Pool<PgPool>) -> Self {
        let (slot, tx) = Slot::new_leased(None);
        extensions.insert(Lazy { pool, tx });
        Self(slot)
    }

    pub(crate) async fn commit(self) -> Result<(), ()> {
        if let Some(tx) = self.0.into_inner().flatten().and_then(Slot::into_inner) {
            tx.commit().await.or(Err(()))
        } else {
            Ok(())
        }
    }

    pub(crate) async fn rollback(self) -> Result<(), ()> {
        if let Some(tx) = self.0.into_inner().flatten().and_then(Slot::into_inner) {
            tx.rollback().await.or(Err(()))
        } else {
            Ok(())
        }
    }
}

#[async_trait]
impl<S: Sized> FromRequestParts<S> for crate::Transaction<SqlxTransaction> {
    type Rejection = ();

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let ext: &mut Lazy<Pool<PgPool>> = parts.extensions.get_mut().ok_or(())?;
        let tx = ext.get_or_begin().await?;

        Ok(Self(tx))
    }
}
