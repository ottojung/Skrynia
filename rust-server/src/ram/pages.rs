use std::collections::HashMap;
use std::sync::atomic::AtomicU8;
use std::sync::{Arc, RwLock};

pub const SIZE: u64 = 4096;
const SHARDS: usize = 64;
pub type Page = Arc<[AtomicU8]>;
type Shard = RwLock<HashMap<u64, Page>>;

#[derive(Debug)]
pub struct Pages([Shard; SHARDS]);

impl Default for Pages {
    fn default() -> Self {
        Self(std::array::from_fn(|_| Shard::default()))
    }
}

impl Pages {
    pub fn shard(&self, index: u64) -> &Shard {
        &self.0[(index % SHARDS as u64) as usize]
    }
    pub fn retain_below(&self, end: u64) {
        for shard in &self.0 {
            shard.write().unwrap().retain(|&idx, _| idx < end);
        }
    }
    pub fn allocate() -> Result<Page, &'static str> {
        let mut page = Vec::new();
        page.try_reserve_exact(SIZE as usize)
            .map_err(|_| "nospace")?;
        page.resize_with(SIZE as usize, || AtomicU8::new(0));
        Ok(Arc::from(page))
    }
}
