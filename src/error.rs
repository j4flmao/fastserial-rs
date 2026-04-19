use core::fmt;

/// Error types for serialization and deserialization operations.
#[non_exhaustive]
pub enum Error {
    /// Unexpected end of input buffer.
    UnexpectedEof,
    /// Invalid UTF-8 sequence encountered.
    InvalidUtf8 {
        /// Byte offset where the invalid sequence was detected.
        byte_offset: usize,
    },
    /// An unexpected byte was encountered.
    UnexpectedByte {
        /// Description of what was expected.
        expected: &'static str,
        /// The byte that was actually found.
        got: u8,
        /// Byte offset where the unexpected byte was detected.
        offset: usize,
    },
    /// A numeric value overflowed its target type.
    NumberOverflow {
        /// The name of the target type (e.g., "u64", "i32").
        type_name: &'static str,
    },
    /// A required field is missing from the input.
    MissingField {
        /// The name of the missing field.
        name: &'static str,
    },
    /// An unknown field was encountered (if strict mode is enabled).
    UnknownField {
        /// The name of the unknown field.
        name: alloc::vec::Vec<u8>,
    },
    /// The output buffer is full.
    BufferFull {
        /// Number of bytes needed.
        needed: usize,
        /// Number of bytes available in the buffer.
        available: usize,
    },
    /// A floating point value is invalid (e.g., NaN or Infinity).
    InvalidFloat,
    /// The binary format magic number is invalid.
    InvalidMagic,
    /// Unsupported binary version.
    UnsupportedVersion {
        /// The unsupported version number found in the input.
        version: u16,
    },
    /// Extra data found after decoding the main value.
    TrailingData,
    /// A custom error from an external source.
    Custom(alloc::boxed::Box<dyn std::error::Error + Send + Sync>),
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedEof => write!(f, "Unexpected end of input"),
            Self::InvalidUtf8 { byte_offset } => {
                write!(f, "Invalid UTF-8 at offset {}", byte_offset)
            }
            Self::UnexpectedByte {
                expected,
                got,
                offset,
            } => {
                write!(
                    f,
                    "Unexpected byte at offset {}: expected {}, got {:#04x}",
                    offset, expected, got
                )
            }
            Self::NumberOverflow { type_name } => write!(f, "Number overflow for {}", type_name),
            Self::MissingField { name } => write!(f, "Missing required field: {}", name),
            Self::UnknownField { name } => {
                write!(
                    f,
                    "Unknown field: {:?}",
                    alloc::string::String::from_utf8_lossy(name)
                )
            }
            Self::BufferFull { needed, available } => {
                write!(
                    f,
                    "Buffer overflow: need {} bytes, have {}",
                    needed, available
                )
            }
            Self::InvalidFloat => write!(f, "Invalid float value (NaN or Infinity)"),
            Self::InvalidMagic => write!(f, "Invalid binary magic number"),
            Self::UnsupportedVersion { version } => {
                write!(f, "Unsupported binary version: {:#06x}", version)
            }
            Self::TrailingData => write!(f, "Extra data found after decoding the main value"),
            Self::Custom(_) => write!(f, "Custom error"),
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
    fn from(e: std::io::Error) -> Self {
        Error::Custom(alloc::boxed::Box::new(e))
    }
}

impl Error {
    pub fn missing_field(name: &'static str) -> Self {
        Error::MissingField { name }
    }
}
