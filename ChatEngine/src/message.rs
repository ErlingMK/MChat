use std::iter::repeat;

pub const NUMBER_OF_BYTES: usize = 1 + 4 + 4 + 40 + 8;

#[derive(Debug)]
pub struct ChatMessage {
    pub kind: MessageKind,
    pub sender: u32,
    pub receiver: u32,
    pub creation_date: String,
    pub data: Vec<u8>,
}

impl ChatMessage {
    pub fn read_content_as_utf8(&self) -> Option<String> {
        match self.kind {
            MessageKind::Text => Some(String::from_utf8(self.data.clone()).unwrap()),
            _ => None,
        }
    }

    pub fn create_text_message(sender: u32, receiver: u32, data: Vec<u8>) -> Vec<u8> {
        let message_meta_data = ChatMessage {
            kind: MessageKind::Text,
            sender,
            receiver,
            creation_date: chrono::Utc::now().to_rfc3339(),
            data,
        };
        ChatMessage::message_as_bytes(message_meta_data)
    }

    pub fn create_hello(sender: u32, receiver: u32) -> Vec<u8> {
        let message_meta_data = ChatMessage {
            kind: MessageKind::Greet,
            sender,
            receiver,
            creation_date: chrono::Utc::now().to_rfc3339(),
            data: vec![0; 0],
        };
        ChatMessage::message_as_bytes(message_meta_data)
    }

    pub fn message_as_bytes(chat_message: ChatMessage) -> Vec<u8> {
        let length = chat_message.data.len();
        let mut message: Vec<u8> = Vec::with_capacity(length + 8 * NUMBER_OF_BYTES);
        message.push(chat_message.kind.as_bytes());
        message.extend(chat_message.sender.to_be_bytes());
        message.extend(chat_message.receiver.to_be_bytes());
        let date_bytes = chat_message.creation_date.as_bytes();
        message.extend(repeat(0).take(40 - date_bytes.len()));
        message.extend(date_bytes);
        message.extend(length.to_be_bytes());
        message.extend(chat_message.data);

        message
    }

    pub fn read_message(buf: &[u8]) -> ChatMessage {
        let header = buf;
        let mut date = header[9..=48].to_vec();
        date.retain(|&x| x != 0);

        let content_length = usize::from_be_bytes(header[49..=56].try_into().unwrap());

        let message = ChatMessage {
            kind: MessageKind::parse_to_message_type(header[0]),
            sender: u32::from_be_bytes(header[1..=4].try_into().unwrap()),
            receiver: u32::from_be_bytes(header[5..=8].try_into().unwrap()),
            creation_date: String::from_utf8(date.to_vec()).unwrap(),
            data: buf[NUMBER_OF_BYTES..].to_vec(),
        };
        return message;
    }
}

#[derive(Debug)]
pub enum MessageKind {
    Greet,
    Text,
    Image,
}

impl MessageKind {
    pub fn as_bytes(&self) -> u8 {
        match self {
            MessageKind::Greet => 0,
            MessageKind::Text => 1,
            MessageKind::Image => 2,
        }
    }

    pub fn parse_to_message_type(message_type: u8) -> MessageKind {
        match message_type {
            0 => MessageKind::Greet,
            1 => MessageKind::Text,
            2 => MessageKind::Image,
            _ => panic!("Unknown message type"),
        }
    }
}
