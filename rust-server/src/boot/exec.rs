use wasmtime::{Engine, Instance, Module, Store};

pub struct Request {
    pub store: Store<()>,
    pub instance: Instance,
}

pub fn prepare(engine: &Engine, module: &Module, fuel: u64) -> Result<Request, String> {
    let mut store = Store::new(engine, ());
    store.set_fuel(fuel).map_err(|e| format!("fuel: {e}"))?;
    let instance = Instance::new(&mut store, module, &[]).map_err(|e| format!("instance: {e}"))?;
    Ok(Request { store, instance })
}
