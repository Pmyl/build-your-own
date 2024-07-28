use std::io::{BufRead, BufReader, Read, Write};

use build_your_own_macros::cli_options;
use build_your_own_utils::my_own_error::MyOwnError;

// https://codingchallenges.fyi/challenges/challenge-wc

pub fn wc_cli(args: &[&str]) -> Result<(), MyOwnError> {
    wc_cli_impl(args, std::io::stdin(), std::io::stdout())
}

fn wc_cli_impl(args: &[&str], stdin: impl Read, mut stdout: impl Write) -> Result<(), MyOwnError> {
    let cli_options = WcCliOptions::from_args(args)?;

    let result = if let Some(filepath) = cli_options.filepath {
        let file =
            std::fs::File::open(filepath).inspect_err(|_| eprintln!("no {} file", filepath))?;

        wc(file)?
    } else {
        wc(stdin)?
    };

    let options = cli_options.options();

    if options.lines {
        write!(stdout, "{} ", result.lines)?;
    }

    if options.words {
        write!(stdout, "{} ", result.words)?;
    }

    if options.characters {
        write!(stdout, "{} ", result.characters)?;
    }

    if options.bytes {
        write!(stdout, "{} ", result.bytes)?;
    }

    if let Some(filepath) = cli_options.filepath {
        write!(stdout, "{}", filepath)?;
    }

    writeln!(stdout, "")?;

    Ok(())
}

pub struct WcResult {
    bytes: usize,
    lines: usize,
    words: usize,
    characters: usize,
}

pub fn wc(reader: impl Read) -> Result<WcResult, MyOwnError> {
    let mut lines = 0;
    let mut bytes = 0;
    let mut words = 0;
    let mut characters = 0;
    let mut is_in_word = false;

    // to implement the "characters" feature we need to do magic to avoid allocation of strings
    // because Read doesn't support reading characters, only bytes

    let mut reader = BufReader::new(reader);
    let mut buf = Vec::<u8>::new();

    while reader.read_until(b'\n', &mut buf)? != 0 {
        // this moves the ownership of the read data to the string
        // there is no allocation
        let line = String::from_utf8(buf)?;
        for character in line.chars() {
            bytes += character.len_utf8();
            characters += 1;

            if character == '\n' {
                lines += 1;
            }

            if character.is_ascii_whitespace() {
                if is_in_word {
                    words += 1;
                    is_in_word = false;
                }
            } else {
                is_in_word = true;
            }
        }
        // this returns the ownership of the read data to buf
        // there is no allocation
        buf = line.into_bytes();
        buf.clear();
    }

    Ok(WcResult {
        lines: usize::max(lines, 1),
        words,
        characters,
        bytes,
    })
}

cli_options! {
    struct WcCliOptions<'a> {
        #[option()]
        filepath: Option<&'a str>,

        #[option(name = "-c")]
        bytes: Option<bool>,

        #[option(name = "-l")]
        lines: Option<bool>,

        #[option(name = "-w")]
        words: Option<bool>,

        #[option(name = "-m")]
        characters: Option<bool>,
    }
}

struct WcOptions {
    bytes: bool,
    lines: bool,
    words: bool,
    characters: bool,
}

