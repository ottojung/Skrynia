use super::{Failure, dns};
use crate::{PushConfig, Subscription};
use std::time::Duration;
pub async fn deliver(
    sub: &Subscription,
    config: &PushConfig,
    msg: web_push::WebPushMessage,
) -> Result<(), Failure> {
    let (host, address) = dns::choose(&sub.endpoint).await?;
    let client = reqwest::Client::builder()
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .resolve(&host, address)
        .timeout(Duration::from_millis(config.timeout_ms))
        .build()
        .map_err(|_| Failure::Retry)?;
    super::http::post(&client, sub, msg).await
}
