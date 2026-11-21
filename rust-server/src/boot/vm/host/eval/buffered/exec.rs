use super::prep::Args;
pub fn eval(c: &mut wasmtime::Caller<'_, crate::boot::vm::state::Vm>, a: Args) -> i32 {
    if c.data().depth >= 32 {
        return crate::boot::vm::codes::ERR_LIMIT;
    }
    c.data_mut().depth += 1;
    let r = super::run::run(c, &a);
    c.data_mut().depth -= 1;
    r
}
