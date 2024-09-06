use bb8_redis::redis::{self, aio::PubSub};

pub struct PubSubConnection;
impl PubSubConnection {
    #[allow(deprecated)]
    pub async fn get() -> PubSub {
        let redis_url = std::env::var("REDIS_URL").expect("Env for Redis not set");

        redis::Client::open(redis_url)
            .unwrap()
            .get_tokio_connection()
            .await
            .expect("Failed get connection to redis")
            .into_pubsub()
    }
}
