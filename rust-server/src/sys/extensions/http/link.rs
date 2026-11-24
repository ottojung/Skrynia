impl super::Fallback {
    pub fn link(
        &self,
        linker: &mut wasmtime::Linker<crate::boot::vm::state::Vm>,
        name: &str,
    ) -> Result<(), String> {
        let mut url = self.base.clone();
        url.path_segments_mut()
            .map_err(|_| "invalid base URL")?
            .push(name);
        let client = self.client.clone();
        linker
            .func_wrap("syscall", name, move |arg: u32| {
                super::request::call(&client, &url, arg).map_err(wasmtime::Error::msg)
            })
            .map_err(|e| e.to_string())?;
        Ok(())
    }
}
