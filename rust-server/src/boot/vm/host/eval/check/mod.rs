mod exp;
mod imp;

pub fn module_ok(module: &wasmtime::Module) -> bool {
    imp::ok(module) && exp::ok(module)
}
pub fn sig(ft: &wasmtime::FuncType) -> bool {
    let params: Vec<_> = ft.params().collect();
    let results: Vec<_> = ft.results().collect();
    params.len() == 4
        && results.len() == 1
        && params.iter().all(|p| matches!(p, wasmtime::ValType::I32))
        && matches!(results[0], wasmtime::ValType::I32)
}
