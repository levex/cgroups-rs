use std::error::Error as StdError;
use std::fmt;

/// The kinds of errors that can occur while operating on cgroups.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum ErrorKind {
    /// Failed to read from a cgroup file.
    ReadFailed,

    /// Failed to parse a string in a cgroup file into a value.
    /// In the future, there will be some information attached to this variant.
    ParseFailed,

    /// Failed to write to a cgroup file.
    WriteFailed,

    /// Failed to apply a value to a subsystem.
    ApplyFailed,

    /// You tried to do something invalid.
    ///
    /// This could be because you tried to set a value in a cgroup that is not a root
    /// cgroup. Or, when using unified hierarchy, you tried to add a task in a non-leaf node.
    InvalidOperation,

    /// The path of the cgroup was invalid.
    ///
    /// This could be caused by trying to escape the cgroup filesystem via a string of `..`.
    /// This crate checks against this and operations will fail with this error.
    InvalidPath,
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
            ErrorKind::ReadFailed => "unable to read a cgroup file",
            ErrorKind::ParseFailed => "unable to parse a string in a cgroup file",
            ErrorKind::WriteFailed => "unable to write to a cgroup file",
            ErrorKind::ApplyFailed => "unable to apply a value to a subsystem (controller)",
            ErrorKind::InvalidOperation => "the requested operation is invalid",
            ErrorKind::InvalidPath => "the given path is invalid",
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

    /// Returns the kind of this error.
    pub fn kind(&self) -> ErrorKind {
        self.kind
    }
}

pub type Result<T> = std::result::Result<T, Error>;
