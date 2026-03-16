use std::{collections::HashMap, fmt, mem};

pub struct Attributes(Vec<(String, String)>);

impl Attributes {
    pub fn pairs(self) -> Vec<(String, String)> {
        self.0
    }

    pub fn as_hashmap(&self) -> HashMap<String, String> {
        let mut result = HashMap::new();

        for (key, value) in &self.0 {
            result.insert(key.clone(), value.clone());
        }

        result
    }
}

pub fn parse(s: &str) -> Result<Attributes, Error> {
    let mut pairs = Vec::new();
    let mut state = State::Idle;

    for char in s.chars() {
        let current_state = mem::replace(&mut state, State::Idle);

        match current_state {
            State::Idle => {
                match char {
                    ' ' => state = State::Idle,
                    '=' => return Err(Error::UnexpectedEndOfString),
                    c => state = State::ReadingKey(vec![c]),
                }
            },
            State::ReadingKey(key) => {
                match char {
                    ' ' => return Err(Error::InvalidKeyChar),
                    '=' => state = State::ReadingValue(key.into_iter().collect(), Vec::new()),
                    c => {
                        let mut key = key;
                        key.push(c);
                        state = State::ReadingKey(key);
                    }
                }
            },
            State::ReadingValue(key, value) => {
                match char {
                    ' ' => {
                        pairs.push((key, value.into_iter().collect()));
                        state = State::Idle
                    },
                    '"' => {
                        if value.len() == 0 {
                            state = State::ReadingQuotedValue(key, value)
                        }
                        else {
                            return Err(Error::InvalidValueChar)
                        }
                    },
                    c => {
                        let mut value = value;
                        value.push(c);
                        state = State::ReadingValue(key, value)
                    }
                }
            },
            State::ReadingQuotedValue(key, value) => {
                match char {
                    '"' => {
                        pairs.push((key, value.into_iter().collect()));
                        state = State::Idle
                    },
                    c => {
                        let mut value = value;
                        value.push(c);
                        state = State::ReadingQuotedValue(key, value)
                    }
                }
            }
        }
    }

    let current_state = mem::replace(&mut state, State::Idle);

    match current_state {
        State::Idle => { },
        State::ReadingKey(_) => {
            return Err(Error::MissingValue)
        },
        State::ReadingValue(key, value) => {
            pairs.push((key, value.into_iter().collect()));
        },
        State::ReadingQuotedValue(_, _) => {
            return Err(Error::UnexpectedEndOfString);
        }
    }

    Ok(Attributes(pairs))
}

#[derive(Debug, Clone)]
pub enum Error {
    UnexpectedEndOfString,
    InvalidKeyChar,
    InvalidValueChar,
    MissingValue,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::UnexpectedEndOfString => write!(f, "unexpected end of string"),
            Error::InvalidKeyChar => write!(f, "invalid key character"),
            Error::InvalidValueChar => write!(f, "invalid value character"),
            Error::MissingValue => write!(f, "missing value"),
        }
    }
}

impl std::error::Error for Error { }

enum State {
    Idle,
    ReadingKey(Vec<char>),
    ReadingValue(String, Vec<char>),
    ReadingQuotedValue(String, Vec<char>),
}

#[cfg(test)]
mod test {
    use super::parse;

    #[test]
    fn empty() {
        let attributes = parse("").unwrap().as_hashmap();

        assert_eq!(attributes.len(), 0)
    }

    #[test]
    fn a_is_b() {
        let attributes = parse("a=b").unwrap().as_hashmap();

        assert_eq!(attributes.len(), 1);
        assert_eq!(attributes["a"], "b")
    }

    #[test]
    fn aa_is_xyz() {
        let attributes = parse("aa=xyz").unwrap().as_hashmap();

        assert_eq!(attributes.len(), 1);
        assert_eq!(attributes["aa"], "xyz")
    }

    #[test]
    fn aa_is_xyz_and_bb_is_x() {
        let attributes = parse("aa=xyz bb=x").unwrap().as_hashmap();

        assert_eq!(attributes.len(), 2);
        assert_eq!(attributes["aa"], "xyz");
        assert_eq!(attributes["bb"], "x")
    }

    #[test]
    fn redundant_spaces() {
        let attributes = parse("    aa=xyz  bb=x  ").unwrap().as_hashmap();

        assert_eq!(attributes.len(), 2);
        assert_eq!(attributes["aa"], "xyz");
        assert_eq!(attributes["bb"], "x")
    }

    #[test]
    fn empty_value() {
        let attributes = parse("f= g=\"\" h=").unwrap().as_hashmap();

        assert_eq!(attributes.len(), 3);
        assert_eq!(attributes["f"], "");
        assert_eq!(attributes["g"], "");
        assert_eq!(attributes["h"], "");
    }

    #[test]
    #[should_panic]
    fn unclosed_quoted_value() {
        parse("x=\"").unwrap();
    }

    #[test]
    #[should_panic]
    fn unassigned_key() {
        parse("x y").unwrap();
    }
}