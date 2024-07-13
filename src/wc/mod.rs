use std::io::{BufRead, BufReader, Read, Write};

// https://codingchallenges.fyi/challenges/challenge-wc

pub fn wc_cli(args: &[&str]) {
    wc_cli_impl(args, std::io::stdin(), std::io::stdout())
}

fn wc_cli_impl(args: &[&str], stdin: impl Read, mut stdout: impl Write) {
    let cli_options = wc_cli_options(args);

    let result = if let Some(filepath) = cli_options.filepath {
        let file = std::fs::File::open(filepath)
            .inspect_err(|_| eprintln!("no {} file", filepath))
            .expect("file not found");

        wc(file)
    } else {
        wc(stdin)
    };

    if cli_options.lines {
        write!(stdout, "{} ", result.lines).expect("should write");
    }

    if cli_options.words {
        write!(stdout, "{} ", result.words).expect("should write");
    }

    if cli_options.characters {
        write!(stdout, "{} ", result.characters).expect("should write");
    }

    if cli_options.bytes {
        write!(stdout, "{} ", result.bytes).expect("should write");
    }

    if let Some(filepath) = cli_options.filepath {
        write!(stdout, "{}", filepath).expect("should write");
    }

    writeln!(stdout, "").expect("should write");
}

fn wc_cli_options<'a>(args: &'a[&str]) -> WcCliOptions<'a> {
    let mut options: WcCliOptions = Default::default();
    let mut need_to_apply_defaults = true;

    for arg in args.iter() {
        match *arg {
            "-c" => { options.bytes = true; need_to_apply_defaults = false; }
            "-l" => { options.lines = true; need_to_apply_defaults = false; }
            "-w" => { options.words = true; need_to_apply_defaults = false; }
            "-m" => { options.characters = true; need_to_apply_defaults = false; }
            filepath_parameter => { options.filepath = Some(filepath_parameter); }
        }
    }

    if need_to_apply_defaults {
        options.bytes = true;
        options.lines = true;
        options.words = true;
    }

    options
}

pub struct WcResult {
    bytes: usize,
    lines: usize,
    words: usize,
    characters: usize,
}

