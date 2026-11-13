#[cfg(debug_assertions)]
fn disabled(value: Option<&str>) -> bool {
    value == Some("1")
}
#[cfg(debug_assertions)]
pub fn delivery_enabled() -> bool {
    let value = std::env::var("SKRYNIA_TEST_DISABLE_PUSH").ok();
    !disabled(value.as_deref())
}
#[cfg(not(debug_assertions))]
pub fn delivery_enabled() -> bool {
    true
}
#[cfg(test)]
mod tests {
    #[test]
    fn delivery_mode_is_build_gated() {
        #[cfg(debug_assertions)]
        {
            assert!(super::disabled(Some("1")));
            assert!(!super::disabled(None));
            assert!(!super::disabled(Some("0")));
        }
        #[cfg(not(debug_assertions))]
        assert!(super::delivery_enabled());
    }
}
