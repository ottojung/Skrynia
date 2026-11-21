fn sig_ok(name: &str, ft: &wasmtime::FuncType, eval: bool) -> bool {
    let p: Vec<_> = ft.params().collect();
    let r: Vec<_> = ft.results().collect();
    if !matches!(r.as_slice(), [wasmtime::ValType::I32]) {
        return false;
    }
    match name {
        "invoke" => p.len() == 4 && p.iter().all(|x| matches!(x, wasmtime::ValType::I32)),
        "read" | "write" => p.len() == 3 && p.iter().all(|x| matches!(x, wasmtime::ValType::I32)),
        "close" => matches!(p.as_slice(), [wasmtime::ValType::I32]),
        "eval" | "eval_delegate" if eval => {
            p.len() == 5 && p.iter().all(|x| matches!(x, wasmtime::ValType::I32))
        }
        _ => false,
    }
}
pub fn ok(module: &wasmtime::Module, eval: bool) -> bool {
    for imp in module.imports() {
        if imp.module() != "skrynia" {
            return false;
        }
        let ty = imp.ty();
        let Some(ft) = ty.func() else {
            return false;
        };
        if !sig_ok(imp.name(), ft, eval) {
            return false;
        }
    }
    true
}
