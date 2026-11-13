use crate::e2e::fixture::{Server, corrupt::inject};
#[tokio::test]
async fn unknown_grant_fields_fail_startup() {
    let mut server = Server::new().await;
    let (id, _) = server.root();
    let record = server.home.join(format!("state/grants/{id}.json"));
    inject(&record, None);
    server.reject_restart().await;
}
#[tokio::test]
async fn unknown_user_fields_fail_startup() {
    let mut server = Server::new().await;
    server.user().await;
    let (id, _) = server.root();
    let record = server.home.join(format!("state/grants/{id}.json"));
    inject(&record, Some("user"));
    server.reject_restart().await;
}
