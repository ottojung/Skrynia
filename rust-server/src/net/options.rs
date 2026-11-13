use std::io;
use std::time::Duration;
pub struct NetworkOptions {
    pub max_connections: usize,
    pub idle_timeout: Duration,
}
pub fn number(name: &str, default: u64, maximum: u64) -> io::Result<u64> {
    let value = std::env::var(name)
        .ok()
        .map_or(Ok(default), |s| s.parse())
        .map_err(|_| io::Error::other(format!("invalid {name}")))?;
    if value == 0 || value > maximum {
        return Err(io::Error::other(format!("invalid {name}")));
    }
    Ok(value)
}
impl NetworkOptions {
    pub fn load() -> io::Result<Self> {
        let max_connections = number("SKRYNIA_MAX_CONNECTIONS", 512, 100000)? as usize;
        let idle_timeout =
            Duration::from_millis(number("SKRYNIA_SOCKET_IDLE_TIMEOUT_MS", 30000, 86400000)?);
        Ok(Self {
            max_connections,
            idle_timeout,
        })
    }
}
