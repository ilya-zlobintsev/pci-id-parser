#[derive(Debug)]
pub enum Error {
    FileNotFound,
    Parse(String),
    Io(std::io::Error),
    #[cfg(feature = "online")]
    Request(ureq::Error),
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

#[cfg(feature = "online")]
impl From<ureq::Error> for Error {
    fn from(error: ureq::Error) -> Self {
        Self::Request(error)
    }
}

impl Error {
    pub(crate) fn no_current_vendor() -> Error {
        Error::Parse("trying to add a device without a vendor".to_owned())
    }

    pub(crate) fn no_current_device() -> Error {
        Error::Parse("trying to add a subdevice without a device".to_owned())
    }

    pub(crate) fn no_current_class() -> Error {
        Error::Parse("trying to add a subclass without a class".to_owned())
    }

    pub(crate) fn no_current_subclass() -> Error {
        Error::Parse("trying to add a programming interface without a subclass".to_owned())
    }
}
