mod headers;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio_rustls::TlsAcceptor;
pub async fn respond(listener: TcpListener, acceptor: TlsAcceptor, status: u16) {
    let (stream, _) = listener.accept().await.unwrap();
    let mut stream = acceptor.accept(stream).await.unwrap();
    let mut content = Vec::new();
    let (end, length) = loop {
        let mut part = [0u8; 8192];
        let count = stream.read(&mut part).await.unwrap();
        assert!(count > 0, "client disconnected before encrypted Push");
        content.extend_from_slice(&part[..count]);
        if let Some((end, length)) = headers::parse(&content) {
            if content.len() >= end + 4 + length {
                break (end, length);
            }
        }
    };
    let headers = String::from_utf8_lossy(&content[..end]).to_ascii_lowercase();
    assert!(headers.starts_with("post /notification http/1.1"));
    assert!(headers.contains("authorization:"));
    assert!(headers.contains("content-encoding: aes128gcm"));
    assert!(length > 32);
    assert!(!content[end + 4..].windows(7).any(|s| s == b"changed"));
    let reply = format!(
        "HTTP/1.1 {status} Test\r\nContent-Length: 0\r\nLocation: http://127.0.0.1/fake\r\nConnection: close\r\n\r\n"
    );
    stream.write_all(reply.as_bytes()).await.unwrap();
}
