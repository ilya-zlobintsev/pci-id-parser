use std::{fmt::Display, str::Utf8Error};

#[derive(Debug)]
pub enum Error {
    FileNotFound,
    Parse(String),
    Io(std::io::Error),
    #[cfg(feature = "online")]
    Request(Box<ureq::Error>),
    Utf8Error(Utf8Error),
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

#[cfg(feature = "online")]
impl From<ureq::Error> for Error {
    fn from(error: ureq::Error) -> Self {
        Self::Request(Box::new(error))
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::FileNotFound => write!(f, "file not found"),
            Error::Parse(err) => write!(f, "parsing error: {err}"),
            Error::Io(err) => write!(f, "io error: {err}"),
            #[cfg(feature = "online")]
            Error::Request(err) => write!(f, "network request error: {err}"),
            Error::Utf8Error(err) => write!(f, "UTF-8 error: {err}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::FileNotFound => None,
            Error::Parse(_) => None,
            Error::Io(err) => Some(err),
            #[cfg(feature = "online")]
            Error::Request(err) => Some(err),
            Error::Utf8Error(err) => Some(err),
        }
    }
}

impl From<Utf8Error> for Error {
    fn from(err: Utf8Error) -> Self {
        Self::Utf8Error(err)
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
