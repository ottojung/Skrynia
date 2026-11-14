mod dns;
mod http;
mod signature;
mod transport;
use crate::{PushConfig, Subscription, Vapid};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Failure {
    Gone,
    Blocked,
    Retry,
}
pub async fn send(sub: &Subscription, config: &PushConfig, vapid: &Vapid) -> Result<(), Failure> {
    let message = signature::create(sub, config, vapid)?;
    transport::deliver(sub, config, message).await
}
