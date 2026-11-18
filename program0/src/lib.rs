//! Application-defined durable program registry and VM root.
pub mod cap;
pub mod protocol;
pub mod registry;

pub use protocol::{handle, wdec, wenc, wire};
pub use registry::{id, rcache, reg, reg_ops, store, store_io, store_ls};

#[cfg(any(target_arch = "wasm32", feature = "vm-check"))]
pub mod host;
#[cfg(any(target_arch = "wasm32", feature = "vm-check"))]
pub use host::{root_abi, root_main};
