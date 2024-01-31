use std::{io::Read, thread};

use rand::{thread_rng, Rng};
use tokio::{io::AsyncWriteExt, net::TcpStream};

#[tokio::main]
async fn main() {
    let mut stream = TcpStream::connect("127.0.0.1:6060").await.unwrap();

    loop {
        let mut introduction = create_header();
        let body = create_body();
        introduction.extend(body);

        stream.write_all(&introduction).await.unwrap();
        thread::sleep(std::time::Duration::from_secs(5));
    }
}

fn create_body() -> Vec<u8> {
    let mut buf = Vec::new();
    buf.push(0x02);
    buf.extend_from_slice(b"Content of body");
    buf.extend_from_slice(b"\r\n");
    buf
}

fn create_header() -> Vec<u8> {
    let mut buf = Vec::new();
    buf.push(0x01);
    buf.extend_from_slice(b"Content of header");
    buf.extend(create_random_u8_array(20));
    buf.extend_from_slice(b"\r\n");
    buf
}

fn create_random_u8_array(size: usize) -> Vec<u8> {
    let mut rng = thread_rng();
    let mut buffer = vec![0u8; size];
    rng.fill(&mut buffer[..]);
    buffer
}
