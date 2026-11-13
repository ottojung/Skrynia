use std::io;
#[derive(Debug)]
pub enum Failure {
    Clean(io::Error),
    Uncertain(io::Error),
}
impl Failure {
    pub fn error(&self) -> &io::Error {
        match self {
            Self::Clean(error) | Self::Uncertain(error) => error,
        }
    }
}
