use crate::{Decode, Encode, Error, io};
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Null,
    Bool(bool),
    Number(Number),
    String(String),
    Array(Vec<Value>),
    Object(BTreeMap<String, Value>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Number {
    U64(u64),
    I64(i64),
    F64(f64),
}

impl Number {
    pub fn as_u64(&self) -> Option<u64> {
        match *self {
            Number::U64(n) => Some(n),
            Number::I64(n) if n >= 0 => Some(n as u64),
            _ => None,
        }
    }

    pub fn as_i64(&self) -> Option<i64> {
        match *self {
            Number::I64(n) => Some(n),
            Number::U64(n) if n <= i64::MAX as u64 => Some(n as i64),
            _ => None,
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match *self {
            Number::F64(n) => Some(n),
            Number::U64(n) => Some(n as f64),
            Number::I64(n) => Some(n as f64),
        }
    }
}

impl Value {
    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null)
    }

    pub fn is_bool(&self) -> bool {
        matches!(self, Value::Bool(_))
    }

    pub fn is_number(&self) -> bool {
        matches!(self, Value::Number(_))
    }

    pub fn is_string(&self) -> bool {
        matches!(self, Value::String(_))
    }

    pub fn is_array(&self) -> bool {
        matches!(self, Value::Array(_))
    }

    pub fn is_object(&self) -> bool {
        matches!(self, Value::Object(_))
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_u64(&self) -> Option<u64> {
        match self {
            Value::Number(n) => n.as_u64(),
            _ => None,
        }
    }

    pub fn as_i64(&self) -> Option<i64> {
        match self {
            Value::Number(n) => n.as_i64(),
            _ => None,
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Value::Number(n) => n.as_f64(),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<&Vec<Value>> {
        match self {
            Value::Array(a) => Some(a),
            _ => None,
        }
    }

    pub fn as_object(&self) -> Option<&BTreeMap<String, Value>> {
        match self {
            Value::Object(o) => Some(o),
            _ => None,
        }
    }
}

impl Encode for Value {
    const SCHEMA_HASH: u64 = 0x56414C5545;

    fn encode<W: io::WriteBuffer>(&self, w: &mut W) -> Result<(), Error> {
        match self {
            Value::Null => w.write_bytes(b"null"),
            Value::Bool(b) => w.write_bytes(if *b { b"true" } else { b"false" }),
            Value::Number(n) => match n {
                Number::U64(v) => crate::codec::json::write_u64(*v, w),
                Number::I64(v) => crate::codec::json::write_i64(*v, w),
                Number::F64(v) => crate::codec::json::write_f64(*v, w),
            },
            Value::String(s) => crate::codec::json::write_str(s, w),
            Value::Array(arr) => {
                w.write_byte(b'[')?;
                let mut iter = arr.iter();
                if let Some(item) = iter.next() {
                    item.encode(w)?;
                    for item in iter {
                        w.write_byte(b',')?;
                        item.encode(w)?;
                    }
                }
                w.write_byte(b']')
            }
            Value::Object(map) => {
                w.write_byte(b'{')?;
                let mut first = true;
                for (k, v) in map {
                    if !first {
                        w.write_byte(b',')?;
                    }
                    first = false;
                    crate::codec::json::write_str(k, w)?;
                    w.write_byte(b':')?;
                    v.encode(w)?;
                }
                w.write_byte(b'}')
            }
        }
    }
}

impl<'de> Decode<'de> for Value {
    fn decode(r: &mut io::ReadBuffer<'de>) -> Result<Self, Error> {
        crate::codec::json::skip_whitespace(r);
        match r.peek() {
            b'"' => {
                let s = crate::codec::json::read_string_cow(r)?.into_owned();
                Ok(Value::String(s))
            }
            b'{' => {
                r.advance(1);
                let mut map = BTreeMap::new();
                crate::codec::json::skip_whitespace(r);
                if r.peek() == b'}' {
                    r.advance(1);
                    return Ok(Value::Object(map));
                }
                loop {
                    crate::codec::json::skip_whitespace(r);
                    let key = crate::codec::json::read_string_cow(r)?.into_owned();
                    crate::codec::json::skip_whitespace(r);
                    r.expect_byte(b':')?;
                    let val = Value::decode(r)?;
                    map.insert(key, val);
                    crate::codec::json::skip_comma_or_close(r, b'}')?;
                    if r.peek() == b'}' {
                        r.advance(1);
                        break;
                    }
                }
                Ok(Value::Object(map))
            }
            b'[' => {
                r.advance(1);
                let mut arr = Vec::new();
                crate::codec::json::skip_whitespace(r);
                if r.peek() == b']' {
                    r.advance(1);
                    return Ok(Value::Array(arr));
                }
                loop {
                    arr.push(Value::decode(r)?);
                    crate::codec::json::skip_comma_or_close(r, b']')?;
                    if r.peek() == b']' {
                        r.advance(1);
                        break;
                    }
                }
                Ok(Value::Array(arr))
            }
            b't' => {
                r.expect_bytes(b"true")?;
                Ok(Value::Bool(true))
            }
            b'f' => {
                r.expect_bytes(b"false")?;
                Ok(Value::Bool(false))
            }
            b'n' => {
                r.expect_bytes(b"null")?;
                Ok(Value::Null)
            }
            b'0'..=b'9' | b'-' => {
                let start = r.get_pos();
                let negative = r.peek() == b'-';
                if negative {
                    r.advance(1);
                }
                while r.get_pos() < r.data.len() && r.data[r.get_pos()].is_ascii_digit() {
                    r.advance(1);
                }
                let has_frac_or_exp = r.peek() == b'.' || r.peek() == b'e' || r.peek() == b'E';
                if has_frac_or_exp {
                    // Reset and parse as float
                    r.pos = start;
                    let f = crate::codec::json::read_float(r)?;
                    Ok(Value::Number(Number::F64(f)))
                } else if negative {
                    let slice = core::str::from_utf8(&r.data[start..r.get_pos()])
                        .map_err(|_| Error::InvalidUtf8 { byte_offset: start })?;
                    let n: i64 = slice
                        .parse()
                        .map_err(|_| Error::NumberOverflow { type_name: "i64" })?;
                    Ok(Value::Number(Number::I64(n)))
                } else {
                    let slice = core::str::from_utf8(&r.data[start..r.get_pos()])
                        .map_err(|_| Error::InvalidUtf8 { byte_offset: start })?;
                    let n: u64 = slice
                        .parse()
                        .map_err(|_| Error::NumberOverflow { type_name: "u64" })?;
                    Ok(Value::Number(Number::U64(n)))
                }
            }
            b => Err(Error::UnexpectedByte {
                expected: "value",
                got: b,
                offset: r.get_pos(),
            }),
        }
    }
}

impl core::fmt::Display for Value {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Value::Null => f.write_str("null"),
            Value::Bool(true) => f.write_str("true"),
            Value::Bool(false) => f.write_str("false"),
            Value::Number(Number::U64(n)) => write!(f, "{}", n),
            Value::Number(Number::I64(n)) => write!(f, "{}", n),
            Value::Number(Number::F64(n)) => {
                if n.fract() == 0.0 && n.is_finite() && n.abs() < (1u64 << 53) as f64 {
                    write!(f, "{}.0", *n as i64)
                } else {
                    write!(f, "{}", ryu::Buffer::new().format(*n))
                }
            }
            Value::String(s) => {
                f.write_str("\"")?;
                for ch in s.chars() {
                    match ch {
                        '"' => f.write_str("\\\"")?,
                        '\\' => f.write_str("\\\\")?,
                        '\n' => f.write_str("\\n")?,
                        '\r' => f.write_str("\\r")?,
                        '\t' => f.write_str("\\t")?,
                        c if (c as u32) < 0x20 => write!(f, "\\u{:04x}", c as u32)?,
                        c => write!(f, "{}", c)?,
                    }
                }
                f.write_str("\"")
            }
            Value::Array(arr) => {
                f.write_str("[")?;
                let mut first = true;
                for item in arr {
                    if !first {
                        f.write_str(",")?;
                    }
                    first = false;
                    write!(f, "{}", item)?;
                }
                f.write_str("]")
            }
            Value::Object(map) => {
                f.write_str("{")?;
                let mut first = true;
                for (k, v) in map {
                    if !first {
                        f.write_str(",")?;
                    }
                    first = false;
                    // Write key as JSON string
                    write!(f, "{}", Value::String(k.clone()))?;
                    f.write_str(":")?;
                    write!(f, "{}", v)?;
                }
                f.write_str("}")
            }
        }
    }
}

impl From<bool> for Value {
    fn from(v: bool) -> Self {
        Value::Bool(v)
    }
}

impl From<u64> for Value {
    fn from(v: u64) -> Self {
        Value::Number(Number::U64(v))
    }
}

impl From<i64> for Value {
    fn from(v: i64) -> Self {
        Value::Number(Number::I64(v))
    }
}

impl From<f64> for Value {
    fn from(v: f64) -> Self {
        Value::Number(Number::F64(v))
    }
}

impl From<String> for Value {
    fn from(v: String) -> Self {
        Value::String(v)
    }
}

impl From<&str> for Value {
    fn from(v: &str) -> Self {
        Value::String(String::from(v))
    }
}

impl From<Vec<Value>> for Value {
    fn from(v: Vec<Value>) -> Self {
        Value::Array(v)
    }
}

impl From<BTreeMap<String, Value>> for Value {
    fn from(v: BTreeMap<String, Value>) -> Self {
        Value::Object(v)
    }
}

impl From<()> for Value {
    fn from(_: ()) -> Self {
        Value::Null
    }
}
