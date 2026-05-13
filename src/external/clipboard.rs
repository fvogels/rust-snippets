use std::error::Error;

pub fn copy_to_clipboard<S: Into<String>>(contents: S) -> std::result::Result<(), Box<dyn Error>> {
    cli_clipboard::set_contents(contents.into())
}
