use std::cmp::{Ordering, Reverse};
use std::collections::{BinaryHeap, HashMap};
use std::fmt::Display;
use std::fmt::Formatter;
use std::io::{BufRead, BufReader, Read, stdin, stdout, Write};

// https://codingchallenges.fyi/challenges/challenge-huffman

pub fn huffman_cli(args: &[&str]) {
    huffman_cli_impl(args, stdin(), stdout());
}

fn huffman_cli_impl(args: &[&str], input: impl Read, output: impl Write) {
    let options = get_options(args);

    let targets = HuffmanTargets::new(options.input_file, input);

    let mut output: Box<dyn Write> = if let Some(file) = options.output_file {
        Box::new(std::fs::File::create(file).expect("to create a file"))
    } else {
        Box::new(output)
    };

    if let HuffmanMode::Encode = options.mode {
        let frequencies = huffman_frequencies(&mut targets.input());
        let root = huffman_tree(frequencies);
        let table = huffman_prefix_code_table(root.clone());

        write_huffman_file(
            &mut targets.input(),
            &mut output,
            table,
            root
        );
    } else {
        read_huffman_file(
            &mut targets.input(),
            &mut output,
        );
    }
}

fn huffman_frequencies(input: &mut (impl Read + ?Sized)) -> HashMap<u8, usize> {
    let mut frequencies: HashMap<u8, usize> = HashMap::new();

    let mut reader = BufReader::new(input);
    let mut buf = Vec::<u8>::new();

    while reader.read_until(b'\n', &mut buf).expect("read_until failed") != 0 {
        for byte in buf.into_iter() {
            let frequency = frequencies.entry(byte).or_insert(0);
            *frequency += 1;
        }

        buf = Vec::new();
    }

    frequencies
}

fn huffman_tree(frequencies: HashMap<u8, usize>) -> HuffmanNode {
    let mut nodes = BinaryHeap::<Reverse<HuffmanNode>>::new();
    for byte_frequency in frequencies {
        nodes.push(Reverse(HuffmanNode {
            frequency: byte_frequency.1,
            byte: Some(byte_frequency.0),
            left: None,
            right: None
        }));
    }

    while nodes.len() > 1 {
        let node1 = nodes.pop().unwrap().0;
        let node2 = nodes.pop().unwrap().0;
        nodes.push(Reverse(HuffmanNode {
            frequency: node1.frequency + node2.frequency,
            byte: None,
            left: Some(Box::new(node1)),
            right: Some(Box::new(node2))
        }));
    }

    nodes.pop().unwrap().0
}

fn huffman_prefix_code_table(root: HuffmanNode) -> HuffmanPrefixCodeTable {
    let mut prefix_code_table = HashMap::new();
    let mut nodes_to_process: Vec<(HuffmanNode, Bits)> = vec![(root, Bits::empty())];
    while nodes_to_process.len() > 0 {
        let node_with_prefix = nodes_to_process.pop().unwrap();

        if let Some(byte) = node_with_prefix.0.byte {
            prefix_code_table.insert(byte, node_with_prefix.1);
        } else {
            if let Some(left) = node_with_prefix.0.left {
                nodes_to_process.push((*left, node_with_prefix.1.add(false)));
            }
            if let Some(right) = node_with_prefix.0.right {
                nodes_to_process.push((*right, node_with_prefix.1.add(true)));
            }
        }
    }

    HuffmanPrefixCodeTable(prefix_code_table)
}

fn write_huffman_file(input: &mut (impl Read + ?Sized), output: &mut (impl Write + ?Sized), table: HuffmanPrefixCodeTable, root: HuffmanNode) {
    let mut nodes_to_process: Vec<HuffmanNode> = vec![root];
    let mut writer = BitsWriter::new(output);

    while nodes_to_process.len() > 0 {
        let node = nodes_to_process.pop().unwrap();

        if let Some(byte) = node.byte {
            writer.write(&Bits::empty().add(true));
            writer.write(&Bits::byte(byte));
        } else {
            writer.write(&Bits::empty().add(false));
            if let Some(right) = node.right {
                nodes_to_process.push(*right);
            }
            if let Some(left) = node.left {
                nodes_to_process.push(*left);
            }
        }
    }

    for byte in input.bytes() {
        let byte = byte.unwrap();
        let prefix_code = table.0.get(&byte).expect("byte not found");
        writer.write(prefix_code);
    }
}

