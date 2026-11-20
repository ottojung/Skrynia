pub fn run(
    engine: &wasmtime::Engine,
    module: &wasmtime::Module,
    fs: std::sync::Arc<crate::fs::Fs>,
    ram: std::sync::Arc<crate::ram::Ram>,
    cache: std::sync::Arc<crate::boot::vm::state::Cache>,
    input: crate::fs::store::Item,
    output: crate::fs::store::Item,
) -> Result<(), i32> {
    let mut store = wasmtime::Store::new(
        engine,
        crate::boot::vm::state::Vm::new(engine.clone(), fs, ram, cache, None),
    );
    store.limiter(|vm| &mut vm.limits);
    store
        .set_fuel(super::req::FUEL)
        .map_err(|_| crate::boot::vm::codes::ERR_LIMIT)?;
    let in_fd = store
        .data_mut()
        .fds
        .insert(input)
        .map_err(|_| crate::boot::vm::codes::ERR_LIMIT)?;
    let out_fd = store
        .data_mut()
        .fds
        .insert(output)
        .map_err(|_| crate::boot::vm::codes::ERR_LIMIT)?;
    let mut linker = wasmtime::Linker::new(engine);
    crate::boot::vm::host::fd::add(&mut linker).map_err(|_| crate::boot::vm::codes::ERR_FAULT)?;
    let inst = linker
        .instantiate(&mut store, module)
        .map_err(|_| crate::boot::vm::codes::ERR_INVALID)?;
    store
        .data_mut()
        .frames
        .push(crate::boot::vm::state::Frame::root(inst));
    super::root::call_root(&mut store, in_fd.0, out_fd.0)
}
