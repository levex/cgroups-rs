use std::error::Error as StdError;
use std::fmt;

/// The different types of errors that can occur while manipulating control groups.
#[derive(Debug, Eq, PartialEq)]
pub enum ErrorKind {
    /// An error occured while writing to a control group file.
    WriteFailed,

    /// An error occured while trying to read from a control group file.
    ReadFailed,

    /// An error occured while trying to parse a value from a control group file.
    ///
    /// In the future, there will be some information attached to this field.
    ParseError,

    /// You tried to do something invalid.
    ///
    /// This could be because you tried to set a value in a control group that is not a root
    /// control group. Or, when using unified hierarchy, you tried to add a task in a leaf node.
    InvalidOperation,

    /// The path of the control group was invalid.
    ///
    /// This could be caused by trying to escape the control group filesystem via a string of "..".
    /// This crate checks against this and operations will fail with this error.
    InvalidPath,

    /// An unknown error has occured.
    Other,
}

/// The error type that can be returned from this crate, in the `Result::Err` variant.
/// The lower-level cause of this error can be obtained from the `source` method.
#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
    source: Option<Box<dyn StdError + Send + 'static>>,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let msg = match self.kind {
            ErrorKind::WriteFailed => "unable to write to a control group file",
            ErrorKind::ReadFailed => "unable to read a control group file",
            ErrorKind::ParseError => "unable to parse control group file",
            ErrorKind::InvalidOperation => "the requested operation is invalid",
            ErrorKind::InvalidPath => "the given path is invalid",
            ErrorKind::Other => "an unknown error",
        };

        write!(f, "{}", msg)
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self.source {
            Some(ref x) => Some(&**x),
            None => None,
        }
    }
}

impl Error {
    pub(crate) fn new(kind: ErrorKind) -> Self {
        Self { kind, source: None }
    }

    pub(crate) fn with_source<E>(kind: ErrorKind, source: E) -> Self
    where
        E: StdError + Send + 'static,
    {
        Self {
            kind,
            source: Some(Box::new(source)),
        }
    }

    pub fn kind(&self) -> &ErrorKind {
        &self.kind
    }
}

pub type Result<T> = std::result::Result<T, Error>;
