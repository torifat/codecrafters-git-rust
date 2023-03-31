#[allow(unused_imports)]
use std::env;
#[allow(unused_imports)]
use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::io::Read;
use std::path::Path;

use clap::ArgAction;
use clap::{Arg, Command};
use flate2::read::ZlibDecoder;

fn cli() -> Command {
    Command::new("git")
        .subcommand(
            Command::new("init")
                .about("Create an empty Git repository or reinitialize an existing one"),
        )
        .subcommand(
            Command::new("cat-file")
                .about("Provide content or type and size information for repository objects")
                .arg(
                    Arg::new("pretty-print")
                        .help("Pretty-print the contents of <object> based on its type.")
                        .short('p')
                        .action(ArgAction::SetTrue),
                )
                .arg(Arg::new("object").required(true)),
        )
}

fn main() -> std::io::Result<()> {
    let matches = cli().get_matches();

    match matches.subcommand() {
        Some(("init", _sub_matches)) => {
            fs::create_dir(".git")
                .and_then(|()| fs::create_dir(".git/objects"))
                .and_then(|()| fs::create_dir(".git/refs"))
                .and_then(|()| fs::write(".git/HEAD", "ref: refs/heads/master\n"))?;
            println!("Initialized git directory");
            Ok(())
        }
        Some(("cat-file", sub_matches)) => {
            // println!("{:?}", sub_matches.get_flag("pretty-print"));
            // println!("{:?}", sub_matches.get_one::<String>("object"));

            sub_matches
                .get_one::<String>("object")
                .map(|hash| hash.split_at(2))
                .map(|(dir, file)| Path::new(".git/objects").join(dir).join(file))
                .map(|path| File::open(path).expect("Unable to read file"))
                .map(BufReader::new)
                .map(|buffered_input| {
                    let mut decoder = ZlibDecoder::new(buffered_input);

                    let mut decompressed_data = Vec::new();
                    decoder.read_to_end(&mut decompressed_data).unwrap();

                    // Find the null byte (0x00) and skip the header
                    let body_start = decompressed_data.iter().position(|&byte| byte == b'\x00');

                    if let Some(pos) = body_start {
                        print!("{}", String::from_utf8_lossy(&decompressed_data[pos + 1..]));
                            
                    } else {
                        eprintln!("No null byte found in the decompressed data. Cannot separate header from body.");
                    }
                });
            Ok(())
        }
        _ => unreachable!(),
    }
}
