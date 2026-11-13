use crate::e2e::fixture::{Server, corrupt::inject};
#[tokio::test]
async fn unknown_vapid_fields_fail_startup() {
    let mut server = Server::new().await;
    inject(&server.home.join("state/push/vapid.json"), None);
    server.reject_restart().await;
}
