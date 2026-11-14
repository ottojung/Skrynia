use crate::PushConfig;
use url::Url;
pub fn parse(value: &str) -> Option<Url> {
    if value.is_empty() || value.len() > 2048 {
        return None;
    }
    let url = Url::parse(value).ok()?;
    let host = url.host_str()?.to_ascii_lowercase();
    let unbracketed = host.trim_start_matches('[').trim_end_matches(']');
    if url.scheme() != "https"
        || !url.username().is_empty()
        || url.password().is_some()
        || unbracketed.parse::<std::net::IpAddr>().is_ok()
        || !host.contains('.')
        || host.ends_with('.')
        || [
            ".local",
            ".localhost",
            ".internal",
            ".test",
            ".invalid",
            ".example",
        ]
        .iter()
        .any(|suffix| host.ends_with(suffix))
    {
        return None;
    }
    Some(url)
}
pub fn endpoint_allowed(value: &str, config: &PushConfig) -> bool {
    let Some(url) = parse(value) else {
        return false;
    };
    let host = url.host_str().unwrap();
    config.allowed_hosts.contains("*")
        || config.allowed_hosts.contains(host)
        || config
            .allowed_hosts
            .iter()
            .filter_map(|name| name.strip_prefix("*."))
            .any(|suffix| {
                host.strip_suffix(suffix)
                    .is_some_and(|prefix| prefix.ends_with('.'))
            })
}
