use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::{Arc, RwLock};
mod cas;
#[cfg(test)]
mod concurrent;
mod io;
mod resize;
#[cfg(test)]
mod tests;

// Capacity is a host-safety bound, not an execution budget or quota.
pub const MAX_RAM_BYTES: u64 = 64 * 1024 * 1024;

#[derive(Debug, Default)]
pub struct Ram {
    slots: RwLock<Arc<Vec<AtomicU8>>>,
}

impl Ram {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn len(&self) -> u64 {
        self.slots.read().unwrap().len() as u64
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    pub fn read_byte(&self, off: u64) -> Result<u8, &'static str> {
        let b = self.slots.read().unwrap().clone();
        let i = usize::try_from(off).map_err(|_| "oob")?;
        b.get(i).map(|x| x.load(Ordering::SeqCst)).ok_or("oob")
    }
    pub fn write_byte(&self, off: u64, byte: u8) -> Result<(), &'static str> {
        let b = self.slots.read().unwrap().clone();
        let i = usize::try_from(off).map_err(|_| "oob")?;
        let x = b.get(i).ok_or("oob")?;
        x.store(byte, Ordering::SeqCst);
        Ok(())
    }
    pub fn cas1(&self, off: u64, expected: u8, desired: u8) -> Result<u8, &'static str> {
        let b = self.slots.read().unwrap().clone();
        let i = usize::try_from(off).map_err(|_| "oob")?;
        let x = b.get(i).ok_or("oob")?;
        Ok(
            x.compare_exchange(expected, desired, Ordering::SeqCst, Ordering::SeqCst)
                .unwrap_or_else(|observed| observed),
        )
    }
}
