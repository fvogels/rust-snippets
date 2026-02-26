use syntect::parsing::SyntaxSet;

mod snippets;
mod util;
mod cli;
mod ui;


fn main() {
    cli::start()


    // let syntax_set = SyntaxSet::load_defaults_newlines();

    // println!("syntaxes:");
    // syntax_set.syntaxes().iter().for_each(|s| println!("{}", s.name));
}
