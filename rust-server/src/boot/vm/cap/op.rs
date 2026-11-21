pub type FsRoot = crate::fs::RootKind;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FsOp {
    OpenDisk,
    OpenDir,
    Mkdir,
    Rename,
    Delete,
    Exists,
    IsDir,
    OpenDiskWrite,
}
