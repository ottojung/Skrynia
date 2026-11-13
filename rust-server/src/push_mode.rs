#[cfg(debug_assertions)]
pub fn delivery_enabled() -> bool {
    !std::env::var("SKRYNIA_TEST_DISABLE_PUSH").is_ok_and(|v| v == "1")
}
#[cfg(not(debug_assertions))]
pub fn delivery_enabled() -> bool {
    true
}
#[cfg(test)]
mod tests {
    #[test]
    fn test_mode_is_compilation_gated() {
        #[cfg(not(debug_assertions))]
        assert!(super::delivery_enabled());
    }
}
