use std::fs::File;
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
    let file = File::open(file_path)?;
    let lines = io::BufReader::new(file).lines();
    let mut segments = Vec::new();
    let mut current_segment = Segment{caption: None, lines: Vec::new()};

    for line in lines {
        match line {
            Ok(line) => {
                match is_separator(line.as_str()) {
                    Some(caption) => {
                        segments.push(current_segment);
                        current_segment = Segment{
                            caption: if caption.len() > 0 { Some(String::from(caption)) } else { None },
                            lines: Vec::new(),
                        };
                    },
                    None => {
                        current_segment.lines.push(line);
                    }
                }
            },
            Err(err) => return Err(err),
        }
    }

    segments.push(current_segment);

    Ok(segments)
}