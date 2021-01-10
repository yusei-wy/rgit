pub mod blob;
pub mod commit;
pub mod tree;

pub use blob::Blob;
pub use commit::Commit;
pub use tree::Tree;

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
}
