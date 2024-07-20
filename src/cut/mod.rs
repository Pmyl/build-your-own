use std::{
    error::Error,
    io::{stdin, stdout, BufRead, BufReader, Read, Write},
    num::ParseIntError,
};

// https://codingchallenges.fyi/challenges/challenge-cut

pub fn cut_cli(args: &[&str]) {
    cut_cli_impl(args, stdin(), stdout()).expect("to work")
}

fn cut_cli_impl(args: &[&str], input: impl Read, output: impl Write) -> Result<(), Box<dyn Error>> {
    let options = CutCliOptions::from_args(args);

    if let Some(input_file) = options.input_file {
        let file = std::fs::File::open(input_file)
            .inspect_err(|_| eprintln!("no {} file", input_file))
            .expect("file not found");

        cut(options.options, file, output)
    } else {
        cut(options.options, input, output)
    }
}

fn cut(
    options: CutOptions,
    input: impl Read,
    mut output: impl Write,
) -> Result<(), Box<dyn Error>> {
    let mut reader = BufReader::new(input);
    let mut buf = Vec::new();

    while reader.read_until(b'\n', &mut buf)? != 0 {
        let line = String::from_utf8(buf)
            .expect("from_utf8 failed")
            .trim_end()
            .to_string();
        if let Some(ref fields) = options.fields {
            let all_fields: Vec<&str> = line.split(options.delimiter).collect();

            let result = write!(
                output,
                "{}",
                fields
                    .iter()
                    .map(|f| all_fields.get(f - 1).map(|s| *s).unwrap_or(""))
                    .collect::<Vec<&str>>()
                    .join(options.delimiter.to_string().as_str())
            );

            if let Err(err) = result {
                if err.kind() == std::io::ErrorKind::BrokenPipe {
                    return Ok(());
                }
                return Err(err.into());
            }
        }

        let result = writeln!(output);
        if let Err(err) = result {
            if err.kind() == std::io::ErrorKind::BrokenPipe {
                return Ok(());
            }
            return Err(err.into());
        }
        buf = line.into_bytes();
        buf.clear();
    }
    Ok(())
}

struct CutCliOptions<'a> {
    input_file: Option<&'a str>,
    options: CutOptions,
}

struct CutOptions {
    fields: Option<Vec<usize>>,
    delimiter: char,
}

impl<'a> CutCliOptions<'a> {
    fn from_args(args: &[&'a str]) -> Self {
        let mut input_file = None;
        let mut fields = None;
        let mut delimiter = '\t';

        let mut args = args.iter();
        loop {
            let arg = args.next();
            let Some(&arg) = arg else {
                break;
            };

            if arg.starts_with("-f") {
                fields = Some(
                    arg.trim_start_matches("-f")
                        .split(&[','])
                        .map(|f| f.parse())
                        .collect::<Result<Vec<usize>, ParseIntError>>()
                        .expect("-f to have numbers e.g. -f1 -f1,2"),
                );
                continue;
            }

            if arg.starts_with("-d") {
                let provided_delimiter = arg.trim_start_matches("-d");
                if provided_delimiter.len() != 1 {
                    panic!("delimiter must be a single character e.g. -d,");
                }
                delimiter = provided_delimiter.chars().next().unwrap();
                continue;
            }

            if arg == "-" {
                continue;
            }

            input_file = Some(arg);
        }

        Self {
            input_file,
            options: CutOptions { fields, delimiter },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cut_field_all_lines() {
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
}
