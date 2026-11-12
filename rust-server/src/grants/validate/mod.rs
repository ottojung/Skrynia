mod family;
mod identifiers;
mod records;
use crate::Snapshot;
use std::collections::HashSet;
use std::io;

impl Snapshot {
    pub fn validate(&self) -> io::Result<()> {
        let mut roots = 0;
        let mut users = HashSet::new();
        for grant in self.grants.values() {
            records::check(grant, &mut users)?;
            if grant.parent.is_none() {
                roots += 1;
            }
            family::check(grant, &self.grants)?;
        }
        if !self.grants.is_empty() && roots != 1 {
            return Err(io::Error::other("invalid grant roots"));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
