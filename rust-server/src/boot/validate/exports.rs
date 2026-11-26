pub fn check(module: &wasmtime::Module) -> Result<(), String> {
    match module.get_export("skrynia_main") {
        Some(wasmtime::ExternType::Func(ty)) if super::shape::matches(&ty, "ii", "i") => Ok(()),
        _ => Err("root must export skrynia_main(input_fd,output_fd)->i32".into()),
    }
}
