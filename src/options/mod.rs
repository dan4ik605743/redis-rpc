use anyhow::{anyhow, Result};
use bb8_redis::redis::AsyncCommands;
use serde::{de::DeserializeOwned, Serialize};
use uuid::Uuid;

use tokio::{
    sync::broadcast::Sender,
    time::{self, Duration},
};

use crate::db::{redis::RedisConnection, PoolConnection};

use super::prelude::HandlerResult;

pub type OperationStatus = Result<(), String>;

pub mod tx;
pub use tx::{BrokerTx, BuildTxMessage, TxHandler, TxQueue};

// Broker
pub trait Broker: Send {
    type BrokerResponseMessage: BrokerResponseMessage;
    type BrokerRequestMessage: BrokerRequestMessage;

    const HB: Duration = Duration::from_secs(10);
    const ERROR_MESSAGE: &'static str;
    const BROKER_NAME: &'static str;

    fn hb() -> impl std::future::Future<Output = Result<()>> + Send {
        async {
            time::sleep(Self::HB).await;

            Err(anyhow!(format!(
                "{} error: timeout for operation",
                Self::BROKER_NAME
            )))
        }
    }

    fn send_response_from_service<B: Broker>(
        resp_msg: B::BrokerResponseMessage,
    ) -> impl std::future::Future<Output = Result<()>> + Send
where {
        async move {
            let mut redis_conn = RedisConnection::get().get().await?;

            redis_conn
                .publish(
                    B::get_ch().get_response_channel(),
                    &serde_json::to_string(&resp_msg)?,
                )
                .await?;

            Ok(())
        }
    }

    fn init();

    fn get_ch() -> impl BrokerChannel;
    fn get_sender() -> &'static Sender<Self::BrokerResponseMessage>;
}

pub trait BrokerResponseMessage: DeserializeOwned + Serialize + Clone + Send + 'static {
    type ResponseData: Serialize + DeserializeOwned + Clone;

    fn response<B: Broker>(self) -> HandlerResult<Option<Self::ResponseData>>;
    fn build(operation_status: OperationStatus, req_uuid: Uuid) -> Self;
}

pub trait BrokerRequestMessage: DeserializeOwned + Serialize {
    type BrokerResponseMessage: BrokerResponseMessage;

    fn check_message_for_me(&self, resp_msg: &Self::BrokerResponseMessage) -> bool;
}

pub trait BrokerChannel: Send {
    fn get_request_channel(&self) -> &'static str;
    fn get_response_channel(&self) -> &'static str;
}
