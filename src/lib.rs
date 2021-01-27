pub mod cmd;
pub mod fs;
pub mod index;
pub mod object;

use crate::index::{Entry, Index};
use chrono::{Local, TimeZone, Utc};
use fs::FileSystem;
use libflate::zlib::{Decoder, Encoder};
use object::{blob::Blob, Tree};
use object::{commit, GitObject};
use object::{tree, Commit};
use std::io::{self, Read, Write};

pub struct Git<F: FileSystem> {
    pub filesystem: F,
}

impl<F: FileSystem> Git<F> {
    pub fn new(filesystem: F) -> Self {
        Self { filesystem }
    }

    pub fn cat_file_p(&self, bytes: &[u8]) -> io::Result<GitObject> {
        let mut d = Decoder::new(&bytes[..])?;
        let mut buf = Vec::new();
        d.read_to_end(&mut buf)?;

        GitObject::new(&buf).ok_or(io::Error::from(io::ErrorKind::InvalidData))
    }

    pub fn read_index(&self) -> io::Result<Vec<u8>> {
        self.filesystem.read(".git/index".to_string())
    }

    pub fn write_index(&mut self, index: &Index) -> io::Result<()> {
        self.filesystem
            .write(".git/index".to_string(), &index.as_bytes())
    }

    pub fn read_object(&self, hash: String) -> io::Result<Vec<u8>> {
        let (sub_dir, file) = hash.split_at(2);
        self.filesystem
            .read(format!(".git/objects/{}/{}", sub_dir, file))
    }

    pub fn write_object(&mut self, object: &GitObject) -> io::Result<()> {
        let hash = hex::encode(object.calc_hash());
        let (sub_dir, file) = hash.split_at(2);

        let path = format!(".git/objects{}", sub_dir);
        // ディレクトがなければ
        if let Err(_) = self.filesystem.stat(path.clone()) {
            self.filesystem.create_dir(path.clone())?;
        }

        let path = format!("{}/{}", path, file);

        let mut encoder = Encoder::new(Vec::new())?;
        encoder.write_all(&object.as_bytes())?;
        let bytes = encoder.finish().into_result()?;

        self.filesystem.write(path, &bytes)
    }

    pub fn ls_files_stage(&self, bytes: &[u8]) -> io::Result<Index> {
        Index::from(&bytes).ok_or(io::Error::from(io::ErrorKind::InvalidData))
    }

    pub fn hash_object(&self, bytes: &[u8]) -> io::Result<Blob> {
        let blob = Blob::from(&bytes).ok_or(io::Error::from(io::ErrorKind::InvalidInput))?;
        Ok(blob)
    }

    pub fn update_index(&self, idx: Index, hash: &[u8], filename: String) -> io::Result<Index> {
        let metadata = self.filesystem.stat(filename.clone())?;
        let entry = Entry::new(
            Utc.timestamp(metadata.ctime as i64, metadata.ctime_nsec),
            Utc.timestamp(metadata.mtime as i64, metadata.mtime_nsec),
            metadata.dev,
            metadata.ino,
            metadata.mode,
            metadata.uid,
            metadata.gid,
            metadata.size,
            Vec::from(hash),
            filename.clone(),
        );

        let mut entries: Vec<Entry> = idx
            .entries
            .into_iter()
            // ファイル名が同じまたは hash 値が同じ場合, 同一ファイルなので取り除く
            .filter(|x| x.name != entry.name && x.hash != entry.hash)
            .collect();
        entries.push(entry);
        entries.sort_by(|a, b| a.name.cmp(&b.name));

        Ok(Index::new(entries))
    }

    pub fn write_tree(&self) -> io::Result<Tree> {
        let bytes = self.read_index()?;
        let index = self.ls_files_stage(&bytes)?;

        let contents = index
            .entries
            .iter()
            .map(|x| tree::File::new(100644, x.name.clone(), &x.hash)) // 今回はファイルにのみ対応するので mode は 100644 固定
            .collect::<Vec<_>>();

        Ok(Tree::new(contents))
    }

    pub fn commit_tree(
        &self,
        name: String,
        email: String,
        tree_hash: String,
        message: String,
    ) -> io::Result<Commit> {
        let parent = self.head_ref().and_then(|x| self.read_ref(x)).ok();
        let offset = {
            let local = Local::now();
            *local.offset()
        };
        let ts = offset.from_utc_datetime(&Utc::now().naive_utc());
        let author = commit::User::new(name.clone(), email.clone(), ts);
        let commit = Commit::new(tree_hash, parent, author.clone(), author.clone(), message);

        Ok(commit)
    }

    // .git/HEAD に書かれた ref を参照する
    fn head_ref(&self) -> io::Result<String> {
        let path = ".git/HEAD".to_string();
        let file = self.filesystem.read(path)?;
        let refs =
            String::from_utf8(file).map_err(|_| io::Error::from(io::ErrorKind::InvalidData))?;

        let (prefix, path) = refs.split_at(5);

        // 今回は `ref: xxx` のフォーマットのみ対応
        // `git checkout hash` で移動した際には hash 値が入っていありえます
        if prefix != "ref: " {
            return Err(io::Error::from(io::ErrorKind::InvalidData));
        }

        Ok(path.trim().to_string())
    }

    fn read_ref(&self, path: String) -> io::Result<String> {
        let path = format!(".git/{}", path);
        let file = self.filesystem.read(path)?;
        let hash =
            String::from_utf8(file).map_err(|_| io::Error::from(io::ErrorKind::InvalidData))?;

        Ok(hash.trim().to_string())
    }

    pub fn update_ref(&mut self, path: String, hash: &[u8]) -> io::Result<()> {
        self.write_ref(path, hash)
    }

    fn write_ref(&mut self, path: String, hash: &[u8]) -> io::Result<()> {
        let path = format!(".git/{}", path);
        self.filesystem.write(path, hex::encode(hash).as_bytes())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fs::linux::LinuxFileSystem;

    #[test]
    fn ls_files_stage_index() {
        let fs = LinuxFileSystem::init().unwrap();
        let git = Git::new(fs);
        let bytes = git.read_index();
        assert!(bytes.is_ok());
        let index = bytes.and_then(|x| git.ls_files_stage(&x)).unwrap();
        assert!(index.to_string().len() > 0);
    }
}
