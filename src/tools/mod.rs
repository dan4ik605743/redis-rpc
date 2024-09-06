use actix_web::http::StatusCode;

use super::{
    options::{Broker, BrokerChannel, BrokerRequestMessage, BrokerResponseMessage},
    prelude::*,
};

mod handlers;

pub async fn send_request<B>(
    req_msg: &B::BrokerRequestMessage,
) -> HandlerResult<Option<<B::BrokerResponseMessage as BrokerResponseMessage>::ResponseData>>
where
    B: Broker,
    B::BrokerRequestMessage: BrokerRequestMessage<BrokerResponseMessage = B::BrokerResponseMessage>,
    B::BrokerResponseMessage: BrokerResponseMessage,
{
    let fut = async {
        let mut conn = RedisConnection::get().get().await?;

        conn.publish(
            B::get_ch().get_request_channel(),
            serde_json::to_string(req_msg)?,
        )
        .await?;

        let mut rx = B::get_sender().subscribe();

        loop {
            match rx.recv().await {
                Ok(resp_msg) if req_msg.check_message_for_me(&resp_msg) => {
                    return resp_msg.response::<B>();
                }
                Ok(_) => (),
                Err(e) => {
                    return HandlerError::send_err(
                        e.to_string(),
                        StatusCode::INTERNAL_SERVER_ERROR,
                    );
                }
            }
        }
    };

    tokio::select! {
        res = fut => {
            match res {
                Ok(data) => Ok(data),
                Err(e) => {
                    HandlerError::send_err(e.err.to_string(), e.status_code)
                }
            }
        }

        Err(e) = B::hb() => HandlerError::send_err(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub fn send_response<B: Broker>(resp_msg: B::BrokerResponseMessage) -> Result<()> {
    let sender = B::get_sender();

    if sender.receiver_count() == 0 {
        // tracing::warn!("Not found subscribes");
        return Ok(());
    }

    sender.send(resp_msg).map_err(|e| anyhow!(e.to_string()))?;
    Ok(())
}
