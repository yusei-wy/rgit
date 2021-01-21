use crate::{
    index::{Entry, Index},
    object::{Blob, GitObject},
};
use chrono::{TimeZone, Utc};
use libflate::zlib::{Decoder, Encoder};
use std::env;
use std::fs::{create_dir, File};
use std::io::{self, Read, Write};
use std::os::linux::fs::MetadataExt;

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

pub fn write_object(object: &GitObject) -> io::Result<()> {
    let hash = hex::encode(object.calc_hash());
    let (sub_dir, file) = hash.split_at(2);

    let path = env::current_dir()?;
    let path = path.join(".git/objects").join(sub_dir);

    // ディレクトがなければ
    if let Err(_) = path.metadata() {
        create_dir(&path)?;
    }

    let path = path.join(file);

    let encoder = Encoder::new(Vec::new())?;
    let bytes = encoder.finish().into_result()?;

    let mut file = File::create(path)?;
    file.write_all(&bytes)?;
    file.flush()?;

    Ok(())
}

pub fn write_index(index: &Index) -> io::Result<()> {
    let mut file = File::create(".git/index")?;
    file.write_all(&index.as_bytes())?;
    file.flush()?;

    Ok(())
}

pub fn read_index() -> io::Result<Vec<u8>> {
    let path = env::current_dir().map(|x| x.join(".git/index"))?;
    let mut file = File::open(path)?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)?;

    Ok(bytes)
}

pub fn update_index(hash: &[u8], filename: String) -> io::Result<Index> {
    let bytes = read_index()
        // 初回には存在しないのでからの index ファイルのデータにする
        .unwrap_or([*b"DIRC", 0x0002u32.to_be_bytes(), 0x0000u32.to_be_bytes()].concat());
    let index = ls_files_stage(&bytes)?; // 現在の index を見る

    let metadata = env::current_dir().and_then(|x| x.join(&filename).metadata())?;
    let entry = Entry::new(
        Utc.timestamp(metadata.st_ctime(), metadata.st_ctime_nsec() as u32),
        Utc.timestamp(metadata.st_mtime(), metadata.st_mtime_nsec() as u32),
        metadata.st_dev() as u32,
        metadata.st_ino() as u32,
        metadata.st_mode(),
        metadata.st_uid(),
        metadata.st_gid(),
        metadata.st_size() as u32,
        Vec::from(hash),
        filename.clone(),
    );

    let mut entries: Vec<Entry> = index
        .entries
        .into_iter()
        // ファイル名が同じまたは hash 値が同じ場合, 同一ファイルなので取り除く
        .filter(|x| x.name != entry.name && x.hash != entry.hash)
        .collect();
    entries.push(entry);

    Ok(Index::new(entries))
}

pub fn ls_files_stage(bytes: &[u8]) -> io::Result<Index> {
    Index::from(&bytes).ok_or(io::Error::from(io::ErrorKind::InvalidData))
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

    #[test]
    fn ls_files_stage_index() {
        let bytes = read_index();
        assert!(bytes.is_ok());
        let index = bytes.and_then(|x| ls_files_stage(&x)).unwrap();
        assert!(index.to_string().len() > 0);
    }
}
