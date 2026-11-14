use std::{
    io::{self, Read, Write},
    net::TcpStream,
    time::Duration,
};
pub fn run() -> io::Result<()> {
    let public =
        std::env::var("SKRYNIA_URL").map_err(|_| io::Error::other("SKRYNIA_URL required"))?;
    let url = url::Url::parse(&public).map_err(|_| io::Error::other("invalid SKRYNIA_URL"))?;
    let port = std::env::var("SKRYNIA_PORT").unwrap_or_else(|_| "17380".into());
    let mut socket = TcpStream::connect(format!("127.0.0.1:{port}"))?;
    socket.set_read_timeout(Some(Duration::from_secs(3)))?;
    socket.set_write_timeout(Some(Duration::from_secs(3)))?;
    let path = format!("{}/health", url.path().trim_end_matches('/'));
    socket.write_all(
        format!("GET {path} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n").as_bytes(),
    )?;
    let mut header = [0u8; 128];
    let n = socket.read(&mut header)?;
    if header[..n].starts_with(b"HTTP/1.1 200 ") {
        Ok(())
    } else {
        Err(io::Error::other("Skrynia health endpoint unavailable"))
    }
}
