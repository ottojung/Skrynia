pub struct Frame {
    pub instance: wasmtime::Instance,
    pub last_overflow: i32,
}

impl Frame {
    pub fn root(instance: wasmtime::Instance) -> Self {
        Self {
            instance,
            last_overflow: -1,
        }
    }
    pub fn child(instance: wasmtime::Instance) -> Self {
        Self {
            instance,
            last_overflow: -1,
        }
    }
}
