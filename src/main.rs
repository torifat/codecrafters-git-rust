use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::io::Error;
use std::io::Read;
use std::path::Path;

use clap::Parser;
use cli::{Cli, SubCommands};
use flate2::read::ZlibDecoder;
use git_object::GitObject;

mod cli;
mod git_object;

fn main() -> std::io::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        SubCommands::Init => {
            fs::create_dir(".git")
                .and_then(|()| fs::create_dir(".git/objects"))
                .and_then(|()| fs::create_dir(".git/refs"))
                .and_then(|()| fs::write(".git/HEAD", "ref: refs/heads/master\n"))
                .map(|()| println!("Initialized git directory"))
        }
        SubCommands::CatFile {
            pretty_print: _,
            object,
        } => {
            Result::<&str, Error>::Ok(object.as_str())
                .map(|hash| hash.split_at(2))
                .map(|(dir, file)| Path::new(".git/objects").join(dir).join(file))
                .map(|path| File::open(path).expect("Unable to open the file"))
                .map(BufReader::new)
                .map(ZlibDecoder::new)
                .map(|mut decoder| {
                    let mut decompressed_data = Vec::new();
                    decoder.read_to_end(&mut decompressed_data).unwrap();
                    decompressed_data
                })
                .map(GitObject::from)
                .map(|git_object| {
                    println!("Object type: {:?}", git_object.object_type);
                    println!("Size: {}", git_object.size);

                    // Convert the content to a String and print it
                    if let Ok(content) = String::from_utf8(git_object.content) {
                        print!("Content: {}", content);
                    } else {
                        eprintln!("The object content is not valid UTF-8.");
                    }
                })
        }
    }
}
