use super::ObjectType;
use sha1::{Digest, Sha1};
use std::fmt;

pub struct Blob {
    pub size: usize,
    pub content: String,
}

impl Blob {
    pub fn new(content: String) -> Self {
        Self {
            size: content.len(),
            content,
        }
    }

    pub fn from(bytes: &[u8]) -> Option<Self> {
        let content = String::from_utf8(bytes.to_vec()).ok()?;
        Some(Self {
            size: content.len(),
            content,
        })
    }

    pub fn calc_hash(&self) -> Vec<u8> {
        Vec::from(Sha1::digest(&self.as_bytes()).as_slice())
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let header = format!("{} {}\0", ObjectType::Blob.to_string(), self.size);
        let store = format!("{}{}", header, self.to_string());
        Vec::from(store.as_bytes())
    }
}

impl fmt::Display for Blob {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new() {
        let b = Blob::new(String::from("hello"));

        assert_eq!(b.size, 5);
        assert_eq!(b.content, "hello");
    }

    #[test]
    fn from() {
        let ob = Blob::from(b"");
        assert!(ob.is_some());
        let b = ob.unwrap();
        assert_eq!(b.size, 0);
        assert_eq!(b.content, "");

        let ob = Blob::from(b"aaabbbccc");
        assert!(ob.is_some());
        let b = ob.unwrap();
        assert_eq!(b.size, 9);
        assert_eq!(b.content, "aaabbbccc");
    }

    #[test]
    fn as_bytes() {
        let ob = Blob::from(b"aaabbbccc");
        assert!(ob.is_some());
        let b = ob.unwrap();
        assert_eq!(b.size, 9);
        assert_eq!(b.content, "aaabbbccc");
        assert_eq!(b.as_bytes(), b"blob 9\0aaabbbccc");
    }

    #[test]
    fn calc_hash() {
        use sha1::{Digest, Sha1};
        let ob = Blob::from(b"aaabbbccc");
        let b = ob.unwrap();
        let hash = Vec::from(Sha1::digest(b"blob 9\0aaabbbccc").as_slice());
        assert_eq!(b.calc_hash(), hash);
    }

    #[test]
    fn to_string() {
        let ob = Blob::from(b"aaabbbccc");
        let b = ob.unwrap();
        assert_eq!(b.to_string(), "aaabbbccc");
    }
}
