use crate::client_connection::ClientConnection;
use tokio::net::TcpListener;

pub mod message;
pub mod thread;

mod client_connection;

pub async fn start_server(address: String) {
    let mut clients: Vec<ClientConnection> = Vec::new();

    println!("Starting server at {}", address);
    let tcp_listener = TcpListener::bind(address).await.unwrap();
    loop {
        if let Ok((socket, addr)) = tcp_listener.accept().await {
            println!("New client connection: {}", addr);
            let mut client = ClientConnection::new(socket, addr);
            client.process_socket().await;
            clients.push(client);
        } else {
            eprintln!("Error accepting socket")
        }
    }
}
