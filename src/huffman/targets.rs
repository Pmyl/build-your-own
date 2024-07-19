use std::io::{BufReader, BufWriter, Read, Write};

pub struct HuffmanTargets<'a> {
    input: HuffmanInput<'a>,
    output: HuffmanOutput<'a>,
}

pub enum HuffmanInput<'a> {
    Content(Vec<u8>),
    File(&'a str),
}

pub enum HuffmanOutput<'a> {
    Buffer(Box<dyn Write + 'a>),
    File(&'a str),
}

impl<'a> HuffmanOutput<'a> {
    pub fn take(self) -> Box<dyn Write + 'a> {
        match self {
            HuffmanOutput::Buffer(buffer) => buffer,
            HuffmanOutput::File(ref file) => Box::new(BufWriter::new(
                std::fs::File::create(file).expect("file not found"),
            )),
        }
    }
}

impl<'a> HuffmanInput<'a> {
    pub fn take(&'a self) -> Box<dyn Read + 'a> {
        match self {
            HuffmanInput::Content(ref content) => Box::new(content.as_slice()),
            HuffmanInput::File(ref file) => Box::new(BufReader::new(
                std::fs::File::open(file).expect("file not found"),
            )),
        }
    }
}

impl<'a> HuffmanTargets<'a> {
    pub fn new(
        input_file: Option<&'a str>,
        input: impl Read,
        output_file: Option<&'a str>,
        output: impl Write + 'a,
    ) -> Self {
        let input = if let Some(file) = input_file {
            HuffmanInput::File(file)
        } else {
            let mut reader = BufReader::new(input);
            let mut contents = Vec::new();
            reader.read_to_end(&mut contents).expect("to read");
            HuffmanInput::Content(contents)
        };

        let output = if let Some(file) = output_file {
            HuffmanOutput::File(file)
        } else {
            HuffmanOutput::Buffer(Box::new(output))
        };

        Self { input, output }
    }

    pub fn take(self) -> (HuffmanInput<'a>, HuffmanOutput<'a>) {
        (self.input, self.output)
    }
}
