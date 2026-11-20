pub struct Frame {
    pub instance: wasmtime::Instance,
}

impl Frame {
    pub fn root(instance: wasmtime::Instance) -> Self {
        Self { instance }
    }
    pub fn child(instance: wasmtime::Instance) -> Self {
        Self { instance }
    }
}
