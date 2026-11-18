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
                if !super::sig(ft) {
                    return false;
                }
                main = true;
            }
            _ => {}
        }
    }
    mem && main
}
