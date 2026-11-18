use super::Entry;

pub fn open_dir(
    root: &crate::fs::root::Root,
    rel: &std::path::Path,
) -> Result<Entry, &'static str> {
    let parts = super::super::root::walk::parts(rel)?;
    let fd = super::path::open_path(root.fd(), &parts)?;
    let names = super::names::read_names(&fd)?;
    Ok(Entry { names, pos: 0, fd })
}
