use std::io::{Read, Write};
pub fn receive<T: Read + Send + 'static>(
    pipe: Option<T>,
    max: usize,
) -> std::thread::JoinHandle<Vec<u8>> {
    std::thread::spawn(move || {
        let mut out = Vec::new();
        if let Some(pipe) = pipe {
            let _ = pipe.take((max + 1) as u64).read_to_end(&mut out);
        }
        out
    })
}
pub fn send<T: Write + Send + 'static>(
    pipe: Option<T>,
    data: &[u8],
) -> std::thread::JoinHandle<()> {
    let bytes = data.to_vec();
    std::thread::spawn(move || {
        if let Some(mut pipe) = pipe {
            let _ = pipe.write_all(&bytes);
        }
    })
}
