mod connection;
use tokio::net::TcpListener;


pub async fn start_server(port: u16) {
    let url = format!("127.0.0.1:{}", port);
    let listener = TcpListener::bind(&url).await.unwrap(); // panic is needed
    tokio::spawn(connection::handle_messages());
    while let Ok((socket, _)) = listener.accept().await {
        tokio::spawn(connection::connect_to_server(socket));
        
    }
}

