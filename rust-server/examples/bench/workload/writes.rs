use super::Target;
use sha2::{Digest, Sha256};
use std::time::Instant;
pub async fn run(
    target: &Target,
    token: &str,
    n: usize,
    mut data: Vec<u8>,
) -> Result<Vec<f64>, Box<dyn std::error::Error>> {
    let mut timings = Vec::with_capacity(n);
    for index in 0..n {
        let next = vec![(index % 251) as u8; 4096];
        let hash = hex::encode(Sha256::digest(&data));
        let now = Instant::now();
        target
            .client
            .put(format!("{}/store/bench", target.base))
            .bearer_auth(token)
            .header("X-Skrynia-If-SHA256", hash)
            .body(next.clone())
            .send()
            .await?
            .error_for_status()?;
        timings.push(now.elapsed().as_secs_f64());
        data = next;
    }
    Ok(timings)
}
