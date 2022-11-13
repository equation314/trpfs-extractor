extern "C" {
    fn Kraken_Decompress(src: *const u8, src_len: usize, dst: *mut u8, dst_len: usize) -> usize;
}

pub fn decompress(src: &[u8], dst: &mut [u8]) -> Result<usize, &'static str> {
    // println!("  decompress {} -> {}", src.len(), dst.len());
    let outbytes =
        unsafe { Kraken_Decompress(src.as_ptr(), src.len(), dst.as_mut_ptr(), dst.len()) };
    if outbytes == dst.len() {
        Ok(outbytes)
    } else {
        Err("decompress failed")
    }
}
