use super::Failure;
use std::net::SocketAddr;

pub async fn choose(endpoint: &str) -> Result<(String, SocketAddr), Failure> {
    let url = crate::parse_push_endpoint(endpoint).ok_or(Failure::Blocked)?;
    let host = url.host_str().ok_or(Failure::Blocked)?.to_string();
    let port = url.port_or_known_default().ok_or(Failure::Blocked)?;
    let address = {
        let mut addresses = tokio::net::lookup_host((host.as_str(), port))
            .await
            .map_err(|_| Failure::Retry)?;
        addresses
            .find(|address| crate::public_address(address.ip()))
            .ok_or(Failure::Blocked)?
    };
    Ok((host, address))
}
