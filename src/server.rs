use std::{ops::{AddAssign, SubAssign}, sync::Arc, thread, time::Duration};

use tokio::{
    io::AsyncWriteExt,
    net::{TcpListener, TcpStream},
    sync::Mutex,
};

pub async fn start_server(address: String) {
    let listener = TcpListener::bind(&address)
        .await
        .expect("Failed to bind to specified host");

    println!("Server listening at: {address}");

    let load = Arc::new(Mutex::new(0));

    loop {
        let (stream, _) = listener
            .accept()
            .await
            .expect("Failed to receive incoming request");

        let load = Arc::clone(&load);
        tokio::spawn(async move {
            handle_request(stream, load).await;
        });
    }
}

async fn handle_request(mut stream: TcpStream, load: Arc<Mutex<i32>>) {
    load.lock().await.add_assign(1);

    stream
        .write_all(b"Started processing\n")
        .await
        .expect("Failed to send start confirmation");

    // Simulates some heavy computation.
    let random_sleep = rand::random_range(0..=1);
    thread::sleep(Duration::from_secs(random_sleep));

    stream
        .write_all(b"Done processing\n")
        .await
        .expect("Failed to send done confirmation");

    {
        let mut load_ref = load.lock().await;
        println!("Processed: {load_ref}");

        *load_ref -= 1;
    }
}
