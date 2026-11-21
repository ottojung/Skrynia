use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};
static ACTIVE: AtomicUsize = AtomicUsize::new(0);
const MAX_PARALLEL: usize = 4;
pub struct Guard;
pub fn acquire() -> Result<Guard, &'static str> {
    let start = Instant::now();
    loop {
        if ACTIVE
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |n| {
                (n < MAX_PARALLEL).then_some(n + 1)
            })
            .is_ok()
        {
            return Ok(Guard);
        }
        if start.elapsed() >= Duration::from_millis(crate::boot::limits::MAX_COMPILE_MS) {
            return Err("limit");
        }
        std::thread::sleep(Duration::from_millis(2));
    }
}
impl Drop for Guard {
    fn drop(&mut self) {
        ACTIVE.fetch_sub(1, Ordering::AcqRel);
    }
}
