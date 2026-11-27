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
    let len = headers
        .iter()
        .find_map(|h| h.strip_prefix("content-length:"))
        .and_then(|v| v.trim().parse::<usize>().ok())
        .unwrap_or(0);
    let mut request = vec![0u8; len];
    reader.read_exact(&mut request).unwrap();
    let (status, body, kind): (u16, &[u8], &str) = if path == "/syscalls_list" && method == "GET" {
        (
            discovery.status,
            discovery.body.as_bytes(),
            "application/json",
        )
    } else if path == "/version" && method == "POST" {
        match discovery.version.as_deref() {
            Some(value) => (200, value.as_bytes(), "application/json"),
            None => (404, &[], "text/plain"),
        }
    } else {
        assert_eq!(method, "POST");
        if let Some(expected) = &discovery.request {
            assert_eq!(&request, expected);
        }
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
