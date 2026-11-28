use super::Ram;
use std::sync::atomic::Ordering;

impl Ram {
    pub fn read_byte(&self, off: u64) -> Result<u8, &'static str> {
        self.span(off, 1, false, |page, at, _| {
            page.map_or(0, |p| p[at].load(Ordering::SeqCst))
        })
    }
    pub fn write_byte(&self, off: u64, byte: u8) -> Result<(), &'static str> {
        self.span(off, 1, true, |page, at, _| {
            page.expect("allocated")[at].store(byte, Ordering::SeqCst)
        })
    }
    pub fn cas1(&self, off: u64, expected: u8, desired: u8) -> Result<u8, &'static str> {
        self.span(off, 1, true, |page, at, _| {
            page.expect("allocated")[at]
                .compare_exchange(expected, desired, Ordering::SeqCst, Ordering::SeqCst)
                .unwrap_or_else(|observed| observed)
        })
    }
}
