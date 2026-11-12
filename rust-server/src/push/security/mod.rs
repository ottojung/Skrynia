mod address;
mod endpoint;
pub use address::public_address;
pub use endpoint::{endpoint_allowed, parse as parse_endpoint};

#[cfg(test)]
mod tests;
