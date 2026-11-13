pub fn print(label: &str, readings: &[f64]) {
    let mut sorted = readings.to_vec();
    sorted.sort_by(|a, b| a.total_cmp(b));
    let seconds: f64 = readings.iter().sum();
    let rank = |p: usize| sorted[((sorted.len() - 1) * p) / 100] * 1000.0;
    println!(
        "{label}: n={} ops/s={:.1} p50={:.2}ms p95={:.2}ms p99={:.2}ms",
        readings.len(),
        readings.len() as f64 / seconds,
        rank(50),
        rank(95),
        rank(99)
    );
}