pub fn wc(reader: impl Read) -> WcResult {
    let mut lines = 0;
    let mut bytes = 0;
    let mut words = 0;
    let mut characters = 0;
    let mut is_in_word = false;

    // to implement the "characters" feature we need to do magic to avoid allocation of strings
    // because Read doesn't support reading characters, only bytes

    let mut reader = BufReader::new(reader);
    let mut buf = Vec::<u8>::new();

    while reader.read_until(b'\n', &mut buf).expect("read_until failed") != 0 {
        // this moves the ownership of the read data to the string
        // there is no allocation
        let line = String::from_utf8(buf).expect("from_utf8 failed");
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

    WcResult { lines: usize::max(lines, 1), words, characters, bytes }
}

#[derive(Default)]
struct WcCliOptions<'a> {
    filepath: Option<&'a str>,
    bytes: bool,
    lines: bool,
    words: bool,
    characters: bool
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn wc_cli_options_cases() {
        // All true
        let cli_options = wc_cli_options(&["-c", "-l", "-w", "-m", "test.txt"]);
        assert_eq!(cli_options.filepath, Some("test.txt"));
        assert_eq!(cli_options.bytes, true);
        assert_eq!(cli_options.lines, true);
        assert_eq!(cli_options.words, true);
        assert_eq!(cli_options.characters, true);

        // No filename
        let cli_options = wc_cli_options(&["-c", "-l", "-w", "-m"]);
        assert_eq!(cli_options.filepath, None);
        assert_eq!(cli_options.bytes, true);
        assert_eq!(cli_options.lines, true);
        assert_eq!(cli_options.words, true);
        assert_eq!(cli_options.characters, true);

        // Only lines
        let cli_options = wc_cli_options(&["-l", "test.txt"]);
        assert_eq!(cli_options.filepath, Some("test.txt"));
        assert_eq!(cli_options.bytes, false);
        assert_eq!(cli_options.lines, true);
        assert_eq!(cli_options.words, false);
        assert_eq!(cli_options.characters, false);

        // Only bytes
        let cli_options = wc_cli_options(&["-c", "test.txt"]);
        assert_eq!(cli_options.filepath, Some("test.txt"));
        assert_eq!(cli_options.bytes, true);
        assert_eq!(cli_options.lines, false);
        assert_eq!(cli_options.words, false);
        assert_eq!(cli_options.characters, false);

        // Only words
        let cli_options = wc_cli_options(&["-w", "test.txt"]);
        assert_eq!(cli_options.filepath, Some("test.txt"));
        assert_eq!(cli_options.bytes, false);
        assert_eq!(cli_options.lines, false);
        assert_eq!(cli_options.words, true);
        assert_eq!(cli_options.characters, false);

        // Only characters
        let cli_options = wc_cli_options(&["-m", "test.txt"]);
        assert_eq!(cli_options.filepath, Some("test.txt"));
        assert_eq!(cli_options.bytes, false);
        assert_eq!(cli_options.lines, false);
        assert_eq!(cli_options.words, false);
        assert_eq!(cli_options.characters, true);

        // Only filepath should have default options
        let cli_options = wc_cli_options(&["test.txt"]);
        assert_eq!(cli_options.filepath, Some("test.txt"));
        assert_eq!(cli_options.bytes, true);
        assert_eq!(cli_options.lines, true);
        assert_eq!(cli_options.words, true);
        assert_eq!(cli_options.characters, false);

        // All true in inverted order
        let cli_options = wc_cli_options(&["test.txt", "-l", "-c", "-w", "-m"]);
        assert_eq!(cli_options.filepath, Some("test.txt"));
        assert_eq!(cli_options.bytes, true);
        assert_eq!(cli_options.lines, true);
        assert_eq!(cli_options.words, true);
        assert_eq!(cli_options.characters, true);
    }

    #[test]
    fn wc_dash_c_and_filename_shows_bytes_and_filename() {
        let mut output = Vec::new();
        let input: &[u8] = &[];
        wc_cli_impl(&["-c", "src/wc/test.txt"], input, &mut output);
        assert_eq!(output, b"342190 src/wc/test.txt\n");
    }

    #[test]
    fn wc_dash_l_and_filename_shows_lines_and_filename() {
        let mut output = Vec::new();
        let input: &[u8] = &[];
        wc_cli_impl(&["-l", "src/wc/test.txt"], input, &mut output);
        assert_eq!(output, b"7145 src/wc/test.txt\n");
    }

    #[test]
    fn wc_dash_w_and_filename_shows_words_and_filename() {
        let mut output = Vec::new();
        let input: &[u8] = &[];
        wc_cli_impl(&["-w", "src/wc/test.txt"], input, &mut output);
        assert_eq!(output, b"58164 src/wc/test.txt\n");
    }

    #[test]
    fn wc_dash_m_and_filename_shows_characters_and_filename() {
        let mut output = Vec::new();
        let input: &[u8] = &[];
        wc_cli_impl(&["-m", "src/wc/test.txt"], input, &mut output);
        assert_eq!(output, b"339292 src/wc/test.txt\n");
    }

    #[test]
    fn wc_dash_c_shows_bytes_of_stdin_until_end_of_file() {
        let mut output = Vec::new();
        let input: &[u8] = "some input".as_bytes();
        wc_cli_impl(&["-c"], input, &mut output);
        assert_eq!(output, b"10 \n");
    }

    #[test]
    fn wc_dash_l_only_include_newlines_to_adhere_to_posix_line_definition() {
        let mut output = Vec::new();
        let input: &[u8] = "new line \n no new line".as_bytes();
        wc_cli_impl(&["-l"], input, &mut output);
        assert_eq!(output, b"1 \n");
    }

    #[test]
    fn wc_dash_l_can_never_be_zero_lines() {
        let mut output = Vec::new();
        let input: &[u8] = "no new line".as_bytes();
        wc_cli_impl(&["-l"], input, &mut output);
        assert_eq!(output, b"1 \n");
    }
}
