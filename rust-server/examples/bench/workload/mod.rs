mod requests;
use super::fixture::Target;
pub async fn run(
    target: &Target,
    n: usize,
) -> Result<(Vec<f64>, Vec<f64>), Box<dyn std::error::Error>> {
    let no_op = requests::run(target, n, b'N').await?;
    let disk = requests::run(target, n, b'W').await?;
    target.verify_write()?;
    Ok((no_op, disk))
}
