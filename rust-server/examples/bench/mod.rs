mod fixture;
mod report;
mod workload;
pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let samples: usize = std::env::var("SKRYNIA_BENCH_SAMPLES")
        .ok()
        .map_or(Ok(100), |s| s.parse())?;
    assert!((1..=10_000).contains(&samples));
    let target = fixture::Target::start().await?;
    let (no_op, disk) = workload::run(&target, samples).await?;
    report::print("VM POST / no-op", &no_op);
    report::print("VM POST / disk write+file sync 4 KiB", &disk);
    Ok(())
}
