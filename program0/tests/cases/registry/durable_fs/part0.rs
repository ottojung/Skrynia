use skrynia_program0::cap::{Disk, Eval, Random};
use skrynia_program0::reg::Registry;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

static N: AtomicU64 = AtomicU64::new(0);
fn tmp(name: &str) -> PathBuf {
    let t = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let n = N.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!("skrynia-p0-{name}-{t}-{n}"))
}

struct FsDisk(PathBuf);
impl FsDisk {
    fn p(&self, p: &str) -> PathBuf {
        self.0.join(p.trim_start_matches('/'))
    }
}
