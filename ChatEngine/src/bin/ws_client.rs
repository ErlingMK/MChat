use std::env::{self, args};

use chatengine::message;

#[tokio::main]
async fn main() {
    let args = args().collect::<Vec<String>>();

    match tungstenite::connect("ws://127.0.0.1:3030/ws") {
        Ok(res) => {
            let (mut ws, _) = res;

            let first_message = "first message".as_bytes();
            let message = message::create_message(
                &message::MessageMetaData {
                    message_type: message::TEXT,
                    sender: args[1].parse().unwrap(),
                    receiver: args[2].parse().unwrap(),
                    date: "2021-01-01".to_string(),
                    length: first_message.len(),
                },
                first_message,
            );
            ws.send(tungstenite::Message::Binary(message)).unwrap();

            let second_message = "hello, this should go to the publisher".as_bytes();
            let message = message::create_message(
                &message::MessageMetaData {
                    message_type: message::TEXT,
                    sender: args[1].parse().unwrap(),
                    receiver: args[2].parse().unwrap(),
                    date: "2021-01-01".to_string(),
                    length: second_message.len(),
                },
                second_message,
            );
            ws.send(tungstenite::Message::Binary(message)).unwrap();

            while let Ok(read) = ws.read() {
                println!("{:?}", read);
            }
        }
        Err(err) => eprintln!("Error: {:?}", err),
    }
}
