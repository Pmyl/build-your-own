use std::cell::RefCell;
use std::io::{stdin, Read};
use std::iter::Peekable;
use std::vec::IntoIter;

use build_your_own_utils::my_own_error::MyOwnError;

// https://codingchallenges.fyi/challenges/challenge-json-checker

pub fn json_checker_cli(_: &[&str]) -> Result<(), MyOwnError> {
    let result = json_checker_cli_impl(stdin())?;

    if let JsonCheckerResult::Pass = result {
        std::process::exit(0);
    } else {
        std::process::exit(1);
    }
}

fn json_checker_cli_impl(mut reader: impl Read) -> Result<JsonCheckerResult, MyOwnError> {
    let mut json = String::new();
    reader.read_to_string(&mut json)?;

    let mut tokens = Vec::new();
    let mut in_string = false;
    let mut in_number = false;
    let mut in_identifier = false;
    let mut identifier = String::new();

    for byte in json.bytes() {
        if in_number {
            if !byte.is_ascii_digit() {
                in_number = false;
                tokens.push(Token::Number)
            } else {
                continue;
            }
        }

        if in_identifier {
            if !byte.is_ascii_alphabetic() {
                in_identifier = false;
                tokens.push(Token::Identifier(identifier));
                identifier = String::new();
            } else {
                identifier.push(byte.into());
                continue;
            }
        }

        if in_string {
            if byte == b'"' {
                in_string = false;
                tokens.push(Token::String)
            }
        } else if byte == b'{' {
            tokens.push(Token::OpenCurly);
        } else if byte == b'}' {
            tokens.push(Token::CloseCurly);
        } else if byte == b'[' {
            tokens.push(Token::OpenSquare);
        } else if byte == b']' {
            tokens.push(Token::CloseSquare);
        } else if byte == b'"' {
            in_string = true;
        } else if byte == b',' {
            tokens.push(Token::Comma);
        } else if byte == b':' {
            tokens.push(Token::Colon);
        } else if byte.is_ascii_digit() {
            in_number = true;
        } else if byte.is_ascii_alphabetic() {
            in_identifier = true;
            identifier.push(byte.into());
        } else if byte.is_ascii_whitespace() {
            // do nothing
        } else {
            tokens.push(Token::Unknown)
        }
    }

    Ok(if let None = parse(tokens) {
        JsonCheckerResult::Fail
    } else {
        JsonCheckerResult::Pass
    })
}

// json <- object | array | literal
// object <- '{' properties? '}'
// properties <- property (',' property)*
// property <- string ':' json
// literal <- string | number | 'true' | 'false' | 'null'
// string <- '"' character* '"'
// array <- '[' array_values? ']'
// array_values <- json (',' json)*
fn parse(tokens: Vec<Token>) -> Option<()> {
    if tokens.is_empty() {
        return None;
    }

    let mut tokens = RefCell::new(tokens.into_iter().peekable());

    parse_json(&mut tokens)
}

// json <- object | array | literal
fn parse_json(tokens: &mut RefCell<Peekable<IntoIter<Token>>>) -> Option<()> {
    if let Some(token) = tokens.get_mut().peek() {
        if let Token::OpenCurly = token {
            tokens.get_mut().next();
            parse_object(tokens)?;
        } else if let Token::OpenSquare = token {
            tokens.get_mut().next();
            parse_array(tokens)?;
        } else if !match_literal(tokens)? {
            return None;
        }
    }

    Some(())
}

// object <- '{' properties? '}'
fn parse_object(tokens: &mut RefCell<Peekable<IntoIter<Token>>>) -> Option<()> {
    parse_properties(tokens)?;

    let token = tokens.get_mut().next();
    if let Some(Token::CloseCurly) = token {
        Some(())
    } else {
        None
    }
}

// array <- '[' array_values? ']'
fn parse_array(tokens: &mut RefCell<Peekable<IntoIter<Token>>>) -> Option<()> {
    parse_array_values(tokens)?;

    let token = tokens.get_mut().next();
    if let Some(Token::CloseSquare) = token {
        Some(())
    } else {
        None
    }
}

// array_values <- json (',' json)*
fn parse_array_values(tokens: &mut RefCell<Peekable<IntoIter<Token>>>) -> Option<()> {
    if let Some(Token::String) = tokens.get_mut().peek() {
        parse_json(tokens)?;
    } else {
        return Some(());
    }

    loop {
        let Some(Token::Comma) = tokens.get_mut().peek() else {
            break;
        };

        tokens.get_mut().next();

        if let Some(Token::String) = tokens.get_mut().peek() {
            parse_json(tokens)?;
        } else {
            return None;
        }
    }

    Some(())
}

// properties <- property (',' property)*
fn parse_properties(tokens: &mut RefCell<Peekable<IntoIter<Token>>>) -> Option<()> {
    if let Some(Token::String) = tokens.get_mut().peek() {
        parse_property(tokens)?;
    } else {
        return Some(());
    }

    loop {
        let Some(Token::Comma) = tokens.get_mut().peek() else {
            break;
        };

        tokens.get_mut().next();

        if let Some(Token::String) = tokens.get_mut().peek() {
            parse_property(tokens)?;
        } else {
            return None;
        }
    }

    Some(())
}

// property <- string ':' json
fn parse_property(tokens: &mut RefCell<Peekable<IntoIter<Token>>>) -> Option<()> {
    let Some(Token::String) = tokens.get_mut().next() else {
        return None;
    };

    let Some(Token::Colon) = tokens.get_mut().next() else {
        return None;
    };

    parse_json(tokens)
}

// literal <- string | number | 'true' | 'false' | 'null'
fn match_literal(tokens: &mut RefCell<Peekable<IntoIter<Token>>>) -> Option<bool> {
    let token = tokens.get_mut().next()?;

    if let Token::Identifier(identifier) = token {
        return match identifier.as_str() {
            "true" | "false" | "null" => Some(true),
            _ => Some(false),
        };
    }

    let (Token::Number | Token::String) = token else {
        return Some(false);
    };

    Some(true)
}

#[derive(Debug)]
enum Token {
    OpenCurly,
    CloseCurly,
    OpenSquare,
    CloseSquare,
    String,
    Number,
    Identifier(String),
    Comma,
    Colon,
    Unknown,
}

enum JsonCheckerResult {
    Pass,
    Fail,
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! test_files {
        ($name:ident, $result:ident) => {
            #[test]
            fn $name() {
                let file = std::fs::File::open(concat!(
                    "src/json_checker/tests/",
                    stringify!($name),
                    ".json"
                ))
                .expect("file to exists");
                assert!(matches!(
                    json_checker_cli_impl(file).expect("to not fail"),
                    JsonCheckerResult::$result
                ));
            }
        };
    }

    test_files!(valid1, Pass);
    test_files!(valid2, Pass);
    test_files!(valid3, Pass);
    test_files!(valid4, Pass);
    test_files!(valid5, Pass);
    test_files!(valid6, Pass);

    test_files!(invalid1, Fail);
    test_files!(invalid2, Fail);
    test_files!(invalid3, Fail);
    test_files!(invalid4, Fail);
    test_files!(invalid5, Fail);
}
