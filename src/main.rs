mod snippets;
mod util;

use std::path::PathBuf;

use clap::Parser;

use crate::snippets::snippets::{load_snippet_file, load_snippets};


#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct CommandLineArguments {
    /// Name of the person to greet
    #[arg(short, long)]
    name: String,

    /// Number of times to greet
    #[arg(short, long, default_value_t = 1)]
    count: u8,
}

fn main() -> anyhow::Result<()> {
    // let args = CommandLineArguments::parse();

    // let file_path = "../data/go/io/read-lines-in-file.snippet";
    // let snippet = load_snippet_file(&file_path)?;
    // println!("{:?}", snippet);

    let root = "../data";
    let snippets = load_snippets(&root)?;

    for snippet in snippets {
        println!("{:?}", snippet);
    }

    Ok(())
}
