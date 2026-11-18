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
impl Disk for FsDisk {
    type Err = std::io::Error;
    fn read_all(&self, path: &str) -> Result<Option<Vec<u8>>, Self::Err> {
        match fs::read(self.p(path)) {
            Ok(b) => Ok(Some(b)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(e),
        }
    }
    fn write_atomic(&self, path: &str, bytes: &[u8]) -> Result<(), Self::Err> {
        let p = self.p(path);
        if let Some(parent) = p.parent() {
            fs::create_dir_all(parent)?;
        }
        let tmp = p.with_extension("tmp");
        fs::write(&tmp, bytes)?;
        fs::rename(tmp, p)
    }
    fn remove(&self, path: &str) -> Result<(), Self::Err> {
        match fs::remove_file(self.p(path)) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(e),
        }
    }
    fn list(&self, dir: &str) -> Result<Vec<String>, Self::Err> {
        let p = self.p(dir);
        let rd = match fs::read_dir(&p) {
            Ok(r) => r,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(vec![]),
            Err(e) => return Err(e),
        };
        let mut out = Vec::new();
        for e in rd {
            if let Some(n) = e?.file_name().to_str() {
                out.push(n.to_string());
            }
        }
        Ok(out)
    }
}
struct Rng(u8);
impl Random for Rng {
    type Err = ();
    fn fill(&self, out: &mut [u8]) -> Result<(), ()> {
        for b in out {
            *b = self.0;
        }
        Ok(())
    }
}
struct Tag;
impl Eval for Tag {
    type Err = ();
    fn eval(&self, wasm: &[u8], _: &[u8]) -> Result<Vec<u8>, ()> {
        Ok(wasm.to_vec())
    }
}

#[test]
fn durable_replace_and_recover() {
    let dir = tmp("dur");
    let disk = Arc::new(FsDisk(dir.clone()));
    let reg = Registry::open(&*disk, "reg").unwrap();
    let id = reg.register(&*disk, &Rng(11), b"v1".to_vec()).unwrap();
    reg.set(&*disk, &id, b"v2".to_vec()).unwrap();
    let reg2 = Registry::open(&*disk, "reg").unwrap();
    assert_eq!(reg2.lookup(&id).unwrap(), b"v2");
    assert_eq!(reg2.invoke(&*disk, &Tag, &id, b"").unwrap(), b"v2");
    reg2.revoke(&*disk, &id).unwrap();
    let reg3 = Registry::open(&*disk, "reg").unwrap();
    assert!(reg3.lookup(&id).is_err());
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn durable_concurrent_invoke_during_replace() {
    let dir = tmp("durc");
    let disk = Arc::new(FsDisk(dir.clone()));
    let reg = Arc::new(Registry::open(&*disk, "reg").unwrap());
    let id = reg.register(&*disk, &Rng(12), b"v1".to_vec()).unwrap();
    let mut hs = Vec::new();
    for _ in 0..4 {
        let (r, d, pid) = (reg.clone(), disk.clone(), id.clone());
        hs.push(thread::spawn(move || {
            for _ in 0..200 {
                let v = r.invoke(&*d, &Tag, &pid, b"").unwrap();
                assert!(v == b"v1" || v == b"v2");
            }
        }));
    }
    reg.set(&*disk, &id, b"v2".to_vec()).unwrap();
    for h in hs {
        h.join().unwrap();
    }
    assert_eq!(
        Registry::open(&*disk, "reg").unwrap().lookup(&id).unwrap(),
        b"v2"
    );
    let _ = fs::remove_dir_all(dir);
}
