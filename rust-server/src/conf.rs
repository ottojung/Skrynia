use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct Config {
    pub root_wasm: PathBuf,
    pub operator_dir: PathBuf,
    pub user_dir: PathBuf,
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
        let operator_dir = get("SKRYNIA_OPERATOR_DIR")
            .map(PathBuf::from)
            .ok_or_else(|| "SKRYNIA_OPERATOR_DIR is required".to_string())?;
        let user_dir = get("SKRYNIA_USER_DIR")
            .map(PathBuf::from)
            .ok_or_else(|| "SKRYNIA_USER_DIR is required".to_string())?;
        Ok(Self {
            root_wasm: root,
            operator_dir,
            user_dir,
        })
    }
}
