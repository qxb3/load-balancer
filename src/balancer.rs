use std::{collections::HashMap, sync::Arc};

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::Mutex,
};

type LBData = Arc<Mutex<HashMap<String, u32>>>;

pub async fn start_load_balancer(address: String) {
    let listener = TcpListener::bind(&address)
        .await
        .expect("Failed to bind load balancer");

    println!("Load balancer listening at: {address}");

    let data = Arc::new(Mutex::new(HashMap::new()));

    // Inside a scope for the lock to be freed.
    {
        let mut data_ref = data.lock().await;
        for i in 1..=8 {
            data_ref.insert(format!("127.0.0.1:900{i}"), 0);
        }
    }

    loop {
        let (stream, _) = listener.accept().await.unwrap();

        let data = Arc::clone(&data);

        tokio::spawn(async move {
            balance(stream, data).await;
        });
    }
}

async fn balance(client_stream: TcpStream, data: LBData) {
    let least_load_address = get_least_load_address(Arc::clone(&data)).await;
    let backend_stream = TcpStream::connect(&least_load_address).await.unwrap();

    let (mut csr, mut csw) = client_stream.into_split();
    let (mut bsr, mut bsw) = backend_stream.into_split();

    // client -> backend.
    let cb_handle = tokio::spawn(async move {
        loop {
            let mut request = [0; 1024];
            match csr.read(&mut request).await {
                Ok(0) => break,
                Ok(n) => bsw.write_all(&request[..n]).await.unwrap(),
                Err(_) => break,
            }
        }
    });

    // backend -> client.
    let bc_handle = tokio::spawn(async move {
        loop {
            let mut response = [0; 1024];
            match bsr.read(&mut response).await {
                Ok(0) => break,
                Ok(n) => csw.write_all(&response[..n]).await.unwrap(),
                Err(_) => break,
            }
        }
    });

    let _ = tokio::join!(cb_handle, bc_handle);

    let mut data_ref = data.lock().await;
    let load = data_ref.get_mut(&least_load_address).unwrap();
    *load -= 1;
}

async fn get_least_load_address(data: LBData) -> String {
    let mut data_ref = data.lock().await;
    let (address, load) = data_ref
        .iter_mut()
        .min_by_key(|(_, v)| **v)
        .expect("Failed to find the backend with the least load");

    *load += 1;

    address.to_string()
}
