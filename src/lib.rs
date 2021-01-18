use std::env;
use std::fs::File;
use std::io::{self, Read};

pub mod cmd;
pub mod index;
pub mod object;

pub fn read_index() -> io::Result<Vec<u8>> {
    let path = env::current_dir().map(|x| x.join(".git/index"))?;
    let mut file = File::open(path)?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)?;

    Ok(bytes)
}

pub fn ls_files_stage(bytes: &[u8]) -> io::Result<index::Index> {
    index::Index::from(&bytes).ok_or(io::Error::from(io::ErrorKind::InvalidData))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ls_files_stage_index() {
        let bytes = read_index();
        assert!(bytes.is_ok());
        let index = bytes.and_then(|x| ls_files_stage(&x)).unwrap();
        assert!(index.to_string().len() > 0);
    }
}
