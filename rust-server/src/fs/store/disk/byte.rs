use super::{Fs, Item};

impl Fs {
    pub fn read1(
        &self,
        fds: &mut crate::fs::table::Table<Item>,
        fd: crate::fs::Fd,
    ) -> Result<Option<u8>, &'static str> {
        match fds.get_mut(fd) {
            Some(Item::Input(e)) => e.read1(),
            Some(Item::Output(_) | Item::BufferedOutput(_) | Item::Dir(_)) => Err("perm"),
            Some(Item::Ram(e)) => match self.ram.read_byte(e.pos) {
                Ok(v) => {
                    e.pos = e.pos.saturating_add(1);
                    Ok(Some(v))
                }
                Err("oob") => Ok(None),
                Err(e) => Err(e),
            },
            Some(Item::File(e)) => crate::fs::disk::byte::read1(e),
            None => Err("badfd"),
        }
    }
    pub fn write1(
        &self,
        fds: &mut crate::fs::table::Table<Item>,
        fd: crate::fs::Fd,
        byte: u8,
    ) -> Result<(), &'static str> {
        match fds.get_mut(fd) {
            Some(Item::Input(_) | Item::Dir(_)) => Err("perm"),
            Some(Item::Output(e)) => e.write1(byte),
            Some(Item::BufferedOutput(e)) => e.write1(byte),
            Some(Item::Ram(e)) => {
                self.ram.write_byte(e.pos, byte)?;
                e.pos = e.pos.saturating_add(1);
                Ok(())
            }
            Some(Item::File(e)) => crate::fs::disk::byte::write1(e, byte),
            None => Err("badfd"),
        }
    }
}
