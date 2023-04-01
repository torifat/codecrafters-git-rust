use std::fs::{create_dir, write};

use anyhow::{Error, Result};
use clap::Parser;
use cli::{Cli, SubCommands};
use git_object::GitObject;

mod cli;
mod git_object;

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        SubCommands::Init => create_dir(".git")
            .and_then(|()| create_dir(".git/objects"))
            .and_then(|()| create_dir(".git/refs"))
            .and_then(|()| write(".git/HEAD", "ref: refs/heads/master\n"))
            .map(|()| println!("Initialized git directory"))
            .map_err(Error::from),
        SubCommands::CatFile {
            pretty_print: _,
            object,
        } => GitObject::new_from_object(&object)
            .and_then(|git_object| String::from_utf8(git_object.content).map_err(Error::from))
            .map(|content| print!("{}", content)),
        SubCommands::HashObject { write: _, file } => GitObject::new_from_file(&file)
            .and_then(|git_object| git_object.write())
            .map(|hash| println!("{}", hash)),
    }
}
