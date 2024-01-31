use chatengine::start_server;

#[tokio::main]
async fn main() {
    if let Some(address) = std::env::var("ADDR").ok() {
        start_server(address).await;
    } else {
        eprintln!("ADDR environment variable not set")
    }
}