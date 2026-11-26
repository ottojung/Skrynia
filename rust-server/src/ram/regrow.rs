use super::Ram;

#[test]
fn truncated_partial_page_and_removed_pages_read_zero_after_growth() {
    let r = Ram::new();
    r.truncate(16384).unwrap();
    for i in [4095, 4096, 5000, 8192, 16383] {
        r.write_byte(i, 255).unwrap();
    }
    r.truncate(4097).unwrap();
    r.truncate(16384).unwrap();
    for i in [4095, 4096] {
        assert_eq!(r.read_byte(i), Ok(255));
    }
    for i in [5000, 8192, 16383] {
        assert_eq!(r.read_byte(i), Ok(0));
    }
}
