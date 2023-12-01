use futures_util::{SinkExt, StreamExt};
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc, RwLock};
use warp::{
    ws::{Message, Ws},
    Reply,
};

type Clients = Arc<RwLock<HashMap<usize, mpsc::UnboundedSender<Message>>>>;
static NEXT_USER_ID: AtomicUsize = AtomicUsize::new(1);

#[derive(Clone, Debug)]
struct BroadcastMessage {
    user_id: usize,
    message: Message,
}

lazy_static! {
    static ref CLIENTS: Clients = {
        return Clients::default();
    };
    static ref BROADCAST_CHANNEL: (
        broadcast::Sender<BroadcastMessage>,
        broadcast::Receiver<BroadcastMessage>
    ) = {
        return broadcast::channel(32);
    };
}

// TODO: swap the broadcast channel out for a Hash of mpsc senders (like the warp example has)
pub fn handle(socket: Ws) -> impl Reply {
    return socket.on_upgrade(|websocket| async move {
        let my_id = NEXT_USER_ID.fetch_add(1, Ordering::Relaxed);

        let (mut ws_client, mut ws_stream) = websocket.split();

        // make mpsc unbounded channel
        // let unbounded_channel = mpsc::unbounded_channel();

        //

        let mut broadcast_rx = BROADCAST_CHANNEL.0.subscribe();

        tokio::spawn(async move {
            while let Ok(broadcast) = broadcast_rx.recv().await {
                if broadcast.user_id != my_id {
                    ws_client.send(broadcast.message).await;
                }
            }
        });

        // websocket client has sent message down. Broadcast to everyone
        while let Some(message) = ws_stream.next().await {
            let msg = BroadcastMessage {
                message: message.unwrap(),
                user_id: my_id,
            };
            BROADCAST_CHANNEL.0.send(msg).unwrap();
        }
    });
}
