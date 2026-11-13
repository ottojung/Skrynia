use std::io;
#[derive(Debug)]
pub enum Failure {
    Clean(io::Error),
    Uncertain(io::Error),
}
impl Failure {
    pub fn initial(file: &std::path::Path, error: io::Error) -> Self {
        if let Err(cleanup) = std::fs::remove_file(file)
            && cleanup.kind() != io::ErrorKind::NotFound
        {
            eprintln!("Skrynia initial intent cleanup: {cleanup}");
        }
        Self::Clean(error)
    }
    pub fn error(&self) -> &io::Error {
        match self {
            Self::Clean(error) | Self::Uncertain(error) => error,
        }
    }
}
