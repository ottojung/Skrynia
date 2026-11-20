use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct Config {
    pub root_wasm: PathBuf,
    pub data_dir: PathBuf,
    pub program1_wasm: Option<PathBuf>,
    pub program1_data_dir: Option<PathBuf>,
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
        let p1 = get("SKRYNIA_PROGRAM1_WASM").map(PathBuf::from);
        let scope = get("SKRYNIA_PROGRAM1_DATA_DIR").map(PathBuf::from);
        Ok(Self {
            root_wasm: root,
            data_dir: data,
            program1_wasm: p1,
            program1_data_dir: scope,
        })
    }
}
