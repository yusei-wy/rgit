pub mod blob;
pub mod commit;
pub mod tree;

pub use blob::Blob;
pub use commit::Commit;
pub use tree::Tree;

pub enum ObjectType {
    Blob,
    Tree,
    Commit,
}

impl ObjectType {
    pub fn to_string(self) -> String {
        match self {
            ObjectType::Blob => String::from("blob"),
            ObjectType::Tree => String::from("tree"),
            ObjectType::Commit => String::from("commit"),
        }
    }
}
