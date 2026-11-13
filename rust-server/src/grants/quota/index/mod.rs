mod aggregate;
mod attach;
mod boot;
mod lookup;
mod meta;
mod node;
mod scan;
mod scope;
pub use aggregate::Usage;
pub use node::Node;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
pub type Shared = Arc<Mutex<Usage>>;
#[derive(Default)]
pub struct QuotaIndex {
    pub scopes: Node,
    pub metadata: HashMap<String, Usage>,
    pub by_grant: HashMap<String, Shared>,
}
