use super::State;
use std::fs::File;
use std::io;

impl State {
    pub fn open_object(&self, key: &str) -> io::Result<File> {
        File::open(self.dir.join("store").join(key))
    }
}
