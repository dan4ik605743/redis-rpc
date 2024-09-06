#[macro_export]
macro_rules! handle_response_message {
    ($channel:path, $broker_type:ty, $resp_msg_type:ty) => {
        |s_msg: &str| match serde_json::from_str::<$resp_msg_type>(&s_msg) {
            Ok(resp_msg) => {
                tokio::spawn(async move {
                    if let Err(e) = $crate::tools::send_response::<$broker_type>(resp_msg) {
                        tracing::error!(
                            "{}: send_response error: {}",
                            stringify!($broker_type),
                            e.to_string()
                        );
                    }
                });
            }
            Err(e) => {
                tracing::error!(
                    "{}: serde_parsing error: {}",
                    stringify!($broker_type),
                    e.to_string()
                );
            }
        }
    };
}

#[macro_export]
macro_rules! handle_request_message {
    ($channel:path, $broker_type:ty, $resp_msg_type:ty, $req_msg_type:ty, $status_handler:expr) => {
        |s_msg: &str| match serde_json::from_str::<$req_msg_type>(&s_msg) {
            Ok(req_msg) => {
                tokio::spawn(async move {
                    let req_uuid = req_msg.req_uuid;
                    let operation_status = $status_handler(req_msg).await;

                    type RespMsgType = $resp_msg_type;
                    let mut resp_msg = RespMsgType::build(operation_status, req_uuid);

                    if let Err(e) = <$broker_type as Broker>::send_response_from_service::<
                        $broker_type,
                    >(resp_msg)
                    .await
                    {
                        tracing::error!(
                            "{}: send_response_from_service error: {}",
                            stringify!($broker_type),
                            e.to_string()
                        );
                    }
                });
            }
            Err(e) => {
                tracing::error!(
                    "{}: serde_parsing error: {}",
                    stringify!($broker_type),
                    e.to_string()
                );
            }
        }
    };
}

// Юзается с полным составлением RespMsg в коллбеке
#[macro_export]
macro_rules! handle_request_message_with_resp_msg {
    ($channel:path, $broker_type:ty, $req_msg_type:ty, $handler:expr) => {
        |s_msg: &str| match serde_json::from_str::<$req_msg_type>(&s_msg) {
            Ok(req_msg) => {
                tokio::spawn(async move {
                    let resp_msg = $handler(req_msg).await;

                    if let Err(e) = <$broker_type as Broker>::send_response_from_service::<
                        $broker_type,
                    >(resp_msg)
                    .await
                    {
                        tracing::error!(
                            "{}: send_response_from_service error: {}",
                            stringify!($broker_type),
                            e.to_string()
                        );
                    }
                });
            }
            Err(e) => {
                tracing::error!(
                    "{}: serde_parsing error: {}",
                    stringify!($broker_type),
                    e.to_string()
                );
            }
        }
    };
}

#[macro_export]
macro_rules! subscribe_channels {
    ($pubsub:expr => $( $channel:expr ),* ) => {
        $(
            $pubsub.subscribe($channel).await?;
        )*
    };
}

// Автоматически передает s_msg в макрос который возвращает замыкание и выполняет его
#[macro_export]
macro_rules! handle_messages_chain {
    ($channel:expr, $s_msg:expr, $($channel_pattern:path => $handler:expr),+ $(,)?) => {{
        $(
            match $channel {
                $channel_pattern => {
                    let _ = $handler($s_msg);
                }
                _ => (),
            }
        )+
    }};
}

#[macro_export]
macro_rules! handle_tx {
    ($req_msg:expr, $tx_type:ty) => {{
        match ($req_msg.commit, $req_msg.rollback) {
            (true, false) => <$tx_type>::handle_tx(TxHandler::Commit, $req_msg.req_uuid).await,
            (false, true) => <$tx_type>::handle_tx(TxHandler::Rollback, $req_msg.req_uuid).await,
            _ => Err("Requests args are not set correctly".to_string()),
        }
    }};
}

// Не передает в замыкание ничего
// #[macro_export]
// macro_rules! handle_messages_chain {
//     ($channel:expr, $($channel_pattern:path => $handler:expr),+ $(,)?) => {{
//         match $channel {
//             $($channel_pattern => $handler),+
//             _ => (),
//         }
//     }};
// }
