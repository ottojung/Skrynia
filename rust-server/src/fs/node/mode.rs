#[derive(Clone, Copy, Debug)]
pub struct MkdirMode {
    pub create: bool,
    pub exclusive: bool,
}

impl MkdirMode {
    pub fn create_if_absent() -> Self {
        Self {
            create: true,
            exclusive: false,
        }
    }
    pub fn create_new() -> Self {
        Self {
            create: true,
            exclusive: true,
        }
    }
}
