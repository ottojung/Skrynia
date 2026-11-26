mod json;
mod ty;
mod wire;

pub use json::{from_discovery, from_version, to_list};
pub use wire::{decode, encode, fill, frame_len};

/// Ordered integer parameter and result types: `i` = i32, `l` = i64.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Signature {
    params: String,
    results: String,
}

impl Signature {
    pub fn new(params: &str, results: &str) -> Result<Self, String> {
        if !ty::valid(params) || !ty::valid(results) {
            return Err("invalid integer signature".into());
        }
        wire::frame_len(params)?;
        wire::frame_len(results)?;
        Ok(Self {
            params: params.into(),
            results: results.into(),
        })
    }

    pub fn params(&self) -> &str {
        &self.params
    }

    pub fn results(&self) -> &str {
        &self.results
    }

    pub fn func_type(&self, engine: &wasmtime::Engine) -> wasmtime::FuncType {
        wasmtime::FuncType::new(engine, ty::values(&self.params), ty::values(&self.results))
    }

    pub fn matches(&self, actual: &wasmtime::FuncType) -> bool {
        let actual_params: String = actual.params().map(ty::letter).collect();
        let actual_results: String = actual.results().map(ty::letter).collect();
        actual_params == self.params && actual_results == self.results
    }
}
