mod fixture;
mod report;
mod workload;
pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let samples: usize = std::env::var("SKRYNIA_BENCH_SAMPLES")
        .ok()
        .map_or(Ok(100), |s| s.parse())?;
    assert!((1..=10_000).contains(&samples));
    let target = fixture::Target::start().await?;
    let token = target.user().await?;
    let (reads, writes) = workload::run(&target, &token, samples).await?;
    report::print("GET 4 KiB", &reads);
    report::print("CAS PUT 4 KiB", &writes);
    Ok(())
}
