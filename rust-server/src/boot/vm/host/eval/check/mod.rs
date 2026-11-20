mod exp;
mod imp;

pub fn module_ok(module: &wasmtime::Module) -> bool {
    imp::ok(module) && exp::ok(module)
}
