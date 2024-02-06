use std::{collections::HashMap, sync::Arc};

use chatengine::{
    message::{self, create_message, ChatMessage, NUMBER_OF_BYTES},
    publisher::{self, MessageManager},
};
use futures_util::{FutureExt, SinkExt, StreamExt};
use tokio::sync::{
    mpsc::{self, Sender},
    RwLock,
};
use warp::{
    filters::ws::{self, WebSocket},
    Filter,
};

type Connections = Arc<RwLock<HashMap<u32, Sender<ChatMessage>>>>;
type Publisher = Sender<ChatMessage>;

#[tokio::main]
async fn main() {
    let connections = Connections::default();

    let message_manager = MessageManager::new(10, connections.clone());

    let connections = warp::any().map(move || connections.clone());

    let publisher = warp::any().map(move || message_manager.publisher.clone());

    let websocket = warp::path("ws")
        .and(warp::ws())
        .and(connections)
        .and(publisher)
        .map(|ws: warp::ws::Ws, connections, publisher| {
            ws.on_upgrade(move |socket| connected(socket, connections, publisher))
        });

    warp::serve(websocket).run(([127, 0, 0, 1], 3030)).await;
}

async fn connected(socket: WebSocket, connections: Connections, publisher: Publisher) {
    println!("Connected");

    let (mut tx_socket, mut rx_socket) = socket.split();

    if let Some(Ok(msg)) = rx_socket.next().await {
        // wait for first message to identify sender
        println!("First message {:?}", msg);

        // parse header and create new connection for the parsed id
        let bytes = msg.as_bytes();
        let chat_message = message::read_message(bytes);
        let (tx, mut rx) = mpsc::channel(10);
        connections
            .write()
            .await
            .insert(chat_message.meta_data.sender, tx);

        tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                // wait for message from others
                println!("Received message from publisher: {:?}", msg);
                let received_message = message::create_message(&msg.meta_data, &msg.data);
                tx_socket
                    .send(ws::Message::binary(received_message))
                    .await
                    .unwrap();
            }
        });

        tokio::spawn(async move {
            while let Some(Ok(msg)) = rx_socket.next().await {
                // wait for more messages from client
                println!("Next messages {:?}", msg);
                let chat_message = message::read_message(msg.as_bytes());
                println!("Received message: {:?}", chat_message);

                match publisher.send(chat_message).await {
                    Ok(msg) => println!("Sent message: {:?}", msg),
                    Err(err) => eprintln!("Error: {:?}", err),
                }
            }
        });
    } else {
        eprintln!("Invalid message");
        return;
    }
}
