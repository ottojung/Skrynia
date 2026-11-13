use super::{Server, publish, register};
use reqwest::{Method, StatusCode};
use std::fs;
#[tokio::test]
async fn fanout_restarts_preserve_generations_attempts_and_due_times() {
    let mut server = Server::new().await;
    let token = server.user().await;
    let mut ids = Vec::new();
    for n in 0..50 {
        ids.push(register(&server, &token, &format!("https://push.example.org/{n}")).await);
    }
    assert_eq!(
        publish(&server, &token, Method::POST, "first").await,
        StatusCode::CREATED
    );
    let due = 4_102_444_800u64;
    server
        .restart_with(|home| {
            let file = home.join("state/push/outbox.json");
            let mut data: serde_json::Value =
                serde_json::from_slice(&fs::read(&file).unwrap()).unwrap();
            for id in &ids {
                data[id]["attempts"] = 2.into();
                data[id]["nextAt"] = due.into();
            }
            fs::write(file, serde_json::to_vec(&data).unwrap()).unwrap();
        })
        .await;
    assert_eq!(
        publish(&server, &token, Method::PUT, "next").await,
        StatusCode::OK
    );
    server.restart_after_kill().await;
    let path = server.home.join("state/push/outbox.json");
    let state: serde_json::Value = serde_json::from_slice(&fs::read(path).unwrap()).unwrap();
    assert_eq!(state.as_object().unwrap().len(), 50);
    for id in ids {
        assert_eq!(state[&id]["generation"], 2);
        assert_eq!(state[&id]["attempts"], 2);
        assert_eq!(state[&id]["nextAt"], due);
    }
}
