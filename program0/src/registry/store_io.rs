use super::cap::Disk;
use crate::id;
use crate::store;

#[derive(Debug)]
pub enum StoreErr<E> {
    Disk(E),
    BadId,
    Missing,
    BadRecord,
}

pub fn put<D: Disk>(disk: &D, dir: &str, pid: &[u8], code: &[u8]) -> Result<(), StoreErr<D::Err>> {
    id::validate(pid).map_err(|_| StoreErr::BadId)?;
    let path = store::path_for(dir, pid).ok_or(StoreErr::BadId)?;
    disk.write_atomic(&path, &store::encode_record(code))
        .map_err(StoreErr::Disk)
}

pub fn get<D: Disk>(disk: &D, dir: &str, pid: &[u8]) -> Result<Vec<u8>, StoreErr<D::Err>> {
    let path = store::path_for(dir, pid).ok_or(StoreErr::BadId)?;
    let bytes = disk
        .read_all(&path)
        .map_err(StoreErr::Disk)?
        .ok_or(StoreErr::Missing)?;
    store::decode_record(&bytes).ok_or(StoreErr::BadRecord)
}

pub fn del<D: Disk>(disk: &D, dir: &str, pid: &[u8]) -> Result<(), StoreErr<D::Err>> {
    id::validate(pid).map_err(|_| StoreErr::BadId)?;
    let path = store::path_for(dir, pid).ok_or(StoreErr::BadId)?;
    disk.remove(&path).map_err(StoreErr::Disk)
}
