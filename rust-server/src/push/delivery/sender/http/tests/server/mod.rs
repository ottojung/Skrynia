mod receive;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tokio::task::JoinHandle;
pub async fn start(status: u16) -> (SocketAddr, reqwest::Certificate, JoinHandle<()>) {
    let (acceptor, root) = super::tls::bundle();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let thread = tokio::spawn(async move { receive::respond(listener, acceptor, status).await });
    (address, root, thread)
}
