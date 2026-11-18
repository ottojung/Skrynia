pub struct KernelEval;

impl crate::cap::Eval for KernelEval {
    type Err = i32;
    fn eval(&self, wasm: &[u8], input: &[u8]) -> Result<Vec<u8>, i32> {
        const CAP: usize = 1024 * 1024;
        let mut out = vec![0u8; CAP];
        let rc = unsafe {
            crate::root_abi::eval(
                wasm.as_ptr() as u32,
                wasm.len() as u32,
                input.as_ptr() as u32,
                input.len() as u32,
                -1,
                out.as_mut_ptr() as u32,
                out.len() as u32,
            )
        };
        if rc < 0 || rc as usize > CAP {
            return Err(if rc < 0 { rc } else { -11 });
        }
        out.truncate(rc as usize);
        Ok(out)
    }
}
