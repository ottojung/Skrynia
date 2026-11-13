pub fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |duration| {
            duration.as_millis().min(u128::from(u64::MAX)) as u64
        })
}
pub fn delay(attempts: u32) -> u64 {
    let factor = 1u64 << attempts.saturating_sub(1).min(30);
    1000u64.saturating_mul(factor).min(30 * 60 * 1000)
}
#[cfg(test)]
mod tests {
    use super::delay;
    #[test]
    fn exponential_backoff_is_bounded() {
        assert_eq!(delay(1), 1000);
        assert_eq!(delay(2), 2000);
        assert_eq!(delay(3), 4000);
        assert_eq!(delay(1000), 30 * 60 * 1000);
    }
}
