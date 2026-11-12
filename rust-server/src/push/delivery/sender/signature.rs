use super::Failure;
use crate::{PushConfig, Subscription, Vapid};
use web_push::{ContentEncoding, SubscriptionInfo, VapidSignatureBuilder, WebPushMessageBuilder};

pub fn create(
    sub: &Subscription,
    config: &PushConfig,
    vapid: &Vapid,
) -> Result<web_push::WebPushMessage, Failure> {
    let info = SubscriptionInfo::new(
        sub.endpoint.clone(),
        sub.keys.p256dh.clone(),
        sub.keys.auth.clone(),
    );
    let mut signature = VapidSignatureBuilder::from_base64(&vapid.private_key, &info)
        .map_err(|_| Failure::Retry)?;
    signature.add_claim("sub", config.subject.clone());
    let signature = signature.build().map_err(|_| Failure::Retry)?;
    let mut message = WebPushMessageBuilder::new(&info);
    message.set_payload(ContentEncoding::Aes128Gcm, b"changed");
    message.set_vapid_signature(signature);
    message.build().map_err(|_| Failure::Retry)
}
