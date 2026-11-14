use crate::config_number as number;
use std::io;

pub struct PushConfig {
    pub subject: String,
    pub timeout_ms: u64,
    pub concurrency: usize,
    pub max_attempts: u32,
    pub max_subscriptions: usize,
    pub max_outbox: usize,
}
impl PushConfig {
    pub fn load(public: &str) -> io::Result<Self> {
        Ok(Self {
            subject: std::env::var("SKRYNIA_PUSH_SUBJECT").unwrap_or_else(|_| public.into()),
            timeout_ms: number("SKRYNIA_PUSH_SEND_TIMEOUT_MS", 10000, u64::MAX)?,
            concurrency: number("SKRYNIA_PUSH_CONCURRENCY", 8, 100)? as usize,
            max_attempts: number("SKRYNIA_PUSH_MAX_ATTEMPTS", 12, 100)? as u32,
            max_subscriptions: number("SKRYNIA_MAX_SUBSCRIPTIONS", 2000, 1000000)? as usize,
            max_outbox: number("SKRYNIA_MAX_OUTBOX_ITEMS", 10000, 1000000)? as usize,
        })
    }
}
