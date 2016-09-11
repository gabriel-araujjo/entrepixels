use std::error::Error as ErrorTrait;
use std::convert::From;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result;
use std::io::Error as IoError;
use std::io::ErrorKind as IoErrorKind;

#[derive(Debug)]
pub struct Error {
    message: String,
}

impl Error {
    pub fn new(message: &str) -> Error {
        Error {message: String::from(message)}
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "Err: {}", self.message)
    }
}

impl ErrorTrait for Error {
    fn description(&self) -> &str {
        self.message.as_str()
    }
}

impl From<IoError> for Error {
    fn from(err: IoError) -> Error {
        Error::new(err.description())
    }
}

impl From<Error> for IoError {
    fn from(err: Error) -> IoError {
        IoError::new(IoErrorKind::Other, err.description())
    }
}