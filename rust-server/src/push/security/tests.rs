use super::{address::public_address, endpoint::parse};
use std::net::IpAddr;
#[test]
fn public_ips_only() {
    for value in [
        "127.0.0.1",
        "10.9.4.2",
        "192.168.1.2",
        "169.254.169.254",
        "100.64.0.1",
        "192.0.2.2",
        "2001:db8::1",
        "::1",
        "::ffff:127.0.0.1",
    ] {
        assert!(!public_address(value.parse::<IpAddr>().unwrap()), "{value}");
    }
    for value in ["8.8.8.8", "1.1.1.1", "2606:4700:4700::1111"] {
        assert!(public_address(value.parse::<IpAddr>().unwrap()), "{value}");
    }
}
#[test]
fn endpoint_cannot_name_private_hosts() {
    for value in [
        "http://push.example.org/a",
        "https://127.0.0.1/a",
        "https://[::1]/a",
        "https://[::ffff:127.0.0.1]/a",
        "https://localhost/a",
        "https://metadata.internal/a",
        "https://user:secret@example.org/",
    ] {
        assert!(parse(value).is_none(), "{value}");
    }
    assert!(parse("https://push.example.org/subscription").is_some());
}
