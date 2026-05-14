use std::rc::Rc;

use log;
use clap::{Parser, Subcommand};

use crate::{document, snippets::{self, Library}, timing, ui};


#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct CommandLineInterface {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
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

        println!("Description: {}\n", snippet.description)
    }
}

fn create_archive() {
    let root = "../data/snippets";
    let archive_path = "./archive.bin";

    let (archive, duration) = timing::measure(|| {
        let archive = snippets::Archive::load_snippet_files(&root).unwrap();
        archive.write(&archive_path).unwrap();
        archive
    });

    println!("Generated archive: {} snippets in {}ms", archive.raw_snippets.len(), duration.as_millis());
}

fn load_library() -> Library {
    let archive_path =  "./archive.bin";
    let archive = snippets::Archive::load(&archive_path).unwrap();
    let syntax_highlighter = Rc::new(create_syntax_highlighter());
    Library::from_archive(archive, syntax_highlighter)
}

fn start_ui() {
    let library = {
        let (library, duration) = timing::measure(|| load_library());
        log::info!("Library loaded in {}ms", duration.as_millis());
        library
    };

    ui::start_ui(library);
}

fn create_syntax_highlighter() -> document::SyntaxHighlighter {
    let mut syntax_highlighter = document::SyntaxHighlighter::new();

    syntax_highlighter.add_alias("bash", "Bourne Again Shell (bash)");

    for language in syntax_highlighter.supported_languages() {
        syntax_highlighter.add_alias(language.to_lowercase().as_str(), language.as_str());
    }

    syntax_highlighter
}
