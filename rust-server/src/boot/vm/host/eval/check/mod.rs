mod exp;
mod imp;

pub fn module_ok(module: &wasmtime::Module, eval: bool) -> bool {
    imp::ok(module, eval) && exp::ok(module)
}
