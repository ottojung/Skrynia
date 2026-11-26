use super::provider_serve::serve;
use std::net::TcpListener;

#[derive(Default)]
pub(super) struct Discovery {
    pub status: u16,
    pub body: String,
    pub version: Option<String>,
    pub reply: Vec<u8>,
    pub request: Option<Vec<u8>>,
}

pub(super) struct Provider {
    pub url: String,
    thread: Option<std::thread::JoinHandle<()>>,
}

impl Provider {
    pub fn join(mut self) {
        if let Some(thread) = self.thread.take() {
            thread.join().unwrap();
        }
    }
}

pub(super) fn start(discovery: Discovery, connections: usize) -> Provider {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}/", listener.local_addr().unwrap());
    let thread = std::thread::spawn(move || {
        for stream in listener.incoming().take(connections) {
            let mut socket = stream.unwrap();
            socket
                .set_read_timeout(Some(std::time::Duration::from_secs(2)))
                .unwrap();
            serve(&mut socket, &discovery);
        }
    });
    Provider {
        url,
        thread: Some(thread),
    }
}

pub(super) const DOUBLE: &str = r#"[{"name":"double","params":["i32"],"results":["i32"]}]"#;

pub(super) fn wire(args: &[(u8, u64)]) -> Vec<u8> {
    let mut bytes = vec![1, args.len() as u8];
    for (tag, bits) in args {
        bytes.push(*tag);
        bytes.extend_from_slice(&bits.to_le_bytes());
    }
    bytes
}
