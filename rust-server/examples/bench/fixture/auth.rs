use super::Target;
use serde_json::Value;
use std::fs;
impl Target {
    pub async fn user(&self) -> Result<String, Box<dyn std::error::Error>> {
        let path = fs::read_dir(self.home.join("state/grants"))?
            .next()
            .ok_or("root grant missing")??
            .path();
        let root: Value = serde_json::from_slice(&fs::read(path)?)?;
        let id = root["id"].as_str().ok_or("missing root ID")?;
        let issuer = root["rootIssuerToken"].as_str().ok_or("missing issuer")?;
        let response = self
            .client
            .post(format!("{}/grants/{id}/users", self.base))
            .bearer_auth(issuer)
            .send()
            .await?
            .error_for_status()?;
        let token: Value = response.json().await?;
        Ok(token["userToken"]
            .as_str()
            .ok_or("missing user token")?
            .to_owned())
    }
}
