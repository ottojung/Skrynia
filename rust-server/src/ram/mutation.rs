use std::sync::atomic::Ordering;

impl super::Ram {
    pub fn try_mutation_lock(&self) -> bool {
        self.mutation_held
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
    }
    pub fn release_mutation_lock(&self) {
        self.mutation_held.store(false, Ordering::Release);
    }
}
