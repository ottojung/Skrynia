mod call;
mod copy;
mod exec;
mod fds;
mod grow;
mod inv;
mod link;

pub fn run(
    c: &mut wasmtime::Caller<'_, crate::boot::vm::state::Vm>,
    module: &wasmtime::Module,
    in_fd: i32,
    out_fd: i32,
    cb: Option<wasmtime::Func>,
) -> i32 {
    if c.data().depth >= 32 {
        return crate::boot::vm::codes::ERR_LIMIT;
    }
    c.data_mut().depth += 1;
    let r = exec::run(c, module, in_fd, out_fd, cb);
    c.data_mut().depth -= 1;
    r
}