impl<'a> WcCliOptions<'a> {
    fn options(&self) -> WcOptions {
        if self.bytes.is_none()
            && self.lines.is_none()
            && self.words.is_none()
            && self.characters.is_none()
        {
            WcOptions {
                bytes: true,
                lines: true,
                words: true,
                characters: false,
            }
        } else {
            WcOptions {
                bytes: self.bytes.unwrap_or(false),
                lines: self.lines.unwrap_or(false),
                words: self.words.unwrap_or(false),
                characters: self.characters.unwrap_or(false),
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn wc_cli_options_cases() -> Result<(), MyOwnError> {
        // All true
        let cli_options = WcCliOptions::from_args(&["-c", "-l", "-w", "-m", "test.txt"])?;
        assert_eq!(cli_options.filepath, Some("test.txt"));
        assert_eq!(cli_options.options().bytes, true);
        assert_eq!(cli_options.options().lines, true);
        assert_eq!(cli_options.options().words, true);
        assert_eq!(cli_options.options().characters, true);

        // No filename
        let cli_options = WcCliOptions::from_args(&["-c", "-l", "-w", "-m"])?;
        assert_eq!(cli_options.filepath, None);
        assert_eq!(cli_options.options().bytes, true);
        assert_eq!(cli_options.options().lines, true);
        assert_eq!(cli_options.options().words, true);
        assert_eq!(cli_options.options().characters, true);

        // Only lines
        let cli_options = WcCliOptions::from_args(&["-l", "test.txt"])?;
        assert_eq!(cli_options.filepath, Some("test.txt"));
        assert_eq!(cli_options.options().bytes, false);
        assert_eq!(cli_options.options().lines, true);
        assert_eq!(cli_options.options().words, false);
        assert_eq!(cli_options.options().characters, false);

        // Only bytes
        let cli_options = WcCliOptions::from_args(&["-c", "test.txt"])?;
        assert_eq!(cli_options.filepath, Some("test.txt"));
        assert_eq!(cli_options.options().bytes, true);
        assert_eq!(cli_options.options().lines, false);
        assert_eq!(cli_options.options().words, false);
        assert_eq!(cli_options.options().characters, false);

        // Only words
        let cli_options = WcCliOptions::from_args(&["-w", "test.txt"])?;
        assert_eq!(cli_options.filepath, Some("test.txt"));
        assert_eq!(cli_options.options().bytes, false);
        assert_eq!(cli_options.options().lines, false);
        assert_eq!(cli_options.options().words, true);
        assert_eq!(cli_options.options().characters, false);

        // Only characters
        let cli_options = WcCliOptions::from_args(&["-m", "test.txt"])?;
        assert_eq!(cli_options.filepath, Some("test.txt"));
        assert_eq!(cli_options.options().bytes, false);
        assert_eq!(cli_options.options().lines, false);
        assert_eq!(cli_options.options().words, false);
        assert_eq!(cli_options.options().characters, true);

        // Only filepath should have default options
        let cli_options = WcCliOptions::from_args(&["test.txt"])?;
        assert_eq!(cli_options.filepath, Some("test.txt"));
        assert_eq!(cli_options.options().bytes, true);
        assert_eq!(cli_options.options().lines, true);
        assert_eq!(cli_options.options().words, true);
        assert_eq!(cli_options.options().characters, false);

        // All true in inverted order
        let cli_options = WcCliOptions::from_args(&["test.txt", "-l", "-c", "-w", "-m"])?;
        assert_eq!(cli_options.filepath, Some("test.txt"));
        assert_eq!(cli_options.options().bytes, true);
        assert_eq!(cli_options.options().lines, true);
        assert_eq!(cli_options.options().words, true);
        assert_eq!(cli_options.options().characters, true);

        Ok(())
    }

    #[test]
    fn wc_dash_c_and_filename_shows_bytes_and_filename() {
        let mut output = Vec::new();
        let input: &[u8] = &[];
        wc_cli_impl(&["-c", "src/wc/test.txt"], input, &mut output).expect("to work");
        assert_eq!(output, b"342190 src/wc/test.txt\n");
    }

    #[test]
    fn wc_dash_l_and_filename_shows_lines_and_filename() {
        let mut output = Vec::new();
        let input: &[u8] = &[];
        wc_cli_impl(&["-l", "src/wc/test.txt"], input, &mut output).expect("to work");
        assert_eq!(output, b"7145 src/wc/test.txt\n");
    }

    #[test]
    fn wc_dash_w_and_filename_shows_words_and_filename() {
        let mut output = Vec::new();
        let input: &[u8] = &[];
        wc_cli_impl(&["-w", "src/wc/test.txt"], input, &mut output).expect("to work");
        assert_eq!(output, b"58164 src/wc/test.txt\n");
    }

    #[test]
    fn wc_dash_m_and_filename_shows_characters_and_filename() {
        let mut output = Vec::new();
        let input: &[u8] = &[];
        wc_cli_impl(&["-m", "src/wc/test.txt"], input, &mut output).expect("to work");
        assert_eq!(output, b"339292 src/wc/test.txt\n");
    }

    #[test]
    fn wc_dash_c_shows_bytes_of_stdin_until_end_of_file() {
        let mut output = Vec::new();
        let input: &[u8] = "some input".as_bytes();
        wc_cli_impl(&["-c"], input, &mut output).expect("to work");
        assert_eq!(output, b"10 \n");
    }

    #[test]
    fn wc_dash_l_only_include_newlines_to_adhere_to_posix_line_definition() {
        let mut output = Vec::new();
        let input: &[u8] = "new line \n no new line".as_bytes();
        wc_cli_impl(&["-l"], input, &mut output).expect("to work");
        assert_eq!(output, b"1 \n");
    }

    #[test]
    fn wc_dash_l_can_never_be_zero_lines() {
        let mut output = Vec::new();
        let input: &[u8] = "no new line".as_bytes();
        wc_cli_impl(&["-l"], input, &mut output).expect("to work");
        assert_eq!(output, b"1 \n");
    }
}
