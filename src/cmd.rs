use crate::object::{Blob, GitObject};
use crate::Git;
use libflate::zlib::Decoder;
use std::env;
use std::fs::File;
use std::io::{self, Read};

pub fn cat_file_p(hash: String) -> io::Result<GitObject> {
    let (sub_dir, file) = hash.split_at(2);
    let path = format!(".git/objects/{}/{}", sub_dir, file);

    let mut file = File::open(path)?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;

    let mut d = Decoder::new(&buf[..])?;
    let mut buf = Vec::new();
    d.read_to_end(&mut buf)?;

    GitObject::new(&buf).ok_or(io::Error::from(io::ErrorKind::InvalidData))
}

pub fn hash_object(path: String) -> io::Result<Blob> {
    let mut file = File::open(path)?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;

    Blob::from(&buf).ok_or(io::Error::from(io::ErrorKind::InvalidData))
}

pub fn add(git: Git, filename: String) -> io::Result<()> {
    let path = env::current_dir().map(|x| x.join(&filename))?;
    let mut file = File::open(path)?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)?;

    // git hash-object -w path
    let blob = git.hash_object(&bytes).map(GitObject::Blob)?;
    Git::write_object(&blob)?;

    // git update-index --add --cacheinfo <mode> <hash> <name>
    let index = git.update_index(&blob.calc_hash(), filename)?;
    git.write_index(&index)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    #[should_panic(expected = "byte index 2 is out of bounds of ``")]
    fn cmd_cat_file_p_panic() {
        assert!(cat_file_p(String::from("")).is_err());
    }

    #[test]
    fn cmd_cat_file_p() {
        // file not found
        assert!(cat_file_p(String::from("hoge123...;;;")).is_err());

        // first commit
        let r = cat_file_p(String::from("01a0c85dd05755281466d29983dfcb15889e1a64"));
        assert!(r.is_ok());
        let r = r.ok().unwrap();
        let expected = "tree 179\u{0}tree 38b38f11af50240a2ddf643619e065408211e9e9\nauthor yusei-wy <yusei.kasa@gmail.com> 1609642799 +0900\ncomitter yusei-wy <yusei.kasa@gmail.com> 1609642799 +0900\n\nadd: blob object\n";
        assert_eq!(r.to_string(), expected);
    }

    #[test]
    fn cmd_hash_object() {
        assert!(hash_object(String::from("")).is_err());
        assert!(hash_object(String::from("hoge123...;;;")).is_err());

        let testfile = String::from("hash_object_test.txt");

        // test file
        let mut file = File::create(testfile.clone()).unwrap();
        let mut buf = "hello, git".as_bytes();
        file.write_all(&mut buf).unwrap();
        file.flush().unwrap();

        let blob = hash_object(testfile).unwrap();
        assert_eq!(
            hex::encode(blob.calc_hash()),
            "3edbc45b9a7f744c2345cd2cd073c3de091341ac"
        );
    }
}
