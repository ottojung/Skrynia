use super::Io;
use std::future::Future;
use std::io;
use std::task::{Context, Poll};
impl Io {
    pub(super) fn check(&mut self, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        if self.deadline.as_mut().poll(cx).is_ready() {
            return Poll::Ready(Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "socket idle timeout",
            )));
        }
        Poll::Ready(Ok(()))
    }
    pub(super) fn active(&mut self) {
        self.deadline
            .as_mut()
            .reset(tokio::time::Instant::now() + self.idle);
    }
}
