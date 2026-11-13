use super::{Server, publish, register};
use reqwest::{Method, StatusCode};
use serde_json::Value;
use std::fs;
#[tokio::test]
async fn single_subscriber_and_fanout_share_one_durable_record_format() {
    let server = Server::new().await;
    let token = server.user().await;
    let first = register(&server, &token, "https://push.example.org/one").await;
    assert_eq!(
        publish(&server, &token, Method::POST, "first").await,
        StatusCode::CREATED
    );
    let file = server.home.join("state/push/outbox.json");
    let one: Value = serde_json::from_slice(&fs::read(&file).unwrap()).unwrap();
    assert_eq!(one.as_object().unwrap().len(), 1);
    assert_eq!(one[&first]["generation"], 1);
    assert_eq!(one[&first]["attempts"], 0);
    assert_eq!(one[&first]["nextAt"], 0);
    let mut ids = vec![first];
    for i in 0..49 {
        ids.push(register(&server, &token, &format!("https://push.example.org/{i}")).await);
    }
    assert_eq!(
        publish(&server, &token, Method::PUT, "next").await,
        StatusCode::OK
    );
    let all: Value = serde_json::from_slice(&fs::read(file).unwrap()).unwrap();
    assert_eq!(all.as_object().unwrap().len(), 50);
    assert_eq!(all[&ids[0]]["generation"], 2);
    for id in ids.iter().skip(1) {
        assert_eq!(all[id]["generation"], 1);
    }
}
