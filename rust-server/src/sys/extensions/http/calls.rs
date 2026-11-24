impl super::Provider {
    pub fn into_calls(self) -> Result<super::super::Syscalls, String> {
        let mut result = super::super::Syscalls::empty();
        for name in self.names {
            let mut url = self.base.clone();
            url.path_segments_mut()
                .map_err(|_| "invalid base URL")?
                .push(&name);
            let client = self.client.clone();
            let call = super::super::EquivalentCall::new(&name, move |arg| {
                super::request::call(&client, &url, arg)
            })?;
            result.calls.insert(name, call);
        }
        Ok(result)
    }
}
