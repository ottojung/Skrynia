use super::Target;
use sha2::{Digest, Sha256};
use std::time::Instant;
pub async fn run(
    target: &Target,
    n: usize,
    bytes: &[u8],
) -> Result<Vec<f64>, Box<dyn std::error::Error>> {
    let mut timings = Vec::with_capacity(n);
    let tag = format!("\"{}\"", hex::encode(Sha256::digest(bytes)));
    for _ in 0..n {
        let now = Instant::now();
        let response = target
            .client
            .get(format!("{}/store/bench", target.base))
            .send()
            .await?
            .error_for_status()?;
        assert_eq!(response.headers()["etag"], tag);
        assert_eq!(response.bytes().await?, bytes);
        timings.push(now.elapsed().as_secs_f64());
    }
    Ok(timings)
}
