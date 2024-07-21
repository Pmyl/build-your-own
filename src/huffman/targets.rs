use std::io::{BufReader, Read};

pub enum HuffmanInput<'a> {
    Content(Vec<u8>),
    File(&'a str),
}

impl<'a> HuffmanInput<'a> {
    pub fn new(input_file: Option<&'a str>, input: impl Read) -> Self {
        if let Some(file) = input_file {
            Self::File(file)
        } else {
            let mut reader = BufReader::new(input);
            let mut contents = Vec::new();
            reader.read_to_end(&mut contents).expect("to read");
            Self::Content(contents)
        }
    }

    pub fn take(&'a self) -> Box<dyn Read + 'a> {
        match self {
            HuffmanInput::Content(ref content) => Box::new(content.as_slice()),
            HuffmanInput::File(ref file) => Box::new(BufReader::new(
                std::fs::File::open(file).expect("file not found"),
            )),
        }
    }
}
