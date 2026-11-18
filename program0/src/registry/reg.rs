use super::cap::Disk;
use crate::store_io::{self, StoreErr};
use crate::store_ls;
use std::collections::BTreeMap;
use std::sync::RwLock;

pub struct Registry {
    pub(crate) dir: String,
    pub(crate) code: RwLock<BTreeMap<Vec<u8>, Vec<u8>>>,
}

#[derive(Debug)]
pub enum RegErr<D, E, R> {
    Store(StoreErr<D>),
    Eval(E),
    Rand(R),
    BadId,
    Missing,
}

impl Registry {
    pub fn open<D: Disk>(disk: &D, dir: &str) -> Result<Self, StoreErr<D::Err>> {
        let ids = store_ls::list_ids(disk, dir)?;
        let mut code = BTreeMap::new();
        for pid in ids {
            match store_io::get(disk, dir, &pid) {
                Ok(bytes) => {
                    code.insert(pid, bytes);
                }
                Err(StoreErr::Missing | StoreErr::BadRecord | StoreErr::BadId) => {}
                Err(e) => return Err(e),
            }
        }
        Ok(Self {
            dir: dir.to_string(),
            code: RwLock::new(code),
        })
    }
}
