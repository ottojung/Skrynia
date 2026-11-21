pub fn check(module: &wasmtime::Module) -> Result<(), String> {
    let mut main = false;
    let mut memory = false;
    for export in module.exports() {
        match export.name() {
            "memory" => match export.ty() {
                wasmtime::ExternType::Memory(_) => memory = true,
                _ => return Err("root memory export has invalid type".into()),
            },
            "skrynia_main" => match export.ty() {
                wasmtime::ExternType::Func(ty) if super::shape::matches(&ty, "ii", "i") => {
                    main = true
                }
                _ => return Err("root skrynia_main export has invalid type".into()),
            },
            _ => {}
        }
    }
    if !memory {
        return Err("root must export memory".into());
    }
    if !main {
        return Err("root must export skrynia_main(input_fd,output_fd)->i32".into());
    }
    Ok(())
}
