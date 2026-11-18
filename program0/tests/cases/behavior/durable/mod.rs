use crate::cases::support::{FsDisk, Rng};
use skrynia_program0::cap::Eval;
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

struct Tag;
impl Eval for Tag {
    type Err = ();
    fn eval(&self, wasm: &[u8], _: &[u8]) -> Result<Vec<u8>, ()> {
        Ok(wasm.to_vec())
    }
}

mod case0;
mod case1;
mod case2;
