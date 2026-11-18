use super::{
    guest,
    wire::{call, request},
};
use skrynia_server::boot::Kernel;
use std::{fs, path::Path};
pub fn run() -> Result<(), String> {
    let root = std::env::args().nth(1).ok_or("pass root .wasm path")?;
    let dir = std::env::temp_dir().join(format!("skrynia-v3-smoke-{}", std::process::id()));
    if dir.exists() {
        return Err("test data path already exists".into());
    }
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let a = guest::byte(65);
    let b = guest::byte(66);
    let run = || -> Result<(), String> {
        let k = Kernel::start(Path::new(&root), &dir)?;
        let id = call(&k, &request(1, &[], &a))?;
        if id.len() != 32 {
            return Err("bad minted id length".into());
        }
        if call(&k, &request(3, &id, b"test"))? != b"A" {
            return Err("invoke A failed".into());
        }
        call(&k, &request(2, &id, &b))?;
        if k.cached_modules() != 1 {
            return Err("missing compiled cache entry".into());
        }
        if call(&k, &request(3, &id, b"test"))? != b"B" {
            return Err("invoke B failed".into());
        }
        if k.cached_modules() != 2 {
            return Err("new code not compiled".into());
        }
        drop(k);
        let restarted = Kernel::start(Path::new(&root), &dir)?;
        if call(&restarted, &request(3, &id, &[]))? != b"B" {
            return Err("restart lost code".into());
        }
        call(&restarted, &request(5, &id, &[]))?;
        if call(&restarted, &request(3, &id, &[])).is_ok() {
            return Err("revoke failed".into());
        }
        Ok(())
    };
    let result = run();
    fs::remove_dir_all(&dir).map_err(|e| e.to_string())?;
    result?;
    Ok(())
}
