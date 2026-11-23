use super::Target;
use std::time::Instant;
pub async fn run(
    target: &Target,
    n: usize,
    kind: u8,
) -> Result<Vec<f64>, Box<dyn std::error::Error>> {
    let mut samples = Vec::with_capacity(n);
    let mut body = vec![kind];
    if kind == b'W' {
        body.extend(vec![b'A'; 4096]);
    }
    for _ in 0..n {
        let now = Instant::now();
        let response = target
            .client
            .post(format!("{}/", target.base))
            .body(body.clone())
            .send()
            .await?;
        if !response.status().is_success() {
            return Err(format!("POST {} got HTTP {}", kind as char, response.status()).into());
        }
        if response.bytes().await?.as_ref() != [kind] {
            return Err("v3 benchmark response mismatch".into());
        }
        samples.push(now.elapsed().as_secs_f64());
    }
    Ok(samples)
}
