mod idle;
mod read;
mod write;
use std::pin::Pin;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::sync::OwnedSemaphorePermit;
use tokio::time::Sleep;

pub struct Io {
    pub(super) inner: TcpStream,
    pub(super) deadline: Pin<Box<Sleep>>,
    pub(super) idle: Duration,
    _permit: OwnedSemaphorePermit,
}
impl Io {
    pub fn new(inner: TcpStream, permit: OwnedSemaphorePermit, idle: Duration) -> Self {
        Self {
            inner,
            deadline: Box::pin(tokio::time::sleep(idle)),
            idle,
            _permit: permit,
        }
    }
}
