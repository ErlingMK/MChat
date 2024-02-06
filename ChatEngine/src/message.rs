use std::iter::repeat;

#[derive(Debug)]
pub struct MessageMetaData {
    pub message_type: u8,
    pub sender: u32,
    pub receiver: u32,
    pub date: String,
    pub length: usize,
}

#[derive(Debug)]
pub struct ChatMessage {
    pub meta_data: MessageMetaData,
    pub data: Vec<u8>,
}

pub const NUMBER_OF_BYTES: usize = 1 + 4 + 4 + 40 + 8;

pub const TEXT: u8 = 1;

pub fn create_message(meta_data: &MessageMetaData, data: &[u8]) -> Vec<u8> {
    let length = data.len();
    let mut message: Vec<u8> = Vec::with_capacity(length + 8 * NUMBER_OF_BYTES);
    message.push(meta_data.message_type);
    message.extend(meta_data.sender.to_be_bytes());
    message.extend(meta_data.receiver.to_be_bytes());
    let date_bytes = meta_data.date.as_bytes();
    message.extend(repeat(0).take(40 - date_bytes.len()));
    message.extend(date_bytes);
    message.extend(length.to_be_bytes());
    message.extend(data);

    message
}

pub fn read_header(buf: &[u8]) -> MessageMetaData {
    let header = buf;
    let mut date = header[9..=48].to_vec();
    date.retain(|&x| x != 0);
    let message_meta_data = MessageMetaData {
        message_type: header[0],
        sender: u32::from_be_bytes(header[1..=4].try_into().unwrap()),
        receiver: u32::from_be_bytes(header[5..=8].try_into().unwrap()),
        date: String::from_utf8(date.to_vec()).unwrap(),
        length: usize::from_be_bytes(header[49..=56].try_into().unwrap()),
    };
    return message_meta_data;
}

pub fn read_message(msg: &[u8]) -> ChatMessage {
    let meta_data = read_header(&msg[0..NUMBER_OF_BYTES]);
    let data = msg[NUMBER_OF_BYTES..].to_vec();
    ChatMessage { meta_data, data }
}
