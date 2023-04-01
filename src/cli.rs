use clap::{Parser, Subcommand};

#[derive(Parser)]
pub struct Cli {
    #[command(subcommand)]
    pub command: SubCommands,
}

#[derive(Subcommand)]
pub enum SubCommands {
    /// Create an empty Git repository or reinitialize an existing one
    Init,

    /// Provide content or type and size information for repository objects
    CatFile {
        /// Pretty-print the contents of <object> based on its type.
        #[arg(short)]
        pretty_print: bool,

        /// The name of the object to show.
        object: String,
    },

    /// Compute object ID and optionally creates a blob from a file
    HashObject {
        /// Actually write the object into the object database.
        #[arg(short)]
        write: bool,

        /// The path to the file to hash.
        file: String,
    },

    /// List the contents of a tree object
    LsTree {
        /// List only filenames (instead of the "long" output), one per line. Cannot be combined with --object-only.
        #[arg(long)]
        name_only: bool,

        /// Id of a tree-ish.
        tree_ish: String,
    },
}
