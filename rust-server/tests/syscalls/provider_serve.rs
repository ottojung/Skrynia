use super::provider::Discovery;
use std::io::{BufRead, BufReader, Read, Write};

pub(super) fn serve(socket: &mut std::net::TcpStream, discovery: &Discovery) {
    let mut reader = BufReader::new(&mut *socket);
    let mut line = String::new();
    reader.read_line(&mut line).unwrap();
    let method = line.split_whitespace().next().unwrap_or("").to_owned();
    let path = line.split_whitespace().nth(1).unwrap_or("").to_owned();
    let mut headers = Vec::new();
    loop {
        line.clear();
        if reader.read_line(&mut line).unwrap() == 0 || line == "\r\n" {
            break;
        }
        headers.push(line.to_ascii_lowercase());
    }
    let (status, body, kind) = if path == "/syscalls_list" && method == "GET" {
        assert!(
            headers
                .iter()
                .all(|h| !h.starts_with("content-length:") || h.contains(": 0"))
        );
        (
            discovery.status,
            discovery.body.as_bytes(),
            "application/json",
        )
    } else {
        assert_eq!(method, "POST");
        let mut bytes = [0u8; 4];
        reader.read_exact(&mut bytes).unwrap();
        assert_eq!(bytes, u32::MAX.to_le_bytes());
        (200, discovery.reply.as_slice(), "application/octet-stream")
    };
    drop(reader);
    write!(
        socket,
        "HTTP/1.1 {status} OK\r\nContent-Type: {kind}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    )
    .unwrap();
    socket.write_all(body).unwrap();
}
