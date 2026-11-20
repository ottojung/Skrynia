impl super::Vm {
    pub fn root_mut(&mut self) -> &mut super::Frame {
        &mut self.frames[0]
    }
}
