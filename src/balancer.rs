use std::{collections::HashMap, sync::Arc};

use tokio::{net::{TcpListener, TcpStream}, sync::Mutex};

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
        data_ref.insert("127.0.0.1:9001".to_string(), 0);
        data_ref.insert("127.0.0.1:9002".to_string(), 0);
    }

    loop {
        let (stream, _) = listener.accept()
            .await
            .expect("Failed to receive incoming request");

        let data = Arc::clone(&data);
        tokio::spawn(async move {
            balance(stream, data).await;
        });
    }
}

async fn balance(mut client_stream: TcpStream, data: LBData) {
    let mut data_ref = data.lock().await;
    let (least_load_addr, load) = data_ref
        .iter_mut()
        .min_by_key(|(_, v)| **v)
        .expect("Failed to find the backend with the least load");

    *load += 1;

    let mut backend_stream = TcpStream::connect(&least_load_addr)
        .await
        .expect("Failed to bind to backend");

    let (mut csr, mut csw) = client_stream.split();
    let (mut bsr, mut bsw) = backend_stream.split();

    let forward_request = tokio::io::copy(&mut csr, &mut bsw);
    let backward_request = tokio::io::copy(&mut bsr, &mut csw);

    tokio::try_join!(forward_request, backward_request)
        .expect("Failed to join forward & backward request");
}
