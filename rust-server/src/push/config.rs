use std::collections::HashSet;
use std::io;

pub struct PushConfig {
    pub subject: String,
    pub allowed_hosts: HashSet<String>,
    pub timeout_ms: u64,
    pub concurrency: usize,
    pub max_attempts: u32,
    pub max_subscriptions: usize,
    pub max_outbox: usize,
    pub manual: bool,
}
fn number(name: &str, fallback: u64, maximum: u64) -> io::Result<u64> {
    let value = std::env::var(name)
        .ok()
        .map_or(Ok(fallback), |s| s.parse::<u64>())
        .map_err(|_| io::Error::other(format!("invalid {name}")))?;
    if value == 0 || value > maximum {
        return Err(io::Error::other(format!("invalid {name}")));
    }
    Ok(value)
}
impl PushConfig {
    pub fn load(public: &str) -> io::Result<Self> {
        let mut allowed_hosts = HashSet::new();
        let raw = std::env::var("SKRYNIA_PUSH_ALLOWED_HOSTS").unwrap_or_default();
        for name in raw.split(',').map(str::trim).filter(|s| !s.is_empty()) {
            let url = crate::parse_push_endpoint(&format!("https://{name}/"))
                .ok_or_else(|| io::Error::other("invalid allowed push host"))?;
            allowed_hosts.insert(url.host_str().unwrap().to_ascii_lowercase());
        }
        Ok(Self {
            subject: std::env::var("SKRYNIA_PUSH_SUBJECT").unwrap_or_else(|_| public.into()),
            allowed_hosts,
            timeout_ms: number("SKRYNIA_PUSH_SEND_TIMEOUT_MS", 10000, u64::MAX)?,
            concurrency: number("SKRYNIA_PUSH_CONCURRENCY", 8, 100)? as usize,
            max_attempts: number("SKRYNIA_PUSH_MAX_ATTEMPTS", 12, 100)? as u32,
            max_subscriptions: number("SKRYNIA_MAX_SUBSCRIPTIONS", 2000, 1000000)? as usize,
            max_outbox: number("SKRYNIA_MAX_OUTBOX_ITEMS", 10000, 1000000)? as usize,
            manual: std::env::var("SKRYNIA_PUSH_MANUAL").is_ok_and(|s| s == "1"),
        })
    }
}
