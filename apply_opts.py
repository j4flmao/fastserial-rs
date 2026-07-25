import re

with open('d:/fastserial-rs/src/codec/json.rs', 'r', encoding='utf-8') as f:
    content = f.read()

read_bool_new = '''#[inline(always)]
pub fn read_bool(r: &mut ReadBuffer<'_>) -> Result<bool, Error> {
    skip_whitespace(r);
    let remaining = r.remaining_slice();
    if remaining.len() >= 5 {
        let p = remaining.as_ptr();
        let first = unsafe { *p };
        if first == b't' {
            let val = unsafe { core::ptr::read_unaligned(p as *const u32) };
            if val == u32::from_ne_bytes(*b"true") {
                r.pos += 4;
                return Ok(true);
            }
        } else if first == b'f' {
            let val = unsafe { core::ptr::read_unaligned(p.add(1) as *const u32) };
            if val == u32::from_ne_bytes(*b"alse") {
                r.pos += 5;
                return Ok(false);
            }
        }
    } else if remaining.len() >= 4 {
        if remaining == b"true" {
            r.pos += 4;
            return Ok(true);
        }
    }
    
    match r.peek() {
        b't' => {
            r.expect_bytes(b"true")?;
            Ok(true)
        }
        b'f' => {
            r.expect_bytes(b"false")?;
            Ok(false)
        }
        _b => Err(Error::UnexpectedByte),
    }
}'''

read_null_new = '''#[inline(always)]
pub fn read_null(r: &mut ReadBuffer<'_>) -> Result<(), Error> {
    skip_whitespace(r);
    let remaining = r.remaining_slice();
    if remaining.len() >= 4 {
        let val = unsafe { core::ptr::read_unaligned(remaining.as_ptr() as *const u32) };
        if val == u32::from_ne_bytes(*b"null") {
            r.pos += 4;
            return Ok(());
        }
    }
    r.expect_bytes(b"null")
}'''

quick_hash_new = '''#[inline(always)]
pub fn quick_hash(key: &[u8]) -> u32 {
    let mut hash: u32 = 5381;
    let mut i = 0;
    while i + 4 <= key.len() {
        let chunk = unsafe { core::ptr::read_unaligned(key.as_ptr().add(i) as *const u32) };
        let bytes = chunk.to_le_bytes();
        hash = hash.wrapping_mul(33).wrapping_add(bytes[0] as u32);
        hash = hash.wrapping_mul(33).wrapping_add(bytes[1] as u32);
        hash = hash.wrapping_mul(33).wrapping_add(bytes[2] as u32);
        hash = hash.wrapping_mul(33).wrapping_add(bytes[3] as u32);
        i += 4;
    }
    while i < key.len() {
        hash = hash.wrapping_mul(33).wrapping_add(unsafe { *key.get_unchecked(i) } as u32);
        i += 1;
    }
    hash
}'''

read_float_new = '''#[inline(always)]
pub fn read_float(r: &mut ReadBuffer<'_>) -> Result<f64, Error> {
    skip_whitespace(r);
    let start = r.pos;
    let data = &*r.data;

    if r.pos >= data.len() {
        return Err(Error::InvalidFloat);
    }
    
    let mut neg = false;
    if data[r.pos] == b'-' {
        neg = true;
        r.pos += 1;
    } else if data[r.pos] == b'+' {
        r.pos += 1;
    }

    let mut m: u64 = 0;
    let mut has_digits = false;
    let mut has_dot = false;
    let mut has_exp = false;
    let mut frac_digits = 0;
    let mut fast_path_possible = true;

    while r.pos < data.len() {
        let b = data[r.pos];
        if b.is_ascii_digit() {
            has_digits = true;
            if fast_path_possible {
                if let Some(next_m) = m.checked_mul(10).and_then(|v| v.checked_add((b - b'0') as u64)) {
                    m = next_m;
                    if has_dot {
                        frac_digits += 1;
                    }
                } else {
                    fast_path_possible = false;
                }
            }
            r.pos += 1;
        } else if b == b'.' && !has_dot {
            has_dot = true;
            r.pos += 1;
        } else if (b == b'e' || b == b'E') && !has_exp && has_digits {
            has_exp = true;
            fast_path_possible = false;
            r.pos += 1;
            if r.pos < data.len() && (data[r.pos] == b'+' || data[r.pos] == b'-') {
                r.pos += 1;
            }
        } else {
            break;
        }
    }

    if !has_digits || r.pos == start {
        return Err(Error::InvalidFloat);
    }

    if fast_path_possible && !has_exp && frac_digits <= 15 {
        let mut val = m as f64;
        if frac_digits > 0 {
            const POW10: [f64; 16] = [
                1.0, 10.0, 100.0, 1000.0, 10000.0, 100000.0, 1000000.0, 10000000.0,
                100000000.0, 1000000000.0, 10000000000.0, 100000000000.0, 1000000000000.0,
                10000000000000.0, 100000000000000.0, 1000000000000000.0
            ];
            val /= POW10[frac_digits as usize];
        }
        if neg {
            val = -val;
        }
        return Ok(val);
    }

    let slice = core::str::from_utf8(&data[start..r.pos]).map_err(|_| Error::InvalidUtf8)?;
    slice.parse::<f64>().map_err(|_| Error::InvalidFloat)
}'''

# Replace read_bool
content = re.sub(r'pub fn read_bool\(r: &mut ReadBuffer<\'_>\) -> Result<bool, Error> \{.*?\n\}', read_bool_new, content, flags=re.DOTALL)
content = re.sub(r'pub fn read_null\(r: &mut ReadBuffer<\'_>\) -> Result<\(\), Error> \{.*?\n\}', read_null_new, content, flags=re.DOTALL)

# Because #[inline(always)] might be above read_float, let's carefully replace read_float
content = re.sub(r'#\[inline\(always\)\]\s+pub fn read_float\(r: &mut ReadBuffer<\'_>\) -> Result<f64, Error> \{.*?\n\}', read_float_new, content, flags=re.DOTALL)

# Replace quick_hash (might not be public)
content = re.sub(r'#\[inline\(always\)\]\s+fn quick_hash\(key: &\[u8\]\) -> u32 \{.*?\n\}', quick_hash_new.replace("pub fn", "fn"), content, flags=re.DOTALL)
content = re.sub(r'#\[inline\(always\)\]\s+pub fn quick_hash\(key: &\[u8\]\) -> u32 \{.*?\n\}', quick_hash_new, content, flags=re.DOTALL)

with open('d:/fastserial-rs/src/codec/json.rs', 'w', encoding='utf-8') as f:
    f.write(content)
