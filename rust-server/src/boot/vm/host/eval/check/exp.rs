fn main_sig(ft: &wasmtime::FuncType) -> bool {
    let p: Vec<_> = ft.params().collect();
    let r: Vec<_> = ft.results().collect();
    matches!(
        p.as_slice(),
        [wasmtime::ValType::I32, wasmtime::ValType::I32]
    ) && matches!(r.as_slice(), [wasmtime::ValType::I32])
}

pub fn ok(module: &wasmtime::Module) -> bool {
    let mut mem = false;
    let mut main = false;
    for ex in module.exports() {
        match ex.name() {
            "memory" => {
                if ex.ty().memory().is_none() {
                    return false;
                }
                mem = true;
            }
            "skrynia_main" => {
                let ty = ex.ty();
                let Some(ft) = ty.func() else {
                    return false;
                };
                if !main_sig(ft) {
                    return false;
                }
                main = true;
            }
            _ => {}
        }
    }
    mem && main
}
