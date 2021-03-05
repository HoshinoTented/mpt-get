use std::fmt::Display;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum ErrorKind {
    Index,
    IO,
    Parse
}

#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
    message: String
}

impl std::error::Error for Error {
    
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let kind_human = match self.kind {
            ErrorKind::Index => "updating index",
            ErrorKind::IO => "I/O",
            ErrorKind::Parse => "parsing"
        };

        write!(f, "Error occursed when {}: {}", kind_human, self.message)
    }
}

pub fn index_err<S: ToString>(message: S) -> Error {
    Error {
        kind: ErrorKind::Index,
        message: message.to_string()
    }
}

pub fn io_err<S: ToString>(message: S) -> Error {
    Error {
        kind: ErrorKind::IO,
        message: message.to_string()
    }
}

pub fn parse_err<S: ToString>(message: S) -> Error {
    Error {
        kind: ErrorKind::Parse,
        message: message.to_string()
    }
}

pub trait ToError {
    fn to_index_err<S: ToString>(&self, message: S) -> Error {
        index_err(message)
    }
}

impl <T> ToError for T {}

pub trait AsResult<T> {
    fn as_index_err<S: ToString>(self, message: S) -> Result<T>;
}

impl <T, E> AsResult<T> for std::result::Result<T, E> {
    fn as_index_err<S: ToString>(self, message: S) -> Result<T> {
        self.map_err(| err | err.to_index_err(message))
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        io_err(format!("{:?}", err))
    }
}

impl From<git2::Error> for Error {
    fn from(err: git2::Error) -> Self {
        index_err(format!("{:?}", err))
    }
}

impl From<regex::Error> for Error {
    fn from(err: regex::Error) -> Self {
        Error {
            kind: ErrorKind::Parse,
            message: format!("{:?}", err)
        }
    }
}

impl From<hyper::http::uri::InvalidUri> for Error {
    fn from(err: hyper::http::uri::InvalidUri) -> Self {
        Error {
            kind: ErrorKind::Parse,
            message: format!("{:?}", err)
        }
    }
}