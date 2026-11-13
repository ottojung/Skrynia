use super::Io;
use std::io;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, ReadBuf};
impl AsyncRead for Io {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        if let Poll::Ready(Err(error)) = self.check(cx) {
            return Poll::Ready(Err(error));
        }
        let before = buf.filled().len();
        let outcome = Pin::new(&mut self.inner).poll_read(cx, buf);
        if matches!(&outcome, Poll::Ready(Ok(()))) && buf.filled().len() > before {
            self.active();
        }
        outcome
    }
}
