use skrynia_program0::cap::Ram;
use skrynia_program0::rcache;
use std::sync::Mutex;

struct MemRam(Mutex<Vec<u8>>);
impl MemRam {
    fn new(n: usize) -> Self {
        Self(Mutex::new(vec![0; n]))
    }
}
impl Ram for MemRam {
    type Err = ();
    fn read(&self, off: u64, out: &mut [u8]) -> Result<(), ()> {
        let g = self.0.lock().unwrap();
        let s = off as usize;
        let e = s + out.len();
        out.copy_from_slice(&g[s..e]);
        Ok(())
    }
    fn write(&self, off: u64, data: &[u8]) -> Result<(), ()> {
        let mut g = self.0.lock().unwrap();
        let s = off as usize;
        let e = s + data.len();
        g[s..e].copy_from_slice(data);
        Ok(())
    }
    fn cas_u64(&self, off: u64, expect: u64, want: u64) -> Result<u64, ()> {
        let mut g = self.0.lock().unwrap();
        let s = off as usize;
        let cur = u64::from_le_bytes(g[s..s + 8].try_into().unwrap());
        if cur == expect {
            g[s..s + 8].copy_from_slice(&want.to_le_bytes());
        }
        Ok(cur)
    }
}

#[test]
fn explicit_ram_cas_slot() {
    let ram = MemRam::new(64);
    rcache::store_u64(&ram, 0, 1).unwrap();
    assert_eq!(rcache::cas_u64(&ram, 0, 1, 2).unwrap(), 1);
    assert_eq!(rcache::load_u64(&ram, 0).unwrap(), 2);
    assert_eq!(rcache::cas_u64(&ram, 0, 1, 3).unwrap(), 2);
}
