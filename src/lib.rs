pub mod cmd;
pub mod index;
pub mod object;

use crate::index::{Entry, Index};
use chrono::{TimeZone, Utc};
use libflate::zlib::Encoder;
use object::blob::Blob;
use object::GitObject;
use std::env;
use std::fs::{create_dir, File};
use std::io::{self, Read, Write};
use std::os::linux::fs::MetadataExt;

pub struct Git {}

impl Git {
    pub fn new() -> Self {
        Self {}
    }

    pub fn read_index(&self) -> io::Result<Vec<u8>> {
        let path = env::current_dir().map(|x| x.join(".git/index"))?;
        let mut file = File::open(path)?;
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)?;

        Ok(bytes)
    }

    fn write_object(object: &GitObject) -> io::Result<()> {
        let hash = hex::encode(object.calc_hash());
        let (sub_dir, file) = hash.split_at(2);

        let path = env::current_dir()?;
        let path = path.join(".git/objects").join(sub_dir);

        // ディレクトがなければ
        if let Err(_) = path.metadata() {
            create_dir(&path)?;
        }

        let path = path.join(file);

        let mut encoder = Encoder::new(Vec::new())?;
        encoder.write_all(&object.as_bytes())?;
        let bytes = encoder.finish().into_result()?;

        let mut file = File::create(path)?;
        file.write_all(&bytes)?;
        file.flush()?;

        Ok(())
    }

    pub fn write_index(&self, index: &Index) -> io::Result<()> {
        let mut file = File::create(".git/index")?;
        file.write_all(&index.as_bytes())?;
        file.flush()?;

        Ok(())
    }

    pub fn ls_files_stage(&self, bytes: &[u8]) -> io::Result<Index> {
        Index::from(&bytes).ok_or(io::Error::from(io::ErrorKind::InvalidData))
    }

    pub fn hash_object(&self, bytes: &[u8]) -> io::Result<Blob> {
        let blob = Blob::from(&bytes).ok_or(io::Error::from(io::ErrorKind::InvalidInput))?;
        Ok(blob)
    }

    pub fn update_index(&self, hash: &[u8], filename: String) -> io::Result<Index> {
        let bytes = self
            .read_index()
            // 初回には存在しないのでからの index ファイルのデータにする
            .unwrap_or([*b"DIRC", 0x0002u32.to_be_bytes(), 0x0000u32.to_be_bytes()].concat());
        let index = self.ls_files_stage(&bytes)?; // 現在の index を見る

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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ls_files_stage_index() {
        let git = Git::new();
        let bytes = git.read_index();
        assert!(bytes.is_ok());
        let index = bytes.and_then(|x| git.ls_files_stage(&x)).unwrap();
        assert!(index.to_string().len() > 0);
    }
}
