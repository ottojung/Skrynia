pub fn ok(module: &wasmtime::Module) -> bool {
    let mut count = 0;
    for imp in module.imports() {
        count += 1;
        if count > 1 {
            return false;
        }
        if imp.module() != "skrynia" || imp.name() != "invoke" {
            return false;
        }
        let ty = imp.ty();
        let Some(ft) = ty.func() else {
            return false;
        };
        if !super::sig(ft) {
            return false;
        }
    }
    true
}
