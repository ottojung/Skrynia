impl super::Vm {
    pub fn try_mutation_lock(&mut self) -> bool {
        if self.mutation_held {
            return false;
        }
        if !self.ram.try_mutation_lock() {
            return false;
        }
        self.mutation_held = true;
        true
    }
    pub fn release_mutation_lock(&mut self) -> bool {
        if !self.mutation_held {
            return false;
        }
        self.mutation_held = false;
        self.ram.release_mutation_lock();
        true
    }
}
impl Drop for super::Vm {
    fn drop(&mut self) {
        if self.mutation_held {
            // Close unfinished disk/output handles before a competing writer
            // can enter the next root request after a trap or fuel abort.
            self.fds = crate::fs::table::Table::new();
            self.ram.release_mutation_lock();
        }
    }
}
