use super::bits::BitsReader;
use super::targets::HuffmanTargets;
use std::error::Error;
use std::io::{Read, Write};

pub fn decode(targets: HuffmanTargets) -> Result<(), Box<dyn Error>> {
    let (input, output) = targets.take();
    let mut output = output.take();

    let mut reader = BitsReader::new(input.take())?;
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
            output.write(&[current_node.byte.unwrap()])?;
            current_node = &root;
        }
    }

    Ok(())
}

fn decode_tree<T: Read>(reader: &mut BitsReader<T>) -> HuffmanNode {
    let bit = reader.read();

    if bit {
        HuffmanNode {
            byte: Some(reader.read_byte()),
            left: None,
            right: None,
        }
    } else {
        let left = decode_tree(reader);
        let right = decode_tree(reader);
        HuffmanNode {
            byte: None,
            left: Some(Box::new(left)),
            right: Some(Box::new(right)),
        }
    }
}

#[derive(Debug)]
struct HuffmanNode {
    byte: Option<u8>,
    left: Option<Box<HuffmanNode>>,
    right: Option<Box<HuffmanNode>>,
}
