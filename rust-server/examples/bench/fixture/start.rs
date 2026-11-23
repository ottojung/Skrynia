use super::Target;
use std::fs;
use std::time::Duration;
impl Target {
    pub async fn start() -> Result<Self, Box<dyn std::error::Error>> {
        let home = std::env::temp_dir().join(format!(
            "skrynia-bench-{}-{}",
            std::process::id(),
            rand::random::<u64>()
        ));
        fs::create_dir(&home)?;
        fs::create_dir(home.join("operator"))?;
        fs::create_dir(home.join("user"))?;
        fs::write(
            home.join("root.wasm"),
            wat::parse_str(include_str!("guest.wat"))?,
        )?;
        let listener = std::net::TcpListener::bind("127.0.0.1:0")?;
        let port = listener.local_addr()?.port();
        drop(listener);
        let child = super::launch::spawn(&home, port)?;
        let target = Self {
            base: format!("http://127.0.0.1:{port}"),
            client: reqwest::Client::new(),
            home,
            child,
        };
        for _ in 0..150 {
            if let Ok(response) = target
                .client
                .post(format!("{}/", target.base))
                .body("N")
                .send()
                .await
                && response.status().is_success()
                && response.bytes().await?.as_ref() == b"N"
            {
                return Ok(target);
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        Err("v3 POST / benchmark server did not become ready".into())
    }
}
