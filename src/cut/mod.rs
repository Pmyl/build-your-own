use std::io::{stdin, stdout, BufRead, BufReader, Read, Write};

use build_your_own_macros::cli_options;
use build_your_own_shared::my_own_error::MyOwnError;

// https://codingchallenges.fyi/challenges/challenge-cut

pub fn cut_cli(args: &[&str]) -> Result<(), MyOwnError> {
    cut_cli_impl(args, stdin(), stdout())
}

fn cut_cli_impl(args: &[&str], input: impl Read, output: impl Write) -> Result<(), MyOwnError> {
    let options = CutCliOptions::from_args(args)?;

    if let Some(input_file) = options.input_file {
        let file =
            std::fs::File::open(input_file).inspect_err(|_| eprintln!("no {} file", input_file))?;

        cut(options.options, file, output)
    } else {
        cut(options.options, input, output)
    }
}

fn cut(options: CutOptions, input: impl Read, mut output: impl Write) -> Result<(), MyOwnError> {
    let mut reader = BufReader::new(input);
    let mut buf = Vec::new();

    while reader.read_until(b'\n', &mut buf)? != 0 {
        let line = String::from_utf8(buf)?.trim_end().to_string();
        let all_fields: Vec<&str> = line.split(options.delimiter).collect();

        write!(
            output,
            "{}",
            options
                .fields
                .iter()
                .map(|f| all_fields.get(f - 1).map(|s| *s).unwrap_or(""))
                .collect::<Vec<&str>>()
                .join(options.delimiter.to_string().as_str())
        )?;

        writeln!(output)?;
        buf = line.into_bytes();
        buf.clear();
    }
    Ok(())
}

cli_options! {
    struct CutCliOptions<'a> {
        #[option()]
        input_file: Option<&'a str>,

        #[suboptions(name = "options")]
        struct CutOptions {
            #[option(name = "-f", delimiters = &[' ', ','])]
            fields: Vec<usize>,
            #[option(name = "-d", default = '\t')]
            delimiter: char,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cut_field_all_lines_with_default_delimiter() {
        let input: &[u8] = "1\t2\t3\n4\t5\t6".as_bytes();
        let mut output = Vec::new();
        cut_cli_impl(&["-f1"], input, &mut output).expect("to work");

        assert_eq!(
            String::from_utf8(output).expect("to be a string"),
            "1\n4\n".to_string()
        );

        let mut output = Vec::new();
        cut_cli_impl(&["-f2"], input, &mut output).expect("to work");

        assert_eq!(
            String::from_utf8(output).expect("to be a string"),
            "2\n5\n".to_string()
        );
    }

    #[test]
    fn cut_using_provided_delimiter() {
        let input: &[u8] = "1s2s3".as_bytes();
        let mut output = Vec::new();
        cut_cli_impl(&["-f3", "-ds"], input, &mut output).expect("to work");

        assert_eq!(
            String::from_utf8(output).expect("to be a string"),
            "3\n".to_string()
        );
    }

    #[test]
    fn cut_fields_and_delimiter() {
        let input: &[u8] = "1?2?3\n4?5?6".as_bytes();
        let mut output = Vec::new();
        cut_cli_impl(&["-f1,3", "-d?"], input, &mut output).expect("to work");

        assert_eq!(
            String::from_utf8(output).expect("to be a string"),
            "1?3\n4?6\n".to_string()
        );
    }

    #[test]
    fn cut_fields_and_delimiter_specified_with_space() {
        let input: &[u8] = "1?2?3\n4?5?6".as_bytes();
        let mut output = Vec::new();
        cut_cli_impl(&["-f", "1,3", "-d", "?"], input, &mut output).expect("to work");

        assert_eq!(
            String::from_utf8(output).expect("to be a string"),
            "1?3\n4?6\n".to_string()
        );

        let mut output = Vec::new();
        cut_cli_impl(&["-f", "1 3", "-d", "?"], input, &mut output).expect("to work");

        assert_eq!(
            String::from_utf8(output).expect("to be a string"),
            "1?3\n4?6\n".to_string()
        );
    }
}
