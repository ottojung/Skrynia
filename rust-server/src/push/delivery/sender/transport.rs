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
    let request = web_push::request_builder::build_request::<Vec<u8>>(msg);
    let (parts, body) = request.into_parts();
    let mut send = client.post(sub.endpoint.clone());
    for (name, value) in &parts.headers {
        send = send.header(name.as_str(), value.as_bytes());
    }
    let response = send.body(body).send().await.map_err(|_| Failure::Retry)?;
    match response.status().as_u16() {
        200..=299 => Ok(()),
        404 | 410 => Err(Failure::Gone),
        _ => Err(Failure::Retry),
    }
}
