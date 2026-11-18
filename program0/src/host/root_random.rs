#[cfg(target_arch = "wasm32")]
use super::{cap::Random, root_abi as host};

#[cfg(target_arch = "wasm32")]
pub struct HostRandom;

#[cfg(target_arch = "wasm32")]
impl Random for HostRandom {
    type Err = i32;
    fn fill(&self, out: &mut [u8]) -> Result<(), i32> {
        let bits = out.len().checked_mul(8).ok_or(-10)?;
        let bits = u32::try_from(bits).map_err(|_| -10)?;
        let rc = unsafe { host::secure_random_bits(out.as_mut_ptr() as u32, bits) };
        if rc < 0 { Err(rc) } else { Ok(()) }
    }
}
