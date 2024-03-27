use std::{ops::Deref, str, fmt};

const JR_STR_MAX_LEN: usize = 30;

struct MiniStr {
    len: u8,
    data: [u8; JR_STR_MAX_LEN],
}

impl MiniStr {
    fn new(val: impl AsRef<str>) -> Self {
        let bytes = val.as_ref().as_bytes();
        let len = bytes.len();
        let mut data = [0u8; JR_STR_MAX_LEN];
        data[..len].copy_from_slice(bytes);

        Self {
            len: len as u8,
            data,
        }
    }
}

impl Deref for MiniStr {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        str::from_utf8(&self.data[..self.len as usize]).unwrap()
    }
}

impl fmt::Debug for MiniStr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.deref())
    }
}

#[derive(Debug)]
enum JrStr {
    Inline(MiniStr),
    Standard(String),
}

impl Deref for JrStr {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        match *self {
            JrStr::Inline(ref val) => val.deref(),
            JrStr::Standard(ref val) => val.deref(),
        }
    }
}

impl From<&str> for JrStr {
    fn from(value: &str) -> Self {
        match value.len() > JR_STR_MAX_LEN {
            true => Self::Standard(value.to_owned()),
            _ => Self::Inline(MiniStr::new(value)),
        }
    }
}

impl fmt::Display for JrStr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.deref())
    }
}

#[cfg(test)]
mod tests {
    use crate::jr_str::{JrStr, MiniStr};

    #[test]
    fn jr_str_works() {
        let len1 = std::mem::size_of::<JrStr>();
        let len2 = std::mem::size_of::<MiniStr>();
        println!("Len: JrStr {}, MiniStr {}", len1, len2);

        let s1: JrStr = "hello world".into();
        let s2: JrStr = "this is a long string with a length more than 30".into();

        println!("s1: {:?}, s2: {:?}", s1, s2);

        println!(
            "s1: {}({} bytes, {} chars), s2: {}({} bytes, {} chars)",
            s1, s1.len(), s1.chars().count(),
            s2, s2.len(), s2.chars().count(),
        );

        assert!(s1.ends_with("world"));
        assert!(s2.starts_with("this"));
    }
}
