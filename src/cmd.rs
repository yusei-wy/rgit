use crate::object::{Blob, GitObject};
use libflate::zlib::Decoder;
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
