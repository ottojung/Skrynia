use super::Ram;
use super::pages::{Page, Pages, SIZE};
use std::sync::atomic::Ordering;

impl Ram {
    pub(crate) fn page(&self, off: u64, create: bool) -> Result<Option<Page>, &'static str> {
        let index = off / SIZE;
        let shard = self.pages.shard(index);
        {
            let entries = shard.read().unwrap();
            if off >= self.len() {
                return Err("oob");
            }
            if let Some(page) = entries.get(&index) {
                return Ok(Some(page.clone()));
            }
        }
        if !create {
            return Ok(None);
        }
        let mut entries = shard.write().unwrap();
        if off >= self.len() {
            return Err("oob");
        }
        if let Some(page) = entries.get(&index) {
            return Ok(Some(page.clone()));
        }
        entries.try_reserve(1).map_err(|_| "nospace")?;
        let page = Pages::allocate()?;
        entries.insert(index, page.clone());
        Ok(Some(page))
    }
    pub fn read_byte(&self, off: u64) -> Result<u8, &'static str> {
        Ok(self
            .page(off, false)?
            .map_or(0, |p| p[(off % SIZE) as usize].load(Ordering::SeqCst)))
    }
    pub fn write_byte(&self, off: u64, byte: u8) -> Result<(), &'static str> {
        let p = self.page(off, true)?.expect("allocated");
        p[(off % SIZE) as usize].store(byte, Ordering::SeqCst);
        Ok(())
    }
    pub fn cas1(&self, off: u64, expected: u8, desired: u8) -> Result<u8, &'static str> {
        let p = self.page(off, true)?.expect("allocated");
        Ok(p[(off % SIZE) as usize]
            .compare_exchange(expected, desired, Ordering::SeqCst, Ordering::SeqCst)
            .unwrap_or_else(|observed| observed))
    }
}
