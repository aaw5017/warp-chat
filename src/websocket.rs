use crate::Clients;
use futures_util::{FutureExt, StreamExt};
use warp::{ws::Ws, Reply};

pub fn handle(socket: Ws, clients: Clients) -> impl Reply {
    return socket.on_upgrade(|websocket| {
        let (tx, rx) = websocket.split();

        return rx.forward(tx).map(|reply| {
            if let Err(e) = reply {
                eprintln!("Websocket error: {}", e);
            }
        });
    });
}
