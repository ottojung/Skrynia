use super::{options::NetworkOptions, stream::Io};
use axum::serve::Listener;
use std::io;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::Semaphore;

pub struct BoundedListener {
    inner: TcpListener,
    slots: Arc<Semaphore>,
    timeout: std::time::Duration,
}
impl BoundedListener {
    pub fn new(inner: TcpListener, config: NetworkOptions) -> Self {
        Self {
            inner,
            slots: Arc::new(Semaphore::new(config.max_connections)),
            timeout: config.idle_timeout,
        }
    }
}
impl Listener for BoundedListener {
    type Io = Io;
    type Addr = SocketAddr;
    async fn accept(&mut self) -> (Self::Io, Self::Addr) {
        loop {
            let permit = self
                .slots
                .clone()
                .acquire_owned()
                .await
                .expect("listener semaphore closed");
            match super::accept::one(&self.inner, permit, self.timeout).await {
                Ok(connected) => return connected,
                Err(error) => {
                    eprintln!("Skrynia socket accept error: {error}");
                    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                }
            }
        }
    }
    fn local_addr(&self) -> io::Result<Self::Addr> {
        self.inner.local_addr()
    }
}
