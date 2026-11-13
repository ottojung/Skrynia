use super::stream::Io;
use std::io;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::sync::OwnedSemaphorePermit;

pub async fn one(
    listener: &TcpListener,
    permit: OwnedSemaphorePermit,
    timeout: Duration,
) -> io::Result<(Io, SocketAddr)> {
    let (socket, address) = listener.accept().await?;
    socket.set_nodelay(true)?;
    Ok((Io::new(socket, permit, timeout), address))
}
