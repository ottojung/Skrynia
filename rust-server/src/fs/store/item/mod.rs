mod buf;
mod input;
mod output;
pub use buf::BufferedOutput;
pub use input::Input;
pub use output::Output;
pub struct RamFd {
    pub pos: u64,
}
pub enum Item {
    Input(Input),
    Output(Output),
    BufferedOutput(BufferedOutput),
    File(crate::fs::disk::Entry),
    Dir(crate::fs::dir::Entry),
    Ram(RamFd),
}
