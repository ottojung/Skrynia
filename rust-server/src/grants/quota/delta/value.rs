use std::io;
#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub struct Value {
    pub objects: i128,
    pub bytes: i128,
}
impl Value {
    pub fn change(old: Option<u64>, new: Option<u64>) -> Self {
        Self {
            objects: i128::from(new.is_some()) - i128::from(old.is_some()),
            bytes: new.map_or(0, |v| i128::from(v) + 512) - old.map_or(0, |v| i128::from(v) + 512),
        }
    }
    pub fn plus(self, other: Self) -> io::Result<Self> {
        Ok(Self {
            objects: self
                .objects
                .checked_add(other.objects)
                .ok_or_else(overflow)?,
            bytes: self.bytes.checked_add(other.bytes).ok_or_else(overflow)?,
        })
    }
    pub fn minus(self, other: Self) -> io::Result<Self> {
        Ok(Self {
            objects: self
                .objects
                .checked_sub(other.objects)
                .ok_or_else(overflow)?,
            bytes: self.bytes.checked_sub(other.bytes).ok_or_else(overflow)?,
        })
    }
    pub fn is_zero(self) -> bool {
        self.objects == 0 && self.bytes == 0
    }
    pub fn usage(self) -> io::Result<crate::Usage> {
        Ok(crate::Usage {
            objects: u64::try_from(self.objects).map_err(|_| overflow())?,
            bytes: u64::try_from(self.bytes).map_err(|_| overflow())?,
        })
    }
}
fn overflow() -> io::Error {
    io::Error::other("invalid derived quota arithmetic")
}
