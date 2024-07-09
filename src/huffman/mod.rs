use std::cmp::{Ordering, Reverse};
use std::collections::{BinaryHeap, HashMap};
use std::io::{BufRead, BufReader, Read, stdin, Write};

// https://codingchallenges.fyi/challenges/challenge-huffman

pub fn huffman_cli(args: &[&str]) {
    huffman_cli_impl(args, &mut stdin());
}

fn huffman_cli_impl(args: &[&str], input: &mut impl Read) {
    let options = get_options(args);

    if let HuffmanMode::Encode = options.mode {
        let frequencies = if let Some(input_file) = options.input_file {
            let mut file = std::fs::File::open(input_file).expect("file not found");
            huffman_frequencies(&mut file)
        } else {
            huffman_frequencies(input)
        };
        let root = huffman_tree(frequencies);
        let table = huffman_prefix_code_table(root.clone());
        if let Some(input_file) = options.input_file {
            let file = std::fs::File::open(input_file).expect("file not found");
            write_huffman_file(options, file, table, root);
        } else {
            write_huffman_file(options, input, table, root);
        };
    } else {
        if let Some(input_file) = options.input_file {
            let file = std::fs::File::open(input_file).expect("file not found");
            read_huffman_file(options, file);
        } else {
            read_huffman_file(options, input);
        };
    }
}

fn write_huffman_file(options: HuffmanOptions, input: impl Read, table: HuffmanPrefixCodeTable, root: HuffmanNode) {
    let mut output = std::fs::File::create(options.output_file).expect("cannot create output file");

    let mut nodes_to_process: Vec<HuffmanNode> = vec![root];
    while nodes_to_process.len() > 0 {
        let node = nodes_to_process.pop().unwrap();

        if let Some(byte) = node.byte {
            output.write(&[0b1, byte]).expect("cannot write output");
        } else {
            output.write(&[0b0]).expect("cannot write output");
            if let Some(left) = node.left {
                nodes_to_process.push(*left);
            }
            if let Some(right) = node.right {
                nodes_to_process.push(*right);
            }
        }
    }

    output.write(&input.bytes()
        .map(|b| *table.0.get(&b.unwrap()).unwrap())
        .collect::<Vec<u8>>()).expect("done");
}

fn read_huffman_file(options: HuffmanOptions, input: &mut impl Read) {
    let root = decode_tree(input);

    let mut output = std::fs::File::create(options.output_file).expect("cannot create output file");
    
    let current_node: &HuffmanNode;
    
    let mut buf = Vec::new();
    while input.read(&mut buf).expect("cannot read") != 0 {
        for byte in buf {
            
        }
    }
}

fn decode_tree(input: &mut impl Read) -> HuffmanNode {
    let mut buf = vec![0u8; 1];
    input.read_exact(&mut buf).expect("read byte");
    
    if buf[0] == 0b1 {
        input.read_exact(&mut buf).expect("read byte");
        HuffmanNode {
            frequency: 0,
            byte: Some(buf[0]),
            left: None,
            right: None
        }
    } else {
        let left = decode_tree(input);
        let right = decode_tree(input);
        HuffmanNode {
            frequency: 0,
            byte: None,
            left: Some(Box::new(left)),
            right: Some(Box::new(right))
        }
    }
}

fn get_options<'a>(args: &[&str]) -> HuffmanOptions<'a> {
    HuffmanOptions {
        input_file: Some("src/huffman/test.txt"),
        output_file: "src/huffman/test.txt.huffman",
        mode: HuffmanMode::Encode
    }
}

struct HuffmanOptions<'a> { input_file: Option<&'a str>, output_file: &'a str, mode: HuffmanMode }
enum HuffmanMode { Encode, Decode }

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
    let mut nodes_to_process: Vec<(HuffmanNode, u8)> = vec![(root, 0)];
    while nodes_to_process.len() > 0 {
        let node_with_prefix = nodes_to_process.pop().unwrap();

        if let Some(byte) = node_with_prefix.0.byte {
            prefix_code_table.insert(byte, node_with_prefix.1);
        } else {
            if let Some(left) = node_with_prefix.0.left {
                nodes_to_process.push((*left, node_with_prefix.1 * 2));
            }
            if let Some(right) = node_with_prefix.0.right {
                nodes_to_process.push((*right, node_with_prefix.1 * 2 + 1));
            }
        }
    }

    HuffmanPrefixCodeTable(prefix_code_table)
}

#[derive(Debug)]
struct HuffmanPrefixCodeTable(HashMap<u8, u8>);

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

fn huffman_frequencies(input: &mut impl Read) -> HashMap<u8, usize> {
    let mut frequencies: HashMap<u8, usize> = HashMap::new();

    let mut reader = BufReader::new(input);
    let mut buf = Vec::<u8>::new();

    while reader.read_until(b'\n', &mut buf).expect("read_until failed") != 0 {
        let line = String::from_utf8(buf).expect("from_utf8 failed");
        for byte in line.bytes() {
            let frequency = frequencies.entry(byte).or_insert(0);
            *frequency += 1;
        }
        buf = line.into_bytes();
        buf.clear();
    }

    frequencies
}

#[cfg(test)]
mod tests {
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

        let t_prefix = table.0[&b't'];
        let e_prefix = table.0[&b'e'];
        let s_prefix = table.0[&b's'];

        assert_eq!(t_prefix, 0b0);
        assert_eq!(e_prefix, 0b10);
        assert_eq!(s_prefix, 0b11);
    }
}
