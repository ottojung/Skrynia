pub(crate) const FUEL: u64 = 128_000_000;

pub fn invoke(
    kernel: &crate::boot::Kernel,
    input: Box<dyn std::io::Read + Send>,
    max_bytes: u64,
    output: Box<dyn std::io::Write + Send>,
) -> Result<(), i32> {
    let engine = kernel.engine().clone();
    let cache = kernel.cache().clone();
    let mut vm = crate::boot::vm::state::Vm::new(
        engine.clone(),
        kernel.fs().clone(),
        kernel.ram().clone(),
        cache,
        kernel.program1().cloned(),
    );
    vm.program1_storage = kernel.program1_storage().cloned();
    let mut store = wasmtime::Store::new(&engine, vm);
    store.limiter(|vm| &mut vm.limits);
    let fd = store
        .data_mut()
        .fds
        .insert(crate::fs::store::Item::Input(
            crate::fs::store::item::Input::new(input, max_bytes),
        ))
        .map_err(|_| crate::boot::vm::codes::ERR_LIMIT)?;
    let output_fd = store
        .data_mut()
        .fds
        .insert(crate::fs::store::Item::Output(
            crate::fs::store::item::Output::new(output),
        ))
        .map_err(|_| crate::boot::vm::codes::ERR_LIMIT)?;
    store
        .set_fuel(FUEL)
        .map_err(|_| crate::boot::vm::codes::ERR_LIMIT)?;
    let mut linker = wasmtime::Linker::new(&engine);
    crate::boot::vm::link::root::add(&mut linker).map_err(|_| crate::boot::vm::codes::ERR_FAULT)?;
    let inst = linker
        .instantiate(&mut store, kernel.root())
        .map_err(|_| crate::boot::vm::codes::ERR_FAULT)?;
    store
        .data_mut()
        .frames
        .push(crate::boot::vm::state::Frame::root(inst));
    super::root::call_root(&mut store, fd.0, output_fd.0)
}
