mod auth;
mod caps;
mod op;
pub use auth::FsAuth;
pub use caps::{FsCaps, path_allowed};
pub use op::{FsOp, FsRoot};

impl FsAuth {
    pub fn root() -> Self {
        Self::Full
    }
    pub fn none() -> Self {
        Self::Empty
    }
}
