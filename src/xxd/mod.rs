use std::{
    fs::File,
    io::{stdin, stdout, BufRead, BufReader, Read, Write},
};

use build_your_own_macros::cli_options;
use build_your_own_shared::my_own_error::MyOwnError;

pub fn xxd_cli(args: &[&str]) -> Result<(), MyOwnError> {
    let options = XxdCliOptions::from_args(args)?;

    if let Some(input_file) = options.input_file {
        xxd_cli_impl(options.options, File::open(input_file)?, stdout())
    } else {
        xxd_cli_impl(options.options, stdin(), stdout())
    }
}

cli_options! {
    struct XxdCliOptions<'a> {
        #[option()]
        input_file: Option<&'a str>,

        #[suboptions(name = "options")]
        struct XxdOptions {
            #[option(name = "-e", default = false)]
            little_endian: bool,

            #[option(name = "-g")]
            grouping: Option<u8>,

            #[option(name = "-l")]
            octets_to_output: Option<usize>,

            #[option(name = "-c", default = 16)]
            octets_per_line: u8,

            #[option(name = "-s")]
            start_offset: usize,

            #[option(name = "-r")]
            to_binary: bool,
        }
    }
}

fn xxd_cli_impl(
    options: XxdOptions,
    input: impl Read,
    output: impl Write,
) -> Result<(), MyOwnError> {
    if options.to_binary {
        xxd_to_binary(input, output)
    } else {
        xxd_to_hex(options, input, output)
    }
}

fn xxd_to_hex(
    options: XxdOptions,
    mut input: impl Read,
    mut output: impl Write,
) -> Result<(), MyOwnError> {
    let mut offset = options.start_offset;
    let grouping = options
        .grouping
        .unwrap_or_else(|| if options.little_endian { 4 } else { 2 }) as usize;
    let mut octets_to_output = options.octets_to_output;

    if options.little_endian && grouping != 2 && grouping != 4 && grouping != 8 {
        return Err("myown xxd: number of octets per group must be a power of 2 with -e.".into());
    }

    let mut buffer = vec![0; offset];
    input.read(&mut buffer)?;

    loop {
        let mut buffer = [0; 65536];
        let mut bytes_read = input.read(&mut buffer)?;

        if let Some(octets_to_output) = octets_to_output.as_mut() {
            if bytes_read >= *octets_to_output {
                bytes_read = *octets_to_output;
            }

            *octets_to_output = *octets_to_output - bytes_read;
        }

        if bytes_read == 0 {
            break;
        }

        for buffer in buffer[..bytes_read].chunks(options.octets_per_line as usize) {
            write!(output, "{:08x}:", offset)?;
            offset += options.octets_per_line as usize;

            if options.little_endian {
                for chunk in buffer.chunks(grouping) {
                    write!(output, " ")?;

                    if grouping == 2 {
                        let bytes = u16::from_le_bytes([chunk[0], chunk[1]]).to_be_bytes();
                        for byte in bytes {
                            write!(output, "{:02x}", byte)?;
                        }
                    } else if grouping == 4 {
                        let bytes = u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]])
                            .to_be_bytes();
                        for byte in bytes {
                            write!(output, "{:02x}", byte)?;
                        }
                    } else if grouping == 8 {
                        let bytes = u64::from_le_bytes([
                            chunk[0], chunk[1], chunk[2], chunk[3], chunk[4], chunk[5], chunk[6],
                            chunk[7],
                        ])
                        .to_be_bytes();
                        for byte in bytes {
                            write!(output, "{:02x}", byte)?;
                        }
                    } else {
                        return Err(
                            "myown xxd: number of octets per group must be a power of 2 with -e."
                                .into(),
                        );
                    };
                }
            } else {
                for i in 0..buffer.len() {
                    if i % grouping == 0 {
                        write!(output, " ")?;
                    }
                    write!(output, "{:02x}", buffer[i])?;
                }
            }

            for _ in buffer.len()..(options.octets_per_line as usize) {
                write!(output, "  ")?;
            }

            for _ in 0..((options.octets_per_line as usize - buffer.len()) / grouping) {
                write!(output, " ")?;
            }

            write!(output, "  ")?;
            for byte in buffer.iter() {
                if *byte >= 32 && *byte <= 126 {
                    write!(output, "{}", *byte as char)?;
                } else {
                    write!(output, ".")?;
                }
            }

            write!(output, "\n")?;
        }
    }

    Ok(())
}

fn xxd_to_binary(input: impl Read, mut output: impl Write) -> Result<(), MyOwnError> {
    let mut reader = BufReader::new(input);
    loop {
        let mut prefix = [0; 10];
        reader.read(&mut prefix)?;

        let mut buffer = Vec::new();
        let bytes_read = reader.read_until(b'\n', &mut buffer)?;

        if bytes_read == 0 {
            break;
        }

        let mut space_met = false;
        let mut hex_byte = [' '; 2];
        let mut hex_byte_index = 0;
        for byte in buffer.iter() {
            if *byte == b' ' {
                if space_met {
                    break;
                }

                space_met = true;
                continue;
            }

            space_met = false;
            hex_byte[hex_byte_index] = *byte as char;
            hex_byte_index += 1;

            if hex_byte_index == 2 {
                let byte = u8::from_str_radix(&hex_byte.iter().collect::<String>(), 16)?;
                output.write_all(&[byte])?;
                hex_byte_index = 0;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! test_xxd {
        ($test_name:ident, $($args:expr),+) => {
            #[test]
            fn $test_name() {
                let xxd_output = std::process::Command::new("xxd")
                        $(.arg($args))*
                        .output()
                        .expect("failed to execute process");
                let mut output = Vec::new();
                let options = XxdCliOptions::from_args(&[$($args,)*]).unwrap();
                let file = std::fs::File::open(options.input_file.unwrap()).unwrap();
                xxd_cli_impl(
                    options.options,
                    file,
                    &mut output,
                ).unwrap();

                assert_eq!(xxd_output.stdout, output);
            }
        };
    }

    test_xxd!(vs_real_xxd_full_tar, "src/xxd/files.tar");
    test_xxd!(vs_real_xxd_c2_tar, "-c2", "src/xxd/files.tar");
    test_xxd!(vs_real_xxd_l2_tar, "-l2", "src/xxd/files.tar");
    test_xxd!(vs_real_xxd_l10_tar, "-l10", "src/xxd/files.tar");
    test_xxd!(vs_real_xxd_e_g8_tar, "-e", "-g8", "src/xxd/files.tar");
    test_xxd!(vs_real_xxd_rev_tar, "-r", "src/xxd/files.tar.hex");
}
