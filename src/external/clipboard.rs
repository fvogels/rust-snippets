use std::error::Error;

pub fn copy_to_clipboard(contents: &str) -> std::result::Result<(), Box<dyn Error>> {
    cli_clipboard::set_contents(contents.to_owned())
}
