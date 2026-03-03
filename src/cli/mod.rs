use clap::{Parser, Subcommand};

use crate::{snippets::Library, ui};


#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct CommandLineInterface {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Foo {
        #[clap(subcommand)]
        subcommand: HighlightSubcommand
    },
    List,
    Search {
        keywords: Vec<String>,
    },
    UI,
    Highlight {
        #[clap(subcommand)]
        subcommand: HighlightSubcommand,
    },
    Archive,
}

#[derive(Subcommand, Debug)]
enum HighlightSubcommand {
    #[command(about = "List supported languages", long_about = None)]
    Languages,
    #[command(about = "List supported themes", long_about = None)]
    Themes
}

impl Commands {
    fn handle(&self) {
        match self {
            Self::Foo { subcommand } => println!("{:?}", subcommand),
            Self::List => list_snippets(),
            Self::Search { keywords } => search(keywords),
            Self::UI => start_ui(),
            Self::Highlight { subcommand } => {
                match *subcommand {
                    HighlightSubcommand::Languages => list_syntax_highlighting_languages(),
                    HighlightSubcommand::Themes => list_syntax_highlighting_themes(),
                }
            },
            Self::Archive => create_archive(),
        }
    }
}

pub fn start() {
    CommandLineInterface::parse().command.handle();
}

fn search<'a>(keywords: &Vec<String>) {
    let library = load_library();
    let tags: Vec<&str> = Vec::new();
    let snippets = library.search(keywords.iter().map(std::ops::Deref::deref), tags.into_iter());

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

fn list_snippets() {
    let library = load_library();
    let snippet_ids = library.snippets();

    for snippet_id in snippet_ids {
        let snippet = library.snippet(snippet_id);
        let path = snippet.path.join("/");

        println!("Description: {}\nPath: {}\n", snippet.description, path)
    }
}

fn create_archive() {
    let library = Library::load_files(&"../data/snippets").unwrap();
    if let Err(error) = library.write_to_archive(&"./archive.bin") {
        println!("Failure: {}", error);
    }
}

fn load_library() -> Library {
    Library::read_archive(&"./archive.bin").unwrap()
}

fn start_ui() {
    let library = load_library();

    ui::start_ui(library);
}