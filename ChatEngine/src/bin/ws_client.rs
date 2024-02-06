use std::env::args;

use chatengine::chat_message::{self, read_message};
use futures_util::{stream::SplitStream, SinkExt, StreamExt};
use tokio::{
    io::{stdin, AsyncReadExt},
    sync::mpsc::{self, UnboundedSender},
};
use tokio_tungstenite::{tungstenite::Message, WebSocketStream};

type Reader =
    SplitStream<WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>>;

#[tokio::main]
async fn main() {
    let args = args().collect::<Vec<String>>();
    let sender = args[1].parse().unwrap();
    let receiver = args[2].parse().unwrap();

    let (tx, mut rx) = mpsc::unbounded_channel();

    tokio::spawn(read_in(tx, sender, receiver));

    match tokio_tungstenite::connect_async("ws://127.0.0.1:3030/ws").await {
        Ok((ws, _)) => {
            let (mut writer, reader) = ws.split();

            // spawn a thread to listen to incoming messages
            tokio::spawn(receive_messages(reader));

            // send the inital hello message
            let message = chat_message::create_hello(sender, receiver);
            writer.send(Message::Binary(message)).await.unwrap();

            // sends messages read from stdin
            while let Some(msg) = rx.recv().await {
                writer.send(msg).await.unwrap();
            }
        }
        Err(_) => todo!(),
    }
}

async fn receive_messages(mut reader: Reader) {
    while let Some(msg) = reader.next().await {
        let msg = msg.unwrap();
        let msg = read_message(&msg.into_data());
        println!(
            "Received message: {:?}",
            msg.read_content_as_utf8().unwrap()
        );
    }
}

async fn read_in(tx: UnboundedSender<Message>, sender: u32, receiver: u32) {
    loop {
        let mut buf = vec![0; 1024];
        let n = match stdin().read(&mut buf).await {
            Err(_) | Ok(0) => break,
            Ok(n) => n,
        };

        buf.truncate(n - 1); // take only what was read and also remove the new line
        let message = chat_message::create_text_message(sender, receiver, buf);
        tx.send(Message::Binary(message)).unwrap();
    }
}
