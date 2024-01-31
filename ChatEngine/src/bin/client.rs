use std::{
    io::{stdin, BufReader, Read, Write},
    net::TcpStream,
    process::exit,
    thread::{self, JoinHandle},
};

const TEXT: u8 = 1;
const PICTURE: u8 = 2;
const AUDIO: u8 = 3;

fn main() {
    match TcpStream::connect("127.0.0.1:7878") {
        Ok(stream) => {
            let cloned = stream.try_clone().unwrap();

            let read_handle = thread::spawn(|| read_connection(cloned));
            let write_handle = thread::spawn(|| write_connection(stream));
            write_handle.join().unwrap();
            read_handle.join().unwrap().unwrap();
        }
        Err(err) => eprintln!("Unable to connect to socket {}", err),
    }
}

fn create_message(message_metadata: MessageMetaData, data: &[u8]) -> Vec<u8> {
    let mut message: Vec<u8> = Vec::with_capacity(message_metadata.length);
    message.push(message_metadata.message_type);
    message.extend(message_metadata.length.to_be_bytes());
    message.extend(data);

    message
}

fn write_connection(mut stream: TcpStream) {
    loop {
        let mut buf = String::new();
        stdin().read_line(&mut buf).unwrap();
        let bytes = buf.trim().as_bytes();

        let message = create_message(
            MessageMetaData {
                length: bytes.len(),
                message_type: TEXT,
            },
            &bytes,
        );

        stream.write_all(&message).unwrap();
        stream.flush().unwrap();
    }
}

fn read_connection(mut stream: TcpStream) -> std::io::Result<()> {
    let mut buf_reader = BufReader::new(&mut stream);
    read(&mut buf_reader)
}

fn read(buf_reader: &mut BufReader<&mut TcpStream>) -> std::io::Result<()> {
    loop {
        let meta_data = parse_type_and_length(buf_reader);
        match meta_data.message_type {
            1 => {
                read_message(buf_reader, meta_data.length);
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
    let mut buf: [u8; 9] = [0; 9];

    match buf_reader.read_exact(&mut buf) {
        Ok(_) => {
            let mut length: [u8; 8] = [0; 8];
            length.copy_from_slice(&buf[1..=8]);
            let length = usize::from_be_bytes(length);

            MessageMetaData {
                message_type: buf[0],
                length,
            }
        }
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
            let message = std::str::from_utf8(&buf).unwrap();
            println!("Other: {message}");
        }
        Err(err) => eprintln!("unable to read exact message: {err}"),
    }
}

#[derive(Debug)]
struct MessageMetaData {
    message_type: u8,
    length: usize,
}

use std::sync::{
    mpsc::{self, Receiver},
    Arc, Mutex,
};

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}

struct Worker {
    id: usize,
    thread: Option<JoinHandle<()>>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

pub struct PoolCreationError {}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let message = receiver.lock().unwrap().recv();

            match message {
                Ok(job) => {
                    println!("Worker id {id} got a job; executing.");
                    job();
                }
                Err(_) => {
                    println!("Worker {id} got disconnected; shutting down.");
                    break;
                }
            }
        });
        Worker {
            id,
            thread: Some(thread),
        }
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());

        for worker in &mut self.workers {
            print!("Shutting down worker {}", worker.id);
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

impl ThreadPool {
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel();

        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);
        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)))
        }

        ThreadPool {
            workers,
            sender: Some(sender),
        }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        self.sender.as_ref().unwrap().send(job).unwrap();
    }
}
