fn sig_ok(name: &str, ft: &wasmtime::FuncType) -> bool {
    let p: Vec<_> = ft.params().collect();
    let r: Vec<_> = ft.results().collect();
    if !matches!(r.as_slice(), [wasmtime::ValType::I32]) {
        return false;
    }
    match name {
        "invoke" => matches!(
            p.as_slice(),
            [
                wasmtime::ValType::I32,
                wasmtime::ValType::I32,
                wasmtime::ValType::I32,
                wasmtime::ValType::I32
            ]
        ),
        "read" | "write" => matches!(
            p.as_slice(),
            [
                wasmtime::ValType::I32,
                wasmtime::ValType::I32,
                wasmtime::ValType::I32
            ]
        ),
        "close" => matches!(p.as_slice(), [wasmtime::ValType::I32]),
        _ => false,
    }
}

pub fn ok(module: &wasmtime::Module) -> bool {
    for imp in module.imports() {
        if imp.module() != "skrynia" {
            return false;
        }
        let ty = imp.ty();
        let Some(ft) = ty.func() else {
            return false;
        };
        if !sig_ok(imp.name(), ft) {
            return false;
        }
    }
    true
}
