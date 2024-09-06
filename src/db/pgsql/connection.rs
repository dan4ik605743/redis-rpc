use once_cell::sync::OnceCell;
use sqlx::{PgPool, Postgres, Transaction};

use super::super::PoolConnection;

static PG_POOL: OnceCell<PgPool> = OnceCell::new();
pub struct PgConnection;

impl PoolConnection for PgConnection {
    type Pool = &'static PgPool;

    fn get() -> Self::Pool {
        PG_POOL.get().expect("PgPool is not init")
    }

    async fn init() {
        let pool = PgPool::connect(&std::env::var("DATABASE_URL").expect("Env for Pg not set"))
            .await
            .expect("Failed to set connection pool pg");
        PG_POOL.set(pool).unwrap();
    }
}

impl PgConnection {
    pub async fn get_tx() -> anyhow::Result<Transaction<'static, Postgres>> {
        PgConnection::get()
            .begin()
            .await
            .map_err(|e| anyhow::anyhow!(e))
    }
}
