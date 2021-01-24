use super::ObjectType;
use std::fmt;

use sha1::{Digest, Sha1};

pub struct Tree {
    pub contents: Vec<File>,
}

impl Tree {
    pub fn new(contents: Vec<File>) -> Self {
        Self { contents }
    }

    pub fn from(bytes: &[u8]) -> Option<Self> {
        let contents: Vec<File> = Vec::new();
        let mut iter = bytes.split(|&b| b == b'\0'); // 各 Entry は '\0' 区切り

        let mut header = iter.next()?; // 一番最初の header を取り出し
        let contents = iter.try_fold(contents, |mut acc, x| {
            let (hash, next_header) = x.split_at(20); // hash 値は 20bytes
            let file = File::from(header, hash)?;

            acc.push(file);
            header = next_header;
            Some(acc)
        })?;

        Some(Self { contents })
    }

    pub fn calc_hash(&self) -> Vec<u8> {
        Vec::from(Sha1::digest(&self.as_bytes()).as_slice())
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let content: Vec<u8> = self.contents.iter().flat_map(|x| x.encode()).collect();
        let header = format!("{} {}\0", ObjectType::Tree.to_string(), content.len());

        [header.as_bytes(), content.as_slice()].concat()
    }
}

impl fmt::Display for Tree {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            (&self.contents)
                .into_iter()
                .map(|f| format!("{}", f))
                .collect::<Vec<_>>()
                .join("\n")
        )
    }
}

pub struct File {
    pub mode: usize,
    pub name: String,
    pub hash: Vec<u8>,
}

impl File {
    pub fn new(mode: usize, name: String, hash: &[u8]) -> Self {
        Self {
            mode,
            name,
            hash: hash.to_vec(),
        }
    }

    pub fn from(header: &[u8], hash: &[u8]) -> Option<Self> {
        let split_header = String::from_utf8(header.to_vec()).ok()?;

        let mut iter = split_header.split_whitespace();

        let mode = iter.next().and_then(|x| x.parse::<usize>().ok())?;
        let name = iter.next()?;

        Some(Self::new(mode, String::from(name), hash))
    }

    pub fn encode(&self) -> Vec<u8> {
        let header = format!("{} {}\0", self.mode, self.name);
        [header.as_bytes(), &self.hash].concat()
    }
}

impl fmt::Display for File {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{:>06} ??? {}\t{}",
            self.mode,
            hex::encode(&self.hash),
            self.name
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_new() {
        let f = File::new(0, String::from(""), b"");
        assert_eq!(f.mode, 0);
        assert_eq!(f.name, "");
        assert_eq!(f.hash, []);

        let f = File::new(040000, String::from("hello"), b"hello");
        assert_eq!(f.mode, 040000);
        assert_eq!(f.name, "hello");
        assert_eq!(f.hash, b"hello".to_vec());
    }

    #[test]
    fn file_from() {
        let f = File::from(b"", b"");
        assert!(f.is_none());

        // TODO: hash の例として正しいのかわからない
        let hash = b"11a8200b08ffa1abdc05cd9195ca7af639ce8946";
        let of = File::from(b"040000 test.txt hash", hash);
        let f = of.unwrap();
        assert_eq!(f.mode, 040000);
        assert_eq!(f.name, "test.txt");
        assert_eq!(f.hash, hash.to_vec());
    }

    #[test]
    fn file_encode() {
        let mode = 040000;
        let name = String::from("test.txt");
        // TODO: hash の例として正しいのかわからない
        let hash = b"11a8200b08ffa1abdc05cd9195ca7af639ce8946";
        let header = format!("{} {}\0", mode, name);

        let f = File::new(mode, name, hash);
        assert_eq!(f.encode(), [header.as_bytes(), hash].concat());
    }

    #[test]
    fn file_to_string() {
        let mode = 040000;
        let name = String::from("test.txt");
        let hash = b"aaaaaaaaaaaaaaaaaaaa";
        let f = File::new(mode, name.clone(), hash);
        assert_eq!(
            f.to_string(),
            format!("{:>06} ??? {}\t{}", mode, hex::encode(&hash), name)
        );
    }

    #[test]
    fn tree_from() {
        let ot = Tree::from(b"");
        assert!(ot.is_some());
        let t = ot.unwrap();
        assert_eq!(t.contents.len(), 0);

        let ot = Tree::from(b"040000 test.txt");
        assert!(ot.is_some());
        let t = ot.unwrap();
        assert_eq!(t.contents.len(), 0);

        let t = Tree::from(b"040000 test.txt-aaaaaaaaaaaaaaaaaaaa").unwrap();
        assert_eq!(t.contents.len(), 0);

        let t = Tree::from(b"040000 test.txt\0aaaaaaaaaaaaaaaaaaaa").unwrap();
        assert_eq!(t.contents.len(), 1);

        let t = Tree::from(
            b"040000 test.txt\0aaaaaaaaaaaaaaaaaaaa040000 test.txt\0bbbbbbbbbbbbbbbbbbbb",
        )
        .unwrap();
        assert_eq!(t.contents.len(), 2);
    }

    #[test]
    fn tree_as_bytes() {
        let mode = 040000;
        let name = "test.txt";
        let hash: &[u8] = b"aaaaaaaaaaaaaaaaaaaa";
        let content: Vec<u8> = [format!("{} {}\0", mode, name).as_bytes(), hash].concat();
        let t = Tree::from(b"040000 test.txt\0aaaaaaaaaaaaaaaaaaaa").unwrap();
        assert_eq!(
            t.as_bytes(),
            [
                format!("tree {}\0", content.len()).as_bytes(),
                content.as_slice(),
            ]
            .concat()
        );
    }

    #[test]
    fn tree_to_string() {
        let mode = 040000;
        let name = String::from("test.txt");
        let hash = b"aaaaaaaaaaaaaaaaaaaa";
        let t = Tree::from(
            b"040000 test.txt\0aaaaaaaaaaaaaaaaaaaa040000 test.txt\0aaaaaaaaaaaaaaaaaaaa",
        )
        .unwrap();
        assert_eq!(
            t.to_string(),
            format!("{:>06} ??? {}\t{}", mode, hex::encode(&hash), name)
                + &format!("\n{:>06} ??? {}\t{}", mode, hex::encode(&hash), name)
        );
    }
}
