use std::{collections::HashMap, sync::Arc};

use tokio::{
    io::AsyncReadExt, net::{TcpListener, TcpStream}, sync::Mutex
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
        data_ref.insert("127.0.0.1:9001".to_string(), 0);
        data_ref.insert("127.0.0.1:9002".to_string(), 0);
        data_ref.insert("127.0.0.1:9003".to_string(), 0);
        data_ref.insert("127.0.0.1:9004".to_string(), 0);
        data_ref.insert("127.0.0.1:9005".to_string(), 0);
        data_ref.insert("127.0.0.1:9006".to_string(), 0);
        data_ref.insert("127.0.0.1:9007".to_string(), 0);
        data_ref.insert("127.0.0.1:9008".to_string(), 0);
    }

    loop {
        let (stream, _) = listener
            .accept()
            .await
            .expect("Failed to receive incoming request");

        let data = Arc::clone(&data);
        tokio::spawn(async move {
            balance(stream, data).await;
        });
    }
}

async fn balance(mut client_stream: TcpStream, data: LBData) {
    let least_load_address = get_least_load_address(data).await;

    let mut backend_stream = TcpStream::connect(least_load_address)
        .await
        .expect("Failed to bind to backend");

    let (mut csr, mut csw) = client_stream.split();
    let (mut bsr, mut bsw) = backend_stream.split();

    let forward_handle = tokio::io::copy(&mut csr, &mut bsw);
    let backward_handle = tokio::io::copy(&mut bsr, &mut csw);

    tokio::try_join!(forward_handle, backward_handle)
        .expect("Failed to join handles");
}

async fn get_least_load_address(data: LBData) -> String {
    let mut data_ref = data.lock().await;
    let (address, _) = data_ref
        .iter_mut()
        .min_by_key(|(_, v)| **v)
        .expect("Failed to find the backend with the least load");

    address.to_string()
}
