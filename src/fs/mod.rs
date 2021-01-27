pub mod linux;

use std::io;

pub trait FileSystem {
    fn read(&self, path: String) -> io::Result<Vec<u8>>;
    fn write(&mut self, path: String, data: &[u8]) -> io::Result<()>;
    fn stat(&self, path: String) -> io::Result<Metadata>;
    fn create_dir(&self, path: String) -> io::Result<()>;
    fn rename(&mut self, from: String, to: String) -> io::Result<()>;
    fn remove(&mut self, path: String) -> io::Result<()>;
}

pub struct Metadata {
    pub dev: u32,
    pub ino: u32,
    pub mode: u32,
    pub uid: u32,
    pub gid: u32,
    pub size: u32,
    pub mtime: u32,
    pub mtime_nsec: u32,
    pub ctime: u32,
    pub ctime_nsec: u32,
}
