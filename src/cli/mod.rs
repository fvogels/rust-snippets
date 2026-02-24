use clap::{Parser, Subcommand};

use crate::{snippets::Library, ui::start_ui};


#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct CommandLineInterface {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Search {
        keywords: Vec<String>,
    },
    UI,
}

impl Commands {
    fn handle(&self) {
        match self {
            Self::Search { keywords } => search(keywords),
            Self::UI => start_ui(),
        }
    }
}

pub fn start() {
    CommandLineInterface::parse().command.handle();
}

fn search<'a>(keywords: &Vec<String>) {
    let library = Library::load(&"../data").unwrap();
    let snippets = library.search(keywords.iter().map(std::ops::Deref::deref));

    snippets.iter().copied().for_each(|snippet|
        println!("{}", snippet.description)
    );

    println!("{} snippets found", snippets.len())
}