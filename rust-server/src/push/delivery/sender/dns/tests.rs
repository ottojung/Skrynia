use super::{Failure, select};
use std::net::SocketAddr;
#[test]
fn dns_rebinding_cannot_select_private_or_special_destinations() {
    let addresses = [
        "127.0.0.1:443",
        "10.2.3.4:443",
        "[::1]:443",
        "[::ffff:127.0.0.1]:443",
        "192.168.1.2:443",
    ]
    .into_iter()
    .map(|s| s.parse::<SocketAddr>().unwrap());
    assert_eq!(select(addresses), Err(Failure::Retry));
    let mixed = ["127.0.0.1:443", "1.1.1.1:443", "10.0.0.1:443"]
        .into_iter()
        .map(|s| s.parse::<SocketAddr>().unwrap());
    assert_eq!(select(mixed).unwrap().to_string(), "1.1.1.1:443");
}
#[test]
fn ipv6_transition_and_documentation_addresses_remain_blocked() {
    let addresses = ["[2002::1]:443", "[2001:db8::1]:443", "[2001::1]:443"]
        .into_iter()
        .map(|s| s.parse::<SocketAddr>().unwrap());
    assert_eq!(select(addresses), Err(Failure::Retry));
    assert_eq!(
        select(["[2606:4700:4700::1111]:443".parse().unwrap()].into_iter())
            .unwrap()
            .to_string(),
        "[2606:4700:4700::1111]:443"
    );
}
