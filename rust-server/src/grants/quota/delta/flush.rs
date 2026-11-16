use super::{
    DeltaIndex,
    buffer::Node,
    run::{Run, Writer},
};
use std::{fs, io};
impl DeltaIndex {
    pub fn flush(&mut self) -> io::Result<()> {
        if self.pending.is_empty() {
            return Ok(());
        }
        let mut output = Writer::new(&self.dir)?;
        for (key, delta) in &self.pending {
            output.push(key, *delta)?;
        }
        let new = output.finish()?;
        self.pending.clear();
        self.buffer = Node::default();
        self.promote(new)
    }
    fn promote(&mut self, mut current: Run) -> io::Result<()> {
        let mut level = 0;
        loop {
            if level == self.runs.len() {
                self.runs.push(None);
            }
            let Some(old) = self.runs[level].take() else {
                self.runs[level] = Some(current);
                return Ok(());
            };
            let next = Run::merge(&self.dir, &old, &current)?;
            fs::remove_file(&old.path)?;
            fs::remove_file(&current.path)?;
            current = next;
            if current.len == 0 {
                fs::remove_file(&current.path)?;
                return Ok(());
            }
            level += 1;
        }
    }
}
