use super::Target;
use std::time::Instant;
pub async fn run(
    target: &Target,
    token: &str,
    n: usize,
) -> Result<Vec<f64>, Box<dyn std::error::Error>> {
    let mut timings = Vec::with_capacity(n);
    let mut last = vec![0; 4096];
    for index in 0..n {
        let next = vec![(index % 251) as u8; 4096];
        let now = Instant::now();
        target
            .client
            .put(format!("{}/store/bench", target.base))
            .bearer_auth(token)
            .body(next.clone())
            .send()
            .await?
            .error_for_status()?;
        timings.push(now.elapsed().as_secs_f64());
        last = next;
    }
    let result = target
        .client
        .get(format!("{}/store/bench", target.base))
        .send()
        .await?
        .error_for_status()?
        .bytes()
        .await?;
    assert_eq!(result.as_ref(), last.as_slice());
    Ok(timings)
}
