use reqwest::{Client, Method, StatusCode};
use sha2::{Digest, Sha256};
pub async fn run(client: Client, url: String, token: String) {
    let mut data = "seed".to_owned();
    for iteration in 0..25 {
        if iteration % 4 == 0 {
            let next = format!("update-{iteration}");
            let hash = hex::encode(Sha256::digest(data.as_bytes()));
            let reply = client
                .request(Method::PUT, &url)
                .bearer_auth(&token)
                .header("X-Skrynia-If-SHA256", hash)
                .body(next.clone())
                .send()
                .await
                .unwrap();
            assert_eq!(reply.status(), StatusCode::OK);
            data = next;
        } else {
            let reply = client.get(&url).send().await.unwrap();
            assert_eq!(reply.status(), StatusCode::OK);
            let tag = reply.headers()["etag"].to_str().unwrap().to_owned();
            let bytes = reply.bytes().await.unwrap();
            assert_eq!(bytes, data.as_bytes());
            assert_eq!(tag, format!("\"{}\"", hex::encode(Sha256::digest(&bytes))));
        }
    }
}
