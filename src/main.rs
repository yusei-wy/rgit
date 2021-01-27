use rgit::{
    cmd,
    fs::{linux::LinuxFileSystem, FileSystem},
    Git,
};
use std::io;

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let fs = LinuxFileSystem::init()?;
    let mut git = Git::new(fs);

    let sub_cmd = args.get(1).unwrap().clone();
    match sub_cmd.as_str() {
        "cat-file" => {
            let obj = cmd::cat_file_p(args.get(2).unwrap().clone())?;
            println!("{}", obj);
            Ok(())
        }
        "hash-object" => {
            let blob = cmd::hash_object(args.get(2).unwrap().clone())?;
            println!("{}", hex::encode(blob.calc_hash()));
            Ok(())
        }
        "add" => {
            let bytes = git.filesystem.read(args.get(2).unwrap().clone())?;
            cmd::add(&mut git, args.get(2).unwrap().clone(), &bytes)
        }
        "commit" => cmd::commit(&mut git, args.get(2).unwrap().clone()),
        _ => {
            eprintln!("unexpected command: {}", sub_cmd.as_str());
            Ok(())
        }
    }
}
