use bb8_redis::{bb8::Pool, redis, RedisConnectionManager};
use once_cell::sync::OnceCell;

use super::super::PoolConnection;

pub type RedisPool = Pool<RedisConnectionManager>;
static REDIS_POOL: OnceCell<RedisPool> = OnceCell::new();

pub struct RedisConnection;

impl PoolConnection for RedisConnection {
    type Pool = &'static RedisPool;

    fn get() -> Self::Pool {
        REDIS_POOL.get().expect("Redis pool is not init")
    }

    async fn init() {
        let redis_url = std::env::var("REDIS_URL").expect("Env for Redis not set");
        let manager =
            RedisConnectionManager::new(redis_url.clone()).expect("URL basic checks redis failed");
        let pool = Pool::builder()
            .build(manager)
            .await
            .expect("Could not build redis connection pool");

        // Checks redis online
        redis::Client::open(redis_url)
            .unwrap()
            .get_connection()
            .expect("Failed connect to redis");

        REDIS_POOL.set(pool).unwrap();
    }
}
