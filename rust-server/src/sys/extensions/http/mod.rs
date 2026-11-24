mod calls;
mod discover;
mod request;

pub(super) struct Provider {
    pub names: std::collections::BTreeSet<String>,
    base: reqwest::Url,
    client: reqwest::blocking::Client,
}

impl Provider {
    pub fn discover(base: &str) -> Result<Self, String> {
        let mut base = reqwest::Url::parse(base).map_err(|e| e.to_string())?;
        if !matches!(base.scheme(), "http" | "https")
            || base.host().is_none()
            || base.query().is_some()
            || base.fragment().is_some()
            || !base.username().is_empty()
            || base.password().is_some()
        {
            return Err("invalid syscall base URL".into());
        }
        base.path_segments_mut()
            .map_err(|_| "invalid base URL")?
            .pop_if_empty();
        let client = std::thread::spawn(request::client)
            .join()
            .map_err(|_| "HTTP syscall client initialization failed")??;
        let names = discover::names(&client, &base)?;
        Ok(Self {
            names,
            base,
            client,
        })
    }
}
