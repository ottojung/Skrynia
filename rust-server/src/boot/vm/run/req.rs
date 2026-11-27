#[rustfmt::skip]
pub fn invoke(
    kernel: &crate::boot::Kernel,
    input: Box<dyn std::io::Read + Send>,
    max_bytes: u64,
    output: Box<dyn std::io::Write + Send>,
) -> Result<(), i32> {
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
    let fd = store.data_mut().fds.insert(crate::fs::store::Item::Input(crate::fs::store::item::Input::new(input, max_bytes))).map_err(|_| crate::boot::vm::codes::ERR_LIMIT)?;
    let output_fd = store.data_mut().fds.insert(crate::fs::store::Item::Output(crate::fs::store::item::Output::new(output))).map_err(|_| crate::boot::vm::codes::ERR_LIMIT)?;
    let inst = kernel.pre.instantiate(&mut store).map_err(|_| crate::boot::vm::codes::ERR_FAULT)?;
    super::root::call_root(&mut store, inst, fd.0, output_fd.0)
}
