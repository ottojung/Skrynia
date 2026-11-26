use wasmtime::{Caller, Linker};

const ERR_INVAL: i64 = -3;

pub fn eval(
    c: &mut Caller<'_, crate::boot::vm::state::Vm>,
    code: u64,
    code_size: u64,
    function: i32,
    arg0: i32,
    arg1: i32,
) -> i64 {
    if c.data().depth >= 32 {
        return crate::boot::vm::codes::ERR_LIMIT as i64;
    }
    if function < 0 && (arg0 != 0 || arg1 != 0) {
        return ERR_INVAL;
    }
    let bytes = match super::snapshot::snapshot(c, code, code_size) {
        Ok(v) => v,
        Err(e) => return e,
    };
    let selected = match super::func::select(c, function) {
        Ok(v) => v,
        Err(e) => return e as i64,
    };
    let module = match super::comp::compile(c, &bytes) {
        Ok(m) => m,
        Err(e) => return e as i64,
    };
    let binding = match super::binding::single(&module) {
        Ok(v) => v,
        Err(e) => return e as i64,
    };
    let mut linker = Linker::new(&c.data().engine);
    if let Err(e) = super::link::add(&mut linker, binding, selected, arg0, arg1) {
        return e as i64;
    }
    c.data_mut().depth += 1;
    let result = super::entry::execute(c, &module, &bytes, linker);
    c.data_mut().depth -= 1;
    result
}
