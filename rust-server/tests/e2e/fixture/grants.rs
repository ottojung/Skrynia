use super::Server;
use reqwest::{Method, StatusCode};
use serde_json::{Value, json};
impl Server {
    pub async fn delegated_user(&self, parent: &str) -> String {
        let create = self
            .call(Method::POST, "/grants")
            .bearer_auth(parent)
            .json(&json!({"locations":"team.*","operations":["create","list"],
                "maxBytes":5000,"maxObjects":5}))
            .send()
            .await
            .unwrap();
        assert_eq!(create.status(), StatusCode::OK);
        let child: Value = create.json().await.unwrap();
        let issue = self
            .call(
                Method::POST,
                &format!("/grants/{}/users", child["id"].as_str().unwrap()),
            )
            .bearer_auth(child["issuerToken"].as_str().unwrap())
            .send()
            .await
            .unwrap();
        assert_eq!(issue.status(), StatusCode::OK);
        let issued: Value = issue.json().await.unwrap();
        issued["userToken"].as_str().unwrap().into()
    }
}
