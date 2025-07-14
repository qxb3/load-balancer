mod balancer;
mod server;

use std::env;

use crate::{balancer::start_load_balancer, server::start_server};

#[tokio::main]
async fn main() {
    let mut args = env::args().skip(1);

    let command = args
        .next()
        .expect("First command is required: balancer / server <host> <port>");

    match command.as_str() {
        "balancer" => start_load_balancer("127.0.0.1:4321".to_string()).await,
        "server" => {
            let host = args.next().expect("<host> is required in server");
            let port = args.next().expect("<port> is required in server");
            let address = format!("{host}:{port}");

            start_server(address).await;
        }
        _ => panic!("Unknown command"),
    }
}
