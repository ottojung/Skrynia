#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub struct Usage {
    pub bytes: u64,
    pub objects: u64,
}
impl Usage {
    pub fn object(size: u64) -> Self {
        Self {
            bytes: size + 512,
            objects: 1,
        }
    }
    pub fn add(&mut self, other: Self) {
        self.bytes = self
            .bytes
            .checked_add(other.bytes)
            .expect("quota byte overflow");
        self.objects = self
            .objects
            .checked_add(other.objects)
            .expect("quota object overflow");
    }
    pub fn subtract(&mut self, other: Self) {
        self.bytes = self
            .bytes
            .checked_sub(other.bytes)
            .expect("quota byte underflow");
        self.objects = self
            .objects
            .checked_sub(other.objects)
            .expect("quota object underflow");
    }
    pub fn update(&mut self, old: Option<u64>, new: Option<u64>) {
        if let Some(size) = old {
            self.subtract(Self::object(size));
        }
        if let Some(size) = new {
            self.add(Self::object(size));
        }
    }
}
