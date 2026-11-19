mod call;
mod copy;
mod exec;
mod fin;
mod inv;
mod link;
mod mem;

pub fn run_child(
    c: &mut wasmtime::Caller<'_, crate::boot::vm::state::Vm>,
    module: &wasmtime::Module,
    input: &[u8],
    cb: Option<wasmtime::Func>,
    out_ptr: u32,
    out_cap: u32,
) -> i32 {
    if c.data().depth >= 32 {
        return crate::boot::vm::codes::ERR_LIMIT;
    }
    c.data_mut().depth += 1;
    let res = exec::run_inner(c, module, input, cb, out_ptr, out_cap);
    c.data_mut().depth -= 1;
    match res {
        Ok(n) => n,
        Err(e) => e,
    }
}
