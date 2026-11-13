use super::Server;
use reqwest::{Method, RequestBuilder};
use serde_json::Value;
use std::fs;
use std::process::Command;
impl Server {
    pub fn call(&self, method: Method, path: &str) -> RequestBuilder {
        self.client
            .request(method, format!("{}{}", self.address, path))
    }
    pub fn root(&self) -> (String, String) {
        let folder = self.home.join("state/grants");
        let entry = fs::read_dir(folder).unwrap().next().unwrap().unwrap();
        let record: Value = serde_json::from_slice(&fs::read(entry.path()).unwrap()).unwrap();
        let id = record["id"].as_str().unwrap().to_owned();
        let issuer = record["rootIssuerToken"].as_str().unwrap().to_owned();
        (id, issuer)
    }
    pub async fn user(&self) -> String {
        let (id, issuer) = self.root();
        let reply: Value = self
            .call(Method::POST, &format!("/grants/{id}/users"))
            .bearer_auth(issuer)
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        reply["userToken"].as_str().unwrap().into()
    }
    pub fn operator_credentials(&self) -> Value {
        let result = Command::new(env!("CARGO_BIN_EXE_skrynia-server"))
            .arg("root-credentials")
            .env("SKRYNIA_DATA_DIR", &self.home)
            .output()
            .unwrap();
        assert!(result.status.success());
        serde_json::from_slice(&result.stdout).unwrap()
    }
}
