const FUEL: u64 = 10_000_000;

pub fn invoke(kernel: &crate::boot::Kernel, body: &[u8]) -> Result<Vec<u8>, i32> {
    let engine = kernel.engine().clone();
    let cache = std::sync::Arc::new(crate::boot::vm::state::Cache::new());
    let mut store = wasmtime::Store::new(
        &engine,
        crate::boot::vm::state::Vm::new(
            engine.clone(),
            kernel.fs().clone(),
            kernel.ram().clone(),
            cache,
        ),
    );
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
    super::root::call_root(&mut store, body)
}
