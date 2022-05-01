#[derive(Debug)]
pub enum Error {
    FileNotFound,
    ParseError(String),
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

impl Error {
    pub(crate) fn no_current_vendor() -> Error {
        Error::ParseError("trying to add a device without a vendor".to_owned())
    }

    pub(crate) fn no_current_device() -> Error {
        Error::ParseError("trying to add a subdevice without a device".to_owned())
    }
}
