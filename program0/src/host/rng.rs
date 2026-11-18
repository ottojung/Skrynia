pub struct KernelRng;

impl crate::cap::Random for KernelRng {
    type Err = i32;
    fn fill(&self, out: &mut [u8]) -> Result<(), i32> {
        let bits = out.len().checked_mul(8).ok_or(-10)?;
        let bits = u32::try_from(bits).map_err(|_| -10)?;
        let rc = unsafe { crate::root_abi::secure_random_bits(out.as_mut_ptr() as u32, bits) };
        if rc < 0 { Err(rc) } else { Ok(()) }
    }
}
