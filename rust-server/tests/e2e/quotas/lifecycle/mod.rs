mod mutation;
mod tokens;
use super::{
    Server,
    common::usage,
    grant_api::{create, issue},
    requests::send,
};
use reqwest::{Method, StatusCode};
#[tokio::test]
async fn nested_grant_metadata_and_revocation_keep_counters_consistent() {
    let mut server = Server::new().await;
    let root = server.user().await;
    let parent = create(&server, &root, "team.*", 5100).await;
    let pid = parent["id"].as_str().unwrap();
    let puser = issue(&server, &parent).await;
    let ptoken = puser["userToken"].as_str().unwrap();
    let grand = create(&server, ptoken, "team.a*", 4000).await;
    let grand_user = issue(&server, &grand).await;
    let grand_token = grand_user["userToken"].as_str().unwrap();
    assert_eq!(usage(&server, pid, ptoken).await["bytes"], 4096);
    tokens::cycle(&server, &parent, pid, ptoken).await;
    mutation::exercise(&mut server, pid, ptoken, &grand, grand_token).await;
}
