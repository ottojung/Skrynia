use wasmtime::{Engine, Instance, Module, Store};

pub struct Request {
    pub store: Store<()>,
    pub instance: Instance,
}

pub fn prepare(engine: &Engine, module: &Module) -> Result<Request, String> {
    let mut store = Store::new(engine, ());
    let instance = Instance::new(&mut store, module, &[]).map_err(|e| format!("instance: {e}"))?;
    Ok(Request { store, instance })
}
