use wasmtime::{Caller, Linker, Module};

const ERR_INVAL: i64 = -3;
const ERR_IO: i64 = -5;
const ERR_TRAP: i64 = -9;

pub fn execute(
    c: &mut Caller<'_, crate::boot::vm::state::Vm>,
    module: &Module,
    bytes: &[u8],
    linker: Linker<crate::boot::vm::state::Vm>,
) -> i64 {
    let instance = match linker.instantiate(&mut *c, module) {
        Ok(v) => v,
        Err(_) => return ERR_IO,
    };
    match module.exports().find(|e| e.name() == "_start") {
        Some(export) => {
            let export_ty = export.ty();
            let Some(ty) = export_ty.func() else {
                return ERR_INVAL;
            };
            if ty.params().count() != 0 || ty.results().count() != 0 {
                return ERR_INVAL;
            }
            let start = match instance.get_func(&mut *c, "_start") {
                Some(v) => v,
                None => return ERR_IO,
            };
            let mut results: [wasmtime::Val; 0] = [];
            match start.call(&mut *c, &[], &mut results) {
                Ok(()) => 0,
                Err(_) => ERR_TRAP,
            }
        }
        None => {
            if super::start::has_start_section(bytes) {
                0
            } else {
                ERR_INVAL
            }
        }
    }
}
