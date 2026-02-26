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
    #[command(about, long_about = Some("Supported languages for syntax highlighting"))]
    Languages,
    #[command(about, long_about = Some("Supported themes for syntax highlighting"))]
    Themes,
}

impl Commands {
    fn handle(&self) {
        match self {
            Self::Search { keywords } => search(keywords),
            Self::UI => start_ui(),
            Self::Languages => list_syntax_highlighting_languages(),
            Self::Themes => list_syntax_highlighting_themes(),
        }
    }
}

pub fn start() {
    CommandLineInterface::parse().command.handle();
}

fn search<'a>(keywords: &Vec<String>) {
    let library = Library::load(&"../data/snippets").unwrap();
    let snippets = library.search(keywords.iter().map(std::ops::Deref::deref));

    snippets.iter().copied().for_each(|snippet|
        println!("{}", library.snippet(snippet).description)
    );

    println!("{} snippets found", snippets.len())
}

fn list_syntax_highlighting_languages() {
    let syntax_set = syntect::parsing::SyntaxSet::load_defaults_newlines();
    syntax_set.syntaxes().iter().for_each(|s| println!("{}", s.name));
}

fn list_syntax_highlighting_themes() {
    syntect::highlighting::ThemeSet::load_defaults().themes.into_keys().for_each(|theme| {
        println!("{}", theme);
    });
}