fn read_huffman_file(input: &mut (impl Read + ?Sized), output: &mut (impl Write + ?Sized)) {
    let mut reader = BitsReader::new(input);
    let root = decode_tree(&mut reader);

    let mut current_node: &HuffmanNode = &root;

    loop {
        let bit = reader.read_safe();

        match bit {
            Some(true) => current_node = &current_node.right.as_ref().unwrap(),
            Some(false) => current_node = &current_node.left.as_ref().unwrap(),
            None => break,
        }

        if current_node.byte.is_some() {
            output.write(&[current_node.byte.unwrap()]).expect("write byte");
            current_node = &root;
        }
    }
}

fn decode_tree<T: Read>(reader: &mut BitsReader<T>) -> HuffmanNode {
    let bit = reader.read();

    if bit {
        HuffmanNode {
            frequency: 0,
            byte: Some(reader.read_byte()),
            left: None,
            right: None
        }
    } else {
        let left = decode_tree(reader);
        let right = decode_tree(reader);
        HuffmanNode {
            frequency: 0,
            byte: None,
            left: Some(Box::new(left)),
            right: Some(Box::new(right))
        }
    }
}

// fn decode_tree2<T: Read>(reader: &mut BitsReader<T>) -> HuffmanNode {
//     let mut nodes = VecDeque::new();
//     let mut nodes_awaiting_children = 0;
//
//     loop {
//         let bit = reader.read();
//
//         let node = if bit {
//             HuffmanNode {
//                 frequency: 0,
//                 byte: Some(reader.read_byte()),
//                 left: None,
//                 right: None
//             }
//         } else {
//             nodes_awaiting_children += 1;
//             HuffmanNode {
//                 frequency: 0,
//                 byte: None,
//                 left: None,
//                 right: None
//             }
//         };
//
//         if nodes.is_empty() && node.byte.is_some() {
//             return node;
//         }
//
//         let last_element = nodes.len() - 1;
//         let parent: &mut HuffmanNode = nodes.get_mut(last_element).unwrap();
//         if parent.left.is_none() {
//             parent.left = Some(Box::new(node));
//         } else if parent.right.is_none() {
//             parent.right = Some(Box::new(node));
//             nodes.pop_back();
//             nodes_awaiting_children -= 1;
//         } else {
//             panic!("lol?");
//         }
//
//         if node.byte.is_none() {
//             nodes.push_back(node);
//         }
//
//         if nodes_awaiting_children == 0 {
//             break;
//         }
//     }
//
//     nodes.pop_front().unwrap()
// }

struct HuffmanTargets<'a> {
    input: HuffmanTargetsInput<'a>,
}

enum HuffmanTargetsInput<'a> {
    Content(Vec<u8>),
    File(&'a str),
}

impl<'a> HuffmanTargets<'a> {
    fn new(input_file: Option<&'a str>, input: impl Read) -> Self {
        let input = if let Some(file) = input_file {
            HuffmanTargetsInput::File(file)
        } else {
            let mut reader = BufReader::new(input);
            let mut contents = Vec::new();
            reader.read_to_end(&mut contents).expect("to read");
            HuffmanTargetsInput::Content(contents)
        };

        Self {
            input,
        }
    }

    fn input(&'a self) -> Box<dyn Read + 'a> {
        match self.input {
            HuffmanTargetsInput::Content(ref content) => Box::new(content.as_slice()),
            HuffmanTargetsInput::File(ref file) => Box::new(std::fs::File::open(file).expect("file not found"))
        }
    }
}

#[derive(Debug)]
struct Bits {
    data: u32,
    amount_of_bits: u8
}

impl Display for Bits {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        let mut string = String::new();
        // TODO: is there another way to do this?
        let mut mask = 0b10000000000000000000000000000000;
        for _ in 0..self.amount_of_bits {
            if self.data & mask == mask {
                string = format!("{}1", string);
            } else {
                string = format!("{}0", string);
            }
            mask = mask >> 1;
        }
        write!(fmt, "{}", string)
    }
}

impl Bits {
    fn empty() -> Self {
        Self { data: 0b0, amount_of_bits: 0 }
    }

    fn byte(byte: u8) -> Self {
        Self { data: (byte as u32) << 24, amount_of_bits: 8 }
    }

    fn add(&self, bit: bool) -> Self {
        if self.amount_of_bits == 32 {
            panic!("cannot add more than 32 bits");
        }

        if !bit {
            Self {
                data: self.data,
                amount_of_bits: self.amount_of_bits + 1
            }
        } else {
            // TODO: is there another way to do this?
            let mut mask = 0b10000000000000000000000000000000;
            mask = mask >> self.amount_of_bits;
            Self {
                data: mask | self.data,
                amount_of_bits: self.amount_of_bits + 1
            }
        }
    }
}

struct BitsWriter<T: Write> {
    writer: T,
    mask: u8,
    shift: u8,
    current_byte: u8
}

