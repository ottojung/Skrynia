mod plain;
mod reads;
mod writes;
use super::fixture::Target;
pub async fn run(
    target: &Target,
    token: &str,
    n: usize,
) -> Result<(Vec<f64>, Vec<f64>, Vec<f64>), Box<dyn std::error::Error>> {
    let data = vec![b'A'; 4096];
    target
        .client
        .post(format!("{}/store/bench", target.base))
        .bearer_auth(token)
        .body(data.clone())
        .send()
        .await?
        .error_for_status()?;
    let reads = reads::run(target, n, &data).await?;
    let writes = writes::run(target, token, n, data).await?;
    let plain = plain::run(target, token, n).await?;
    Ok((reads, writes, plain))
}
