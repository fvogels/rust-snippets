use std::fs::{self, File};
use std::path::{Path};
use std::io::{self, BufRead};


#[derive(Debug)]
pub struct Segment
{
    pub caption: Option<String>,
    pub lines: Vec<String>,
}

pub fn load<P>(file_path: &P, is_separator: fn(&str) -> Option<&str>) -> io::Result<Vec<Segment>>
where P: AsRef<Path>
{
    let contents = fs::read_to_string(file_path)?;
    let lines = contents.lines();
    let segments = parse(lines, is_separator);

    Ok(segments)
}

pub fn parse<'a>(lines: impl Iterator<Item=&'a str>, is_separator: fn(&str) -> Option<&str>) -> Vec<Segment>
{
    let mut segments = Vec::new();
    let mut current_segment = Segment{caption: None, lines: Vec::new()};

    for line in lines {
        match is_separator(line) {
            Some(caption) => {
                segments.push(current_segment);
                current_segment = Segment{
                    caption: if caption.len() > 0 { Some(String::from(caption)) } else { None },
                    lines: Vec::new(),
                };
            },
            None => {
                current_segment.lines.push(line.to_owned());
            }
        }
    }

    segments.push(current_segment);

    segments
}