impl<T: Write> BitsWriter<T> {
    fn new(writer: T) -> Self {
        Self {
            writer,
            mask: 0b10000000,
            shift: 0,
            current_byte: 0b0
        }
    }

    fn write(&mut self, bits: &Bits) {
        for i in 0..bits.amount_of_bits {
            if self.mask == 0b00000000 {
                self.flush();
            }

            self.current_byte = ((((bits.data << i) >> 24) as u8 >> self.shift) & self.mask) | self.current_byte;
            self.mask = self.mask >> 1;
            self.shift += 1;
        }
    }

    fn flush(&mut self) {
        let mut buf = [self.current_byte];
        self.writer.write(&mut buf).expect("to write");
        self.current_byte = 0b00000000;
        self.mask = 0b10000000;
        self.shift = 0;
    }

    fn final_flush_with_offset(&mut self) {
        let mut buf = [self.mask];
        if self.mask != 0b10000000 {
            self.flush();
        }
        self.writer.write(&mut buf).expect("to write");
    }
}

impl<T: Write> Drop for BitsWriter<T> {
    fn drop(&mut self) {
        self.final_flush_with_offset();
    }
}

struct BitsReader<T: Read> {
    reader: T,
    mask: u8,
    current_byte: u8,
    next_byte: u8,
    next_next_byte: Option<u8>,
}
impl<T: Read> BitsReader<T> {
    fn new(mut reader: T) -> Self {
        let mut buf = [0u8; 1];
        reader.read_exact(&mut buf).expect("to have at least two bytes");
        let current_byte = buf[0];
        reader.read_exact(&mut buf).expect("to have at least two bytes");
        let next_byte = buf[0];
        let next_next_byte = reader.read_exact(&mut buf).ok().map(|_| buf[0]);

        Self {
            reader,
            mask: 0b10000000,
            current_byte,
            next_byte,
            next_next_byte
        }
    }

    fn read_byte(&mut self) -> u8 {
        // TODO: improve this performance
        (0..8).fold(0, |acc, _| {
            acc << 1 | if self.read() { 1 } else { 0 }
        })
    }

    fn read(&mut self) -> bool {
        if self.next_next_byte.is_none() && self.mask == self.next_byte {
            panic!("should not call read after end of file");
        }

        if self.mask == 0b00000000 {
            self.current_byte = self.next_byte;
            self.next_byte = self.next_next_byte.expect("not to call read after end of file");
            let mut buf = [0u8; 1];
            self.next_next_byte = self.reader.read(&mut buf).ok().map(|_| buf[0]);
            self.mask = 0b10000000;
        }

        let result = self.current_byte & self.mask == self.mask;
        self.mask = self.mask >> 1;
        result
    }

    fn read_safe(&mut self) -> Option<bool> {
        if self.next_next_byte.is_none() && self.mask == self.next_byte {
            return None;
        }

        if self.mask == 0b00000000 {
            self.current_byte = self.next_byte;
            self.next_byte = self.next_next_byte.expect("not to call read after end of file");
            let mut buf = [0u8; 1];
            self.next_next_byte = self.reader.read_exact(&mut buf).ok().map(|_| buf[0]);
            self.mask = 0b10000000;
        }

        let result = self.current_byte & self.mask == self.mask;
        self.mask = self.mask >> 1;
        Some(result)
    }
}

fn get_options<'a>(args: &[&'a str]) -> HuffmanOptions<'a> {
    let mut mode = HuffmanMode::Encode;
    let mut input_file = None;
    let mut output_file = None;

    let mut args = args.iter();
    loop {
        let arg = args.next();
        let Some(&arg) = arg else {
            break;
        };

        if arg == "--decode" {
            mode = HuffmanMode::Decode;
            continue;
        }

        if arg == "--encode" {
            mode = HuffmanMode::Encode;
            continue;
        }

        if arg == "--input" {
            input_file = args.next().map(|s| *s);
            continue;
        }

        if arg == "--output" {
            output_file = args.next().map(|s| *s);
            continue;
        }
    }

    HuffmanOptions {
        input_file,
        output_file,
        mode
    }
}

struct HuffmanOptions<'a> {
    input_file: Option<&'a str>,
    output_file: Option<&'a str>,
    mode: HuffmanMode
}

enum HuffmanMode { Encode, Decode }

#[derive(Debug)]
struct HuffmanPrefixCodeTable(HashMap<u8, Bits>);

impl Display for HuffmanPrefixCodeTable {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        let mut string = String::new();
        for (b, bits) in &self.0 {
            string = format!("{}\n{} - {:08b} -> {}", string, *b as char, b, bits);
        }
        write!(fmt, "{}", string)
    }
}

