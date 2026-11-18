use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct Config {
    pub root_wasm: PathBuf,
    pub data_dir: PathBuf,
}

impl Config {
    pub fn from_env() -> Result<Self, String> {
        Self::from_lookup(|k| std::env::var_os(k))
    }
    pub fn from_lookup(
        mut get: impl FnMut(&str) -> Option<std::ffi::OsString>,
    ) -> Result<Self, String> {
        let root = get("SKRYNIA_ROOT_WASM")
            .map(PathBuf::from)
            .ok_or_else(|| "SKRYNIA_ROOT_WASM is required".to_string())?;
        let data = get("SKRYNIA_DATA_DIR")
            .map(PathBuf::from)
            .ok_or_else(|| "SKRYNIA_DATA_DIR is required".to_string())?;
        Ok(Self {
            root_wasm: root,
            data_dir: data,
        })
    }
}
