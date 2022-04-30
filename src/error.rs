#[derive(Debug)]
pub enum Error {
    FileNotFound,
    IoError(std::io::Error),
    #[cfg(feature = "online")]
    RequestError(ureq::Error),
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Self::IoError(error)
    }
}

#[cfg(feature = "online")]
impl From<ureq::Error> for Error {
    fn from(error: ureq::Error) -> Self {
        Self::RequestError(error)
    }
}