#[derive(Debug, Clone)]
struct HuffmanNode {
    frequency: usize,
    byte: Option<u8>,
    left: Option<Box<HuffmanNode>>,
    right: Option<Box<HuffmanNode>>,
}

impl PartialEq<Self> for HuffmanNode {
    fn eq(&self, other: &Self) -> bool {
        self.frequency == other.frequency
    }
}

impl PartialOrd<Self> for HuffmanNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for HuffmanNode {}

impl Ord for HuffmanNode {
    fn cmp(&self, other: &Self) -> Ordering {
        self.frequency.cmp(&other.frequency)
    }
}

#[cfg(test)]
mod tests {
    use std::env;

    use super::*;

    #[test]
    fn find_frequencies() {
        let mut input: &[u8] = b"test";

        let frequencies = huffman_frequencies(&mut input);

        assert_eq!(frequencies[&b't'], 2);
        assert_eq!(frequencies[&b'e'], 1);
        assert_eq!(frequencies[&b's'], 1);
    }

    #[test]
    fn find_frequencies_test_file() {
        let mut test_file = std::fs::File::open("src/huffman/test.txt").expect("file not found");

        let frequencies = huffman_frequencies(&mut test_file);

        assert_eq!(frequencies[&b'X'], 333);
        assert_eq!(frequencies[&b't'], 223000);
    }

    #[test]
    fn frequencies_to_tree_to_prefix_code_table() {
        let mut input: &[u8] = b"testts";

        let frequencies = huffman_frequencies(&mut input);
        let root = huffman_tree(frequencies);
        let table = huffman_prefix_code_table(root);

        let t_prefix = table.0.get(&b't').unwrap();
        let e_prefix = table.0.get(&b'e').unwrap();
        let s_prefix = table.0.get(&b's').unwrap();

        assert_eq!(t_prefix.data, 0b0);
        assert_eq!(e_prefix.data, 0b10000000);
        assert_eq!(s_prefix.data, 0b11000000);
    }

    #[test]
    fn bit_write_and_read() {
        let mut output = Vec::new();

        let mut bits = Bits::empty();
        bits = bits.add(true);
        bits = bits.add(true);
        bits = bits.add(false);
        bits = bits.add(true);
        bits = bits.add(false);
        bits = bits.add(true);
        bits = bits.add(false);
        bits = bits.add(true);
        let mut bits2 = Bits::empty();
        bits2 = bits2.add(false);
        bits2 = bits2.add(true);
        let mut writer = BitsWriter::new(&mut output);
        writer.write(&bits);
        writer.write(&bits2);
        drop(writer);

        let input: &[u8] = &output;
        let mut reader = BitsReader::new(input);
        assert_eq!(reader.read(), true);
        assert_eq!(reader.read(), true);
        assert_eq!(reader.read(), false);
        assert_eq!(reader.read(), true);
        assert_eq!(reader.read(), false);
        assert_eq!(reader.read(), true);
        assert_eq!(reader.read(), false);
        assert_eq!(reader.read(), true);
        assert_eq!(reader.read(), false);
        assert_eq!(reader.read(), true);
    }

    #[test]
    fn encode_decode_should_return_original_input() {
        let mut input: &[u8] = b"super long string here woooooo";
        let original_length = input.len();
        let mut output = Vec::new();

        huffman_cli_impl(&["--encode"], &mut input, &mut output);

        let mut input: &[u8] = &output;
        let new_length = input.len();
        let mut output = Vec::new();
        huffman_cli_impl(&["--decode"], &mut input, &mut output);

        assert!(original_length < new_length);
        assert_eq!(String::from_utf8(output.clone()).expect("to do it"), "super long string here woooooo".to_string());
    }

    #[test]
    fn encode_decode_file_should_return_original_input() {
        env::set_var("RUST_BACKTRACE", "1");
        huffman_cli_impl(&["--encode", "--input", "src/huffman/small_test.txt", "--output", "src/huffman/small_test.huffman"], stdin(), stdout());
        huffman_cli_impl(&["--decode", "--input", "src/huffman/small_test.huffman", "--output", "src/huffman/small_test_result.txt"], stdin(), stdout());

        let mut initial_file = std::fs::File::open("src/huffman/small_test.txt").expect("file not found");
        let mut result_file = std::fs::File::open("src/huffman/small_test_result.txt").expect("file not found");

        let mut initial_content = Vec::new();
        initial_file.read_to_end(&mut initial_content).expect("to work");

        let mut result_content = Vec::new();
        result_file.read_to_end(&mut result_content).expect("to work");
        
        let initial = String::from_utf8(initial_content).expect("to do it");
        let result = String::from_utf8(result_content).expect("to do it");

        assert_eq!(result, initial);
    }
}
