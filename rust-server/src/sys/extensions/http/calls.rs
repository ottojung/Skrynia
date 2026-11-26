impl super::Provider {
    pub fn into_calls(self) -> Result<super::super::Syscalls, String> {
        let mut result = super::super::Syscalls::empty();
        for (name, signature) in self.signatures {
            let mut url = self.base.clone();
            url.path_segments_mut()
                .map_err(|_| "invalid base URL")?
                .push(&name);
            let params = signature.params().to_owned();
            let results = signature.results().to_owned();
            let client = self.client.clone();
            let call = super::super::EquivalentCall::new(&name, &params, &results, move |args| {
                super::request::call(&client, &url, &signature, args)
            })?;
            result.calls.insert(name, call);
        }
        Ok(result)
    }
}
