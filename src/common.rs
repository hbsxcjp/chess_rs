#![allow(dead_code)]

use std::error;
use std::fmt;

pub type Result<T> = std::result::Result<T, ParseError>;

#[derive(Clone, Debug)]
pub enum ParseError {
    RowOut,
    ColOut,
    IndexOut,
    StringParse,
    RecordTypeError,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "invalid value(kind: {:?}) to coord.", self)
    }
}

impl error::Error for ParseError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        None
    }
}

// use std::convert::TryInto;
use crate::coord::CoordPair;

fn read_bytes(input: &mut &[u8], size: usize) -> Vec<u8> {
    let (bytes, rest) = input.split_at(size);
    *input = rest;

    bytes.to_vec()
}

pub fn write_coordpair(output: &mut Vec<u8>, coordpair: &CoordPair) {
    let (frow, fcol, trow, tcol) = coordpair.row_col();
    output.append(&mut vec![frow as u8, fcol as u8, trow as u8, tcol as u8]);
}

pub fn write_be_u32(output: &mut Vec<u8>, size: u32) {
    output.append(&mut size.to_be_bytes().to_vec());
}

pub fn write_string(output: &mut Vec<u8>, string: &str) {
    write_be_u32(output, string.len() as u32);
    output.append(&mut string.as_bytes().to_vec());
}

pub fn read_coordpair(input: &mut &[u8]) -> CoordPair {
    let bytes = read_bytes(input, 4);

    CoordPair::from_row_col(
        bytes[0] as usize,
        bytes[1] as usize,
        bytes[2] as usize,
        bytes[3] as usize,
    )
    .unwrap()
}

pub fn read_be_u32(input: &mut &[u8]) -> u32 {
    let bytes = read_bytes(input, std::mem::size_of::<u32>());

    u32::from_be_bytes(bytes.try_into().unwrap())
}

pub fn read_string(input: &mut &[u8]) -> String {
    let size = read_be_u32(input) as usize;
    let bytes = read_bytes(input, size);

    String::from_utf8(bytes.try_into().unwrap()).unwrap()
}
