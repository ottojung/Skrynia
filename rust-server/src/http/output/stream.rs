use super::{Bytes, Chunk, mpsc};

pub fn body(first: Bytes, receiver: mpsc::Receiver<Chunk>) -> axum::body::Body {
    let stream =
        futures_util::stream::unfold((Some(Ok(first)), receiver), |(first, mut rx)| async move {
            if let Some(chunk) = first {
                return Some((chunk, (None, rx)));
            }
            rx.recv().await.map(|chunk| (chunk, (None, rx)))
        });
    axum::body::Body::from_stream(stream)
}
