use super::{Bytes, Chunk, mpsc};

const BATCH: usize = 128;

pub fn body(first: Bytes, receiver: mpsc::Receiver<Chunk>) -> axum::body::Body {
    let stream = futures_util::stream::unfold(
        (Some(first), Some(receiver), None),
        |(first, receiver, pending)| async move {
            let mut rx = receiver?;
            if let Some(bytes) = first {
                return Some((Ok(bytes), (None, Some(rx), None)));
            }
            let next = match pending {
                Some(e) => Err(e),
                None => rx.recv().await?,
            };
            match next {
                Ok(bytes) => {
                    let (chunk, error) = batch(bytes, &mut rx);
                    Some((Ok(chunk), (None, Some(rx), error)))
                }
                Err(e) => Some((Err(e), (None, None, None))),
            }
        },
    );
    axum::body::Body::from_stream(stream)
}

fn batch(first: Bytes, rx: &mut mpsc::Receiver<Chunk>) -> (Bytes, Option<std::io::Error>) {
    let mut joined: Option<Vec<u8>> = None;
    let mut error = None;
    for _ in 1..BATCH {
        match rx.try_recv() {
            Ok(Ok(bytes)) => {
                let out = joined.get_or_insert_with(|| first.to_vec());
                out.extend_from_slice(&bytes);
            }
            Ok(Err(e)) => {
                error = Some(e);
                break;
            }
            Err(_) => break,
        }
    }
    (joined.map_or(first, Bytes::from), error)
}
