use std::{
    io::{stdin, BufReader, Read, Write},
    net::{TcpListener, TcpStream},
    process::exit,
};

use chatengine::message::{create_message, read_header, MessageMetaData, NUMBER_OF_BYTES, TEXT};
use chatengine::thread::ThreadPool;
use chrono::Utc;
use websocket::sync::Server;

fn main() {
    let server = Server::bind("127.0.0.1:7878").unwrap();
    let pool = ThreadPool::new(4);
    for connection in server.filter_map(Result::ok) {
        let client = connection.accept().unwrap();
        let (stream, _) = client.into_stream();
        let cloned = stream.try_clone().unwrap();
        pool.execute(|| read_connection(stream).unwrap());
        pool.execute(|| write_connection(cloned));
    }
}

fn write_connection(mut stream: TcpStream) {
    loop {
        let mut buf = String::new();
        stdin().read_line(&mut buf).unwrap();
        let bytes = buf.trim().as_bytes();

        let message_meta_data = MessageMetaData {
            message_type: TEXT,
            sender: 2,
            receiver: 1,
            date: Utc::now().to_rfc3339(),
            length: bytes.len(),
        };

        let message = create_message(&message_meta_data, &bytes);

        stream.write_all(&message).unwrap();
    }
}

fn read_connection(mut stream: TcpStream) -> std::io::Result<()> {
    let mut buf_reader = BufReader::new(&mut stream);
    loop {
        let meta_data = parse_type_and_length(&mut buf_reader);

        match meta_data.message_type {
            1 => {
                read_message(&mut buf_reader, meta_data.length);
            }
            2 => todo!(),
            3 => todo!(),
            _ => {
                dbg!(&meta_data);
                println!("Not a valid message signifier")
            }
        }
    }
}

fn parse_type_and_length(buf_reader: &mut BufReader<&mut TcpStream>) -> MessageMetaData {
    let mut buf: [u8; NUMBER_OF_BYTES] = [0; NUMBER_OF_BYTES];

    match buf_reader.read_exact(&mut buf) {
        Ok(_) => read_header(&buf),
        Err(err) => {
            eprintln!("Failed to read from buffer {err}");
            exit(1);
        }
    }
}

fn read_message(buf_reader: &mut BufReader<&mut TcpStream>, length: usize) {
    let mut buf = vec![0; length];
    match buf_reader.read_exact(&mut buf) {
        Ok(_) => {
            println!("Content data: {:?}", &buf);
            let message = std::str::from_utf8(&buf).unwrap();
            println!("Content as text: {message}");
        }
        Err(err) => eprintln!("unable to read exact message: {err}"),
    }
}
