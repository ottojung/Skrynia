use super::func::Selected;
use super::link::Parent;
use wasmtime::{Caller, FuncType, Instance, Module, Store};

const ERR_IO: i64 = -5;

pub fn execute(
    c: &mut Caller<'_, crate::boot::vm::state::Vm>,
    module: &Module,
    binding: Option<FuncType>,
    selected: Option<Selected>,
    exported_start: bool,
) -> i64 {
    let engine = c.data().engine.clone();
    let mut store = Store::new(
        &engine,
        Parent {
            caller: c,
            func: selected.map(|s| s.func),
        },
    );
    let imports = super::link::imports(&mut store, binding);
    let instance = match Instance::new(&mut store, module, &imports) {
        Ok(v) => v,
        Err(_) => return ERR_IO,
    };
    if exported_start {
        let start = match instance.get_func(&mut store, "_start") {
            Some(v) => v,
            None => return ERR_IO,
        };
        if start.call(&mut store, &[], &mut []).is_err() {
            return ERR_IO;
        }
    }
    0
}
