use std::{collections::HashMap, sync::Arc, u32};

use crate::message::*;
use tokio::sync::{
    mpsc::{self, Sender},
    RwLock,
};

pub struct MessageManager {
    pub publisher: mpsc::Sender<ChatMessage>,
}

impl MessageManager {
    pub fn new(
        capacity: usize,
        connections: Arc<RwLock<HashMap<u32, Sender<ChatMessage>>>>,
    ) -> MessageManager {
        let (tx, rx) = mpsc::channel::<ChatMessage>(capacity);
        tokio::spawn(async move { receive_messages(rx, connections).await });
        MessageManager { publisher: tx }
    }
}

async fn receive_messages(
    mut rx: mpsc::Receiver<ChatMessage>,
    subscribers: Arc<RwLock<HashMap<u32, Sender<ChatMessage>>>>,
) {
    while let Some(msg) = rx.recv().await {
        println!("Received message in publisher: {:?}", msg);
        let read = subscribers.read().await;
        if read.contains_key(&msg.receiver) {
            let sender = read.get(&msg.receiver).unwrap();
            sender.send(msg).await.unwrap();
        }
    }
}
