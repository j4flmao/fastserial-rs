use core::fmt;

/// Compact error code for serialization and deserialization operations.
#[derive(Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Error {
    UnexpectedEof,
    InvalidUtf8,
    UnexpectedByte,
    NumberOverflow,
    MissingField,
    UnknownField,
    BufferFull,
    InvalidFloat,
    InvalidMagic,
    UnsupportedVersion,
    TrailingData,
    SchemaMismatch,
    EscapeInBorrowedString,
    Custom,
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedEof => write!(f, "UnexpectedEof"),
            Self::InvalidUtf8 => write!(f, "InvalidUtf8"),
            Self::UnexpectedByte => write!(f, "UnexpectedByte"),
            Self::NumberOverflow => write!(f, "NumberOverflow"),
            Self::MissingField => write!(f, "MissingField"),
            Self::UnknownField => write!(f, "UnknownField"),
            Self::BufferFull => write!(f, "BufferFull"),
            Self::InvalidFloat => write!(f, "InvalidFloat"),
            Self::InvalidMagic => write!(f, "InvalidMagic"),
            Self::UnsupportedVersion => write!(f, "UnsupportedVersion"),
            Self::TrailingData => write!(f, "TrailingData"),
            Self::SchemaMismatch => write!(f, "SchemaMismatch"),
            Self::EscapeInBorrowedString => write!(f, "EscapeInBorrowedString"),
            Self::Custom => write!(f, "Custom"),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

impl core::error::Error for Error {}

#[cfg(feature = "std")]
impl From<std::io::Error> for Error {
    fn from(_: std::io::Error) -> Self {
        Error::Custom
    }
}

impl Error {
    pub fn missing_field(_name: &'static str) -> Self {
        Error::MissingField
    }
    pub fn custom<T: core::fmt::Display>(_msg: T) -> Self {
        Error::Custom
    }
}
