use std::fmt::Display;
use std::fmt::Formatter;
use std::io::{Read, Write};

#[derive(Debug, Copy, Clone)]
pub struct Bits {
    pub data: u32,
    pub amount_of_bits: u8,
}

impl Display for Bits {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        let mut string = String::new();
        let mut mask = 1 << 31;
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
    pub fn empty() -> Self {
        Self {
            data: 0b0,
            amount_of_bits: 0,
        }
    }

    pub fn byte(byte: u8) -> Self {
        Self {
            data: (byte as u32) << 24,
            amount_of_bits: 8,
        }
    }

    pub fn add(&self, bit: bool) -> Self {
        if self.amount_of_bits == 32 {
            panic!("cannot add more than 32 bits");
        }

        if !bit {
            Self {
                data: self.data,
                amount_of_bits: self.amount_of_bits + 1,
            }
        } else {
            let mut mask = 1 << 31;
            mask = mask >> self.amount_of_bits;
            Self {
                data: mask | self.data,
                amount_of_bits: self.amount_of_bits + 1,
            }
        }
    }
}

pub struct BitsWriter<T: Write> {
    writer: T,
    mask: u8,
    shift: u8,
    current_byte: u8,
}

impl<T: Write> BitsWriter<T> {
    pub fn new(writer: T) -> Self {
        Self {
            writer,
            mask: 0b10000000,
            shift: 0,
            current_byte: 0b0,
        }
    }

    pub fn write(&mut self, bits: &Bits) {
        for i in 0..bits.amount_of_bits {
            if self.mask == 0b00000000 {
                self.flush();
            }

            self.current_byte =
                ((((bits.data << i) >> 24) as u8 >> self.shift) & self.mask) | self.current_byte;
            self.mask = self.mask >> 1;
            self.shift += 1;
        }
    }

    pub fn flush(&mut self) {
        let mut buf = [self.current_byte];
        self.writer.write(&mut buf).expect("to write");
        self.current_byte = 0b00000000;
        self.mask = 0b10000000;
        self.shift = 0;
    }

    pub fn final_flush_with_offset(&mut self) {
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

pub struct BitsReader<T: Read> {
    reader: T,
    mask: u8,
    current_byte: u8,
    next_byte: u8,
    next_next_byte: Option<u8>,
}

impl<T: Read> BitsReader<T> {
    pub fn new(mut reader: T) -> Self {
        let mut buf = [0u8; 1];
        reader
            .read_exact(&mut buf)
            .expect("to have at least two bytes");
        let current_byte = buf[0];
        reader
            .read_exact(&mut buf)
            .expect("to have at least two bytes");
        let next_byte = buf[0];
        let next_next_byte = reader.read_exact(&mut buf).ok().map(|_| buf[0]);

        Self {
            reader,
            mask: 0b10000000,
            current_byte,
            next_byte,
            next_next_byte,
        }
    }

    pub fn read_byte(&mut self) -> u8 {
        // NOTE: this is not the most efficient way to do it
        (0..8).fold(0, |acc, _| acc << 1 | if self.read() { 1 } else { 0 })
    }

    pub fn read(&mut self) -> bool {
        if self.next_next_byte.is_none() && self.mask == self.next_byte {
            panic!("should not call read after end of file");
        }

        if self.mask == 0b00000000 {
            self.current_byte = self.next_byte;
            self.next_byte = self
                .next_next_byte
                .expect("not to call read after end of file");
            let mut buf = [0u8; 1];
            self.next_next_byte = self.reader.read(&mut buf).ok().map(|_| buf[0]);
            self.mask = 0b10000000;
        }

        let result = self.current_byte & self.mask == self.mask;
        self.mask = self.mask >> 1;
        result
    }

    pub fn read_safe(&mut self) -> Option<bool> {
        if self.next_next_byte.is_none() && self.mask == self.next_byte {
            return None;
        }

        if self.mask == 0b00000000 {
            self.current_byte = self.next_byte;
            self.next_byte = self
                .next_next_byte
                .expect("not to call read after end of file");
            let mut buf = [0u8; 1];
            self.next_next_byte = self.reader.read_exact(&mut buf).ok().map(|_| buf[0]);
            self.mask = 0b10000000;
        }

        let result = self.current_byte & self.mask == self.mask;
        self.mask = self.mask >> 1;
        Some(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
