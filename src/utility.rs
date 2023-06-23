#![allow(dead_code)]

// use std::convert::TryInto;

pub fn write_be_u32(output: &mut Vec<u8>, size: u32) {
    output.append(&mut size.to_be_bytes().to_vec());
}

pub fn write_string(output: &mut Vec<u8>, string: &str) {
    write_be_u32(output, string.len() as u32);
    output.append(&mut string.as_bytes().to_vec());
}

pub fn read_be_u32(input: &mut &[u8]) -> u32 {
    let (bytes, rest) = input.split_at(std::mem::size_of::<u32>());
    *input = rest;

    u32::from_be_bytes(bytes.try_into().unwrap())
}

pub fn read_string(input: &mut &[u8]) -> String {
    let size = read_be_u32(input) as usize;
    let (bytes, rest) = input.split_at(size);
    *input = rest;

    String::from_utf8(bytes.try_into().unwrap()).unwrap()
}
