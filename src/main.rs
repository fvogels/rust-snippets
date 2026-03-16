use chrono::Local;
use fern::Dispatch;

mod snippets;
mod util;
mod cli;
mod ui;
mod document;


fn main() {
    setup_logger().unwrap();

    cli::start()
}

fn setup_logger() -> Result<(), Box<dyn std::error::Error>> {
    Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{} [{}] {}",
                Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                message
            ))
        })
        .level(log::LevelFilter::Debug)
        .chain(fern::log_file("snippets.log")?)
        .apply()?;

    Ok(())
}