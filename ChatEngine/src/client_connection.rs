use std::{
    io::{Cursor, Error},
    net::SocketAddr,
    thread,
};

use bytes::{Buf, Bytes, BytesMut};
use tokio::{
    io::{AsyncReadExt, BufReader},
    net::TcpStream,
};

pub struct ClientConnection {
    socket: TcpStream,
    address: SocketAddr,
}

impl ClientConnection {
    pub fn new(socket: TcpStream, address: SocketAddr) -> ClientConnection {
        ClientConnection { socket, address }
    }
    pub async fn process_socket(&mut self) {
        let mut buf: [u8; 1] = [0; 1];

        loop {
            println!("Peeking");
            let peek = self.socket.peek(&mut buf).await.unwrap();
            if peek > 0 {
                println!("Peeked {} bytes", peek);
                read_from_socket(&mut self.socket).await;
            }
            thread::sleep(std::time::Duration::from_secs(3));
        }
    }
}

async fn read_from_socket(socket: &mut TcpStream) {
    let mut buf = BytesMut::with_capacity(4096);
    let mut reader = BufReader::new(socket);
    let mut read: usize = 0;
    loop {
        match reader.read_buf(&mut buf).await {
            Ok(n) if n > 0 => {
                read += n;
                parse_message(&mut buf[read - n..]); //
            }
            Ok(0) => {
                println!("Received 0 bytes");
                return;
            }
            Ok(_) => println!("Socket read unexpected number of bytes"),
            Err(err) => eprintln!("Error reading from socket: {}", err),
        }
    }
}

fn parse_message(buf: &[u8]) {
    println!("Buffer length {}", buf.len());
    let mut cursor = Cursor::new(&buf[..]);
    check_frame(&mut cursor).unwrap();
}

fn check_frame(src: &mut Cursor<&[u8]>) -> Result<(), Error> {
    while src.has_remaining() {
        println!("Remaining: {}", src.remaining());
        match src.get_u8() {
            0x01 => {
                let header = get_header(src)?;
                dbg!(header);
            }
            0x02 => {
                let body = get_body(src)?;
                dbg!(body);
            }
            actual => {
                eprintln!("Unknown message type");
            }
        }
    }
    Ok(())
}
fn get_body<'a>(src: &mut Cursor<&'a [u8]>) -> Result<&'a [u8], Error> {
    let start = src.position() as usize;
    let end = src.get_ref().len() - 1;

    for i in start..end {
        if src.get_ref()[i] == b'\r' && src.get_ref()[i + 1] == b'\n' {
            src.set_position((i + 2).try_into().unwrap());
            println!("Body length: {}", i - start);
            return Ok(&src.get_ref()[start..i]);
        }
    }
    Err(Error::new(
        std::io::ErrorKind::InvalidData,
        "Could not find end of header",
    ))
}

fn get_header<'a>(src: &mut Cursor<&'a [u8]>) -> Result<&'a [u8], Error> {
    let start = src.position() as usize;
    let end = src.get_ref().len() - 1;

    for i in start..end {
        if src.get_ref()[i] == b'\r' && src.get_ref()[i + 1] == b'\n' {
            src.set_position((i + 2).try_into().unwrap());
            println!("Header length: {}", i - start);
            return Ok(&src.get_ref()[start..i]);
        }
    }
    Err(Error::new(
        std::io::ErrorKind::InvalidData,
        "Could not find end of header",
    ))
}

enum Frame {
    Header {
        message_type: u8,
        id: u32,
        length: u32,
    },
    Body(Bytes),
}
