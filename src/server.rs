use std::{
    io::Write,
    net::{TcpListener, TcpStream},
    thread,
    time::Duration,
};

pub async fn start_server(address: String) {
    let listener = TcpListener::bind(address.to_string())
        .expect("Failed to bind to specified host");

    println!("Server listening at: {address}");

    for stream in listener.incoming() {
        if let Ok(stream) = stream {
            handle_request(stream, &address).await;
        }
    }
}

async fn handle_request(mut stream: TcpStream, address: &String) {
    println!("Request receive");

    // Simulates some heavy computation.
    thread::sleep(Duration::from_secs(1));

    stream
        .write_all(format!("{address} response\n").as_bytes())
        .expect(&format!("{address} Failed to respond at"));
}
