use std::fs::File;
use std::io::{self, Read};

pub fn fnv1a64_hash(buf: &[u8]) -> u64 {
    const PRIME: u64 = 0x00000100000001B3;
    const BASIS: u64 = 0xcbf29ce484222645;

    let mut h = BASIS;
    for c in buf {
        h ^= *c as u64;
        h = h.wrapping_mul(PRIME);
    }
    h
}

pub fn file_read_string(file: &mut File) -> io::Result<String> {
    let len = file_read_u32(file)? as _;
    let mut s = String::with_capacity(len);
    file.take(len as _).read_to_string(&mut s)?;
    Ok(s)
}

pub fn file_read_vec_u32(file: &mut File) -> io::Result<Vec<u32>> {
    let len = file_read_u32(file)? as _;
    let mut v = Vec::with_capacity(len);
    for _ in 0..len {
        v.push(file_read_u32(file)?);
    }
    Ok(v)
}

pub fn file_read_vec_u64(file: &mut File) -> io::Result<Vec<u64>> {
    let len = file_read_u32(file)? as _;
    let mut v = Vec::with_capacity(len);
    for _ in 0..len {
        v.push(file_read_u64(file)?);
    }
    Ok(v)
}

pub fn file_read_u32(file: &mut File) -> io::Result<u32> {
    let mut buf = [0; 4];
    file.read_exact(&mut buf)?;
    Ok(u32::from_le_bytes(buf))
}

pub fn file_read_u64(file: &mut File) -> io::Result<u64> {
    let mut buf = [0; 8];
    file.read_exact(&mut buf)?;
    Ok(u64::from_le_bytes(buf))
}

pub fn file_read_abs_offsets(file: &mut File, offsets_start: u32) -> io::Result<Vec<u32>> {
    let mut rel_offsets = file_read_vec_u32(file)?;
    for (i, off) in rel_offsets.iter_mut().enumerate() {
        *off += offsets_start + 4 + i as u32 * 4;
    }
    Ok(rel_offsets)
}
