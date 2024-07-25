use build_your_own_shared::my_own_error::MyOwnError;

use super::bits::{Bits, BitsWriter};
use super::targets::HuffmanInput;
use std::cmp::{Ordering, Reverse};
use std::collections::BinaryHeap;
use std::fmt::Display;
use std::fmt::Formatter;
use std::io::{BufRead, BufReader, Read, Write};

pub fn encode(input: HuffmanInput, output: &mut impl Write) -> Result<(), MyOwnError> {
    let frequencies = huffman_frequencies(&mut input.take())?;
    let root = huffman_tree(frequencies);
    let table = huffman_prefix_code_table(root.clone());

    write_huffman_file(&mut input.take(), output, table, root)?;

    Ok(())
}

fn huffman_frequencies(input: &mut impl Read) -> Result<[usize; 256], MyOwnError> {
    let mut frequencies: [usize; 256] = [0; 256];
    let mut buf = vec![0u8; 65536];

    loop {
        let count = input.read(&mut buf)?;

        if count == 0 {
            break;
        }

        buf[..count]
            .iter()
            .copied()
            .for_each(|b| frequencies[b as usize] += 1);
        buf.clear();
    }

    Ok(frequencies)
}

fn huffman_tree(frequencies: [usize; 256]) -> HuffmanNode {
    let mut nodes = BinaryHeap::<Reverse<HuffmanNode>>::new();
    for (index, frequency) in frequencies.into_iter().enumerate().filter(|e| e.1 > 0) {
        nodes.push(Reverse(HuffmanNode {
            frequency,
            byte: Some(index as u8),
            left: None,
            right: None,
        }));
    }

    while nodes.len() > 1 {
        let node1 = nodes.pop().unwrap().0;
        let node2 = nodes.pop().unwrap().0;
        nodes.push(Reverse(HuffmanNode {
            frequency: node1.frequency + node2.frequency,
            byte: None,
            left: Some(Box::new(node1)),
            right: Some(Box::new(node2)),
        }));
    }

    nodes.pop().unwrap().0
}

fn huffman_prefix_code_table(root: HuffmanNode) -> HuffmanPrefixCodeTable {
    let mut prefix_code_table: [Bits; 256] = [Bits::empty(); 256];
    let mut nodes_to_process: Vec<(HuffmanNode, Bits)> = vec![(root, Bits::empty())];
    while nodes_to_process.len() > 0 {
        let node_with_prefix = nodes_to_process.pop().unwrap();

        if let Some(byte) = node_with_prefix.0.byte {
            prefix_code_table[byte as usize] = node_with_prefix.1;
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

fn write_huffman_file(
    input: &mut impl Read,
    output: &mut impl Write,
    table: HuffmanPrefixCodeTable,
    root: HuffmanNode,
) -> Result<(), MyOwnError> {
    let mut nodes_to_process: Vec<HuffmanNode> = vec![root];
    let mut writer = BitsWriter::new(output);

    while nodes_to_process.len() > 0 {
        let node = nodes_to_process.pop().unwrap();

        if let Some(byte) = node.byte {
            writer.write(&Bits::empty().add(true))?;
            writer.write(&Bits::byte(byte))?;
        } else {
            writer.write(&Bits::empty().add(false))?;
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
        let prefix_code = table.get(&byte);
        writer.write(prefix_code)?;
    }

    Ok(())
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

#[derive(Debug)]
struct HuffmanPrefixCodeTable([Bits; 256]);

impl HuffmanPrefixCodeTable {
    fn get(&self, byte: &u8) -> &Bits {
        &self.0[*byte as usize]
    }
}

impl Display for HuffmanPrefixCodeTable {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        for (b, bits) in self.0.iter().enumerate() {
            write!(fmt, "\n{} - {:08b} -> {}", b as u8 as char, b as u8, bits)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_frequencies() {
        let mut input: &[u8] = b"test";

        let frequencies = huffman_frequencies(&mut input).unwrap();

        assert_eq!(frequencies[b't' as usize], 2);
        assert_eq!(frequencies[b'e' as usize], 1);
        assert_eq!(frequencies[b's' as usize], 1);
    }

    #[test]
    fn find_frequencies_test_file() {
        let mut test_file = std::fs::File::open("src/huffman/test.txt").unwrap();

        let frequencies = huffman_frequencies(&mut test_file).unwrap();

        assert_eq!(frequencies[b'X' as usize], 333);
        assert_eq!(frequencies[b't' as usize], 223000);
    }

    #[test]
    fn frequencies_to_tree_to_prefix_code_table() {
        let mut input: &[u8] = b"testts";

        let frequencies = huffman_frequencies(&mut input).unwrap();
        let root = huffman_tree(frequencies);
        let table = huffman_prefix_code_table(root);

        let t_prefix = table.get(&b't');
        let e_prefix = table.get(&b'e');
        let s_prefix = table.get(&b's');

        assert_eq!(t_prefix.data, 0b00000000000000000000000000000000);
        assert_eq!(e_prefix.data, 0b10000000000000000000000000000000);
        assert_eq!(s_prefix.data, 0b11000000000000000000000000000000);
    }
}
