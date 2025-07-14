use std::{
    ops::{AddAssign, SubAssign},
    sync::Arc,
};

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
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

    let mut request = Vec::new();
    loop {
        let mut buf = [0; 1024];
        match stream.read(&mut buf).await {
            Ok(0) => break,
            Ok(n) => {
                request.extend_from_slice(&buf[..n]);
                if buf[..n].ends_with(b"\0") {
                    request.pop(); // Removes the null terminator.
                    break;
                }
            }
            Err(_) => break,
        }
    }

    let target_number = String::from_utf8(request).unwrap().parse::<u64>().unwrap();

    let result = fib(target_number);

    stream
        .write_all(format!("{result}").as_bytes())
        .await
        .unwrap();

    println!("Done Calculation: {}", load.lock().await);

    load.lock().await.sub_assign(1);
}

// Calculate fibonacci.
fn fib(n: u64) -> u64 {
    if n <= 1 { n } else { fib(n - 1) + fib(n - 2) }
}
