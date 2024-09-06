use std::collections::VecDeque;

use anyhow::Context;
use tokio::sync::RwLock;
use uuid::Uuid;

use super::Broker;
use crate::db::pgsql::Tx;

pub type TxQueue = RwLock<VecDeque<(Uuid, Tx)>>;

pub enum TxHandler {
    Commit,
    Rollback,
}

pub trait BuildTxMessage {
    fn build_commit_msg(from_service: String, req_uuid: Uuid) -> Self;
    fn build_rollback_msg(from_service: String, req_uuid: Uuid) -> Self;
}

pub trait BrokerTx: Broker<BrokerRequestMessage: BuildTxMessage + Default> {
    fn init_tx_queue();
    fn get_tx_queue() -> &'static TxQueue;

    fn handle_tx(
        handler: TxHandler,
        req_uuid: Uuid,
    ) -> impl std::future::Future<Output = Result<(), String>> + Send {
        async move {
            let tx_queue = Self::get_tx_queue();

            loop {
                let ro_tx_queue = tx_queue.read().await;

                if let Some(pos) = ro_tx_queue.iter().position(|(uuid, _)| *uuid == req_uuid) {
                    if pos == 0 {
                        break;
                    }

                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                } else {
                    return Err("Not found uuid in queue".to_string());
                }
            }
            let mut wr_tx_queue = tx_queue.write().await;
            let (_, tx) = wr_tx_queue
                .pop_front()
                .context("Failed del checked value")
                .map_err(|e| e.to_string())?;

            match handler {
                TxHandler::Commit => tx.commit().await.map_err(|e| e.to_string()),
                TxHandler::Rollback => tx.rollback().await.map_err(|e| e.to_string()),
            }
        }
    }
}
