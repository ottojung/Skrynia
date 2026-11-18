const FUEL: u64 = 10_000_000;

pub fn invoke(
    kernel: &crate::boot::Kernel,
    input: Box<dyn std::io::Read + Send>,
    max_bytes: u64,
) -> Result<Vec<u8>, i32> {
    let engine = kernel.engine().clone();
    let cache = kernel.cache().clone();
    let mut store = wasmtime::Store::new(
        &engine,
        crate::boot::vm::state::Vm::new(
            engine.clone(),
            kernel.fs().clone(),
            kernel.ram().clone(),
            cache,
        ),
    );
    store.limiter(|vm| &mut vm.limits);
    let fd = store.data_mut().fds.insert(crate::fs::store::Item::Input(
        crate::fs::store::item::Input::new(input, max_bytes),
    ));
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
    super::root::call_root(&mut store, fd.0)
}
