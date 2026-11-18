#[cfg(target_arch = "wasm32")]
use super::{cap::Eval, root_abi as host};

#[cfg(target_arch = "wasm32")]
pub struct HostEval;

#[cfg(target_arch = "wasm32")]
impl Eval for HostEval {
    type Err = i32;
    fn eval(&self, code: &[u8], input: &[u8]) -> Result<Vec<u8>, i32> {
        let mut out = vec![0u8; 1024 * 1024];
        let n = unsafe {
            host::eval(
                code.as_ptr() as u32,
                code.len() as u32,
                input.as_ptr() as u32,
                input.len() as u32,
                -1,
                out.as_mut_ptr() as u32,
                out.len() as u32,
            )
        };
        if n < 0 {
            return Err(n);
        }
        out.truncate(n as usize);
        Ok(out)
    }
}
