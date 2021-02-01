pub mod blob;
pub mod commit;
pub mod tree;

use blob::Blob;
use commit::Commit;
#[cfg(feature = "json")]
use serde::ser::{Serialize, SerializeStruct, Serializer};
use std::error::Error;
use std::fmt;
use std::result::Result;
use tree::Tree;

pub enum GitObject {
    Blob(Blob),
    Tree(Tree),
    Commit(Commit),
}

impl GitObject {
    pub fn new(bytes: &[u8]) -> Option<Self> {
        let mut iter = bytes.splitn(2, |&bytes| bytes == b'\0'); // Tree で "\0" を使っている部分があるので header と body の2つに分割する

        let obj_type = iter
            .next()
            .and_then(|x| String::from_utf8(x.to_vec()).ok())
            .and_then(|x| ObjectType::from(&x))?;

        match obj_type {
            ObjectType::Blob => Blob::from(bytes).map(Self::Blob),
            ObjectType::Tree => Tree::from(bytes).map(Self::Tree),
            ObjectType::Commit => Commit::from(bytes).map(Self::Commit),
        }
    }

    pub fn calc_hash(&self) -> Vec<u8> {
        match self {
            Self::Blob(obj) => obj.calc_hash(),
            Self::Tree(obj) => obj.calc_hash(),
            Self::Commit(obj) => obj.calc_hash(),
        }
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        match self {
            Self::Blob(obj) => obj.as_bytes(),
            Self::Tree(obj) => obj.as_bytes(),
            Self::Commit(obj) => obj.as_bytes(),
        }
    }
}

#[cfg(feature = "json")]
impl Serialize for GitObject {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_struct("GitObject", 2)?;
        match self {
            GitObject::Blob(blob) => s.serialize_field("Blob", blob)?,
            GitObject::Tree(tree) => s.serialize_field("Tree", tree)?,
            GitObject::Commit(commit) => s.serialize_field("Commit", commit)?,
        }
        s.serialize_field("hash", &hex::encode(self.calc_hash()))?;
        s.end()
    }
}

impl fmt::Display for GitObject {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Blob(obj) => obj.fmt(f),
            Self::Tree(obj) => obj.fmt(f),
            Self::Commit(obj) => obj.fmt(f),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum ObjectType {
    Blob,
    Tree,
    Commit,
}

impl ObjectType {
    pub fn from(s: &str) -> Option<Self> {
        let mut header = s.split_whitespace();

        match header.next()? {
            "blob" => Some(ObjectType::Blob),
            "tree" => Some(ObjectType::Tree),
            "commit" => Some(ObjectType::Commit),
            _ => None,
        }
    }

    pub fn to_string(self) -> String {
        match self {
            ObjectType::Blob => String::from("blob"),
            ObjectType::Tree => String::from("tree"),
            ObjectType::Commit => String::from("commit"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sha1::{Digest, Sha1};

    #[test]
    fn object_type_from() {
        assert_eq!(ObjectType::from(""), None);
        assert_eq!(ObjectType::from("hoge"), None);
        assert_eq!(ObjectType::from("123"), None);
        assert_eq!(ObjectType::from("blob"), Some(ObjectType::Blob));
        assert_eq!(ObjectType::from("tree"), Some(ObjectType::Tree));
        assert_eq!(ObjectType::from("commit"), Some(ObjectType::Commit));
    }

    #[test]
    fn object_to_string() {
        assert_eq!(ObjectType::from("blob").unwrap().to_string(), "blob");
        assert_eq!(ObjectType::from("tree").unwrap().to_string(), "tree");
        assert_eq!(ObjectType::from("commit").unwrap().to_string(), "commit");
    }

    #[test]
    fn git_object_new() {
        assert!(GitObject::new(b"").is_none());
        assert!(GitObject::new(b"hoge").is_none());
        assert!(GitObject::new(b"123").is_none());
        assert!(GitObject::new(b"blob").is_some());
        assert!(GitObject::new(b"tree").is_some());
        assert!(GitObject::new(b"commit").is_none()); // commit はこれだけだと from で None になる
        let (g, _) = new_commit_git_object();
        assert!(g.is_some());
    }

    #[test]
    fn git_object_as_bytes() {
        assert_eq!(
            GitObject::new(b"blob").unwrap().as_bytes(),
            format!("blob 4\0blob").as_bytes()
        );
        assert_eq!(
            GitObject::new(b"tree").unwrap().as_bytes(),
            format!("tree 0\0").as_bytes()
        );

        let (g, expected) = new_commit_git_object();
        assert_eq!(
            g.unwrap().as_bytes(),
            format!("commit {}\0{}\n", expected.len() + 1, expected).as_bytes(),
        );
    }

    #[test]
    fn git_object_calc_hash() {
        assert_eq!(
            GitObject::new(b"blob").unwrap().calc_hash(),
            calc_hash(format!("blob 4\0blob").as_bytes())
        );
        assert_eq!(
            GitObject::new(b"tree").unwrap().calc_hash(),
            calc_hash(format!("tree 0\0").as_bytes())
        );

        let (g, expected) = new_commit_git_object();
        assert_eq!(
            g.unwrap().calc_hash(),
            calc_hash(format!("commit {}\0{}\n", expected.len() + 1, expected).as_bytes()),
        );
    }

    fn new_commit_git_object() -> (Option<GitObject>, String) {
        let cs = vec![
            "tree adb7e67378d99ab8125f156442999f187db3d1a3",
            "parent 01a0c85dd05755281466d29983dfcb15889e1a64",
            "author author <author@example.com> 1609642799 +0900",
            "comitter comitter <comitter@example.com> 1609642799 +0900",
            "",
            "second commit",
        ]
        .join("\n")
        .trim_end()
        .to_owned();
        let expected = format!("tree {}", cs.clone());
        (
            GitObject::new(format!("commit {}", cs.clone()).as_bytes()),
            expected,
        )
    }

    fn calc_hash(bytes: &[u8]) -> Vec<u8> {
        Vec::from(Sha1::digest(bytes).as_slice())
    }
}
