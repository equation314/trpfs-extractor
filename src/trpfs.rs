use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::{self, prelude::*, SeekFrom};
use std::path::Path;

use crate::kraken;
use crate::trpfd::Trpfd;
use crate::utils::*;

pub struct Trpfs {
    trpfd: Trpfd,

    pack_offsets: Vec<u64>,
    pack_hash_to_id: BTreeMap<u64, usize>,

    file: File,
}

struct TrpfsCompression;

impl TrpfsCompression {
    fn extract(file: &mut File, out_fname: &Path) -> io::Result<()> {
        const MAX_SIZE: u32 = 1024 * 1024 * 1024;

        let _unused = file_read_u32(file)?;
        let comp_type = file_read_u32(file)?;
        let header_size = file_read_u32(file)?;
        let uncomp_size = file_read_u64(file)? as u32;
        file.seek(SeekFrom::Current(header_size as i64 - 12))?;
        let comp_size = file_read_u32(file)?;

        if comp_size > MAX_SIZE {
            panic!(
                "Too large compressed data size: {} > {}",
                comp_size, MAX_SIZE
            );
        }
        if uncomp_size > MAX_SIZE {
            panic!(
                "Too large uncompressed data size: {} > {}",
                uncomp_size, MAX_SIZE
            );
        }
        if comp_type != 0x0403_0000 && comp_type != 0xff00_0000 {
            panic!("Unsupported compression type {:#x}", comp_type);
        }

        let mut comp_data = vec![];
        file.take(comp_size as _).read_to_end(&mut comp_data)?;
        let uncomp_data = if comp_type == 0x0403_0000 {
            // The decompressor will write outside of the target buffer.
            let mut buf = vec![0; uncomp_size as usize + 64];
            // oodle kraken compressed data
            kraken::decompress(&comp_data, &mut buf[..uncomp_size as _]).unwrap();
            buf
        } else if comp_type == 0xff00_0000 {
            // no compression
            comp_data
        } else {
            unreachable!()
        };

        fs::write(out_fname, uncomp_data)?;
        println!("{} => {}", comp_size, uncomp_size);
        Ok(())
    }
}

impl Trpfs {
    pub fn open(fname: &str, trpfd: Trpfd) -> io::Result<Self> {
        let mut file = File::open(fname)?;
        let magic = file_read_u64(&mut file)?.to_le_bytes();
        let offsets_start = file_read_u64(&mut file)?;
        if &magic != b"ONEPACK\0" {
            println!("magic number not match: {:x?}", magic);
            return Err(io::ErrorKind::InvalidData.into());
        }

        file.seek(SeekFrom::Start(offsets_start + 28))?;
        let pack_offsets = file_read_vec_u64(&mut file)?;
        file.seek(SeekFrom::Current(4))?;
        let pack_hashes = file_read_vec_u64(&mut file)?;
        assert_eq!(pack_hashes.len(), trpfd.num_packs());
        assert_eq!(pack_hashes.len(), trpfd.num_packs());

        let mut pack_hash_to_id = BTreeMap::new();
        for (i, h) in pack_hashes.into_iter().enumerate() {
            pack_hash_to_id.insert(h, i);
        }

        Ok(Self {
            trpfd,
            pack_offsets,
            pack_hash_to_id,
            file,
        })
    }

    pub fn extract(&mut self, pack_id: usize, outdir: &Path) -> io::Result<()> {
        assert!(pack_id < self.trpfd.num_packs());

        let pack_info = self.trpfd.pack_info_at(pack_id);
        let idx = *self
            .pack_hash_to_id
            .get(&pack_info.hash)
            .unwrap_or_else(|| {
                panic!(
                    "Pack {} ({:016x}) not found",
                    pack_info.name, pack_info.hash
                )
            });

        let outdir = outdir.join(&pack_info.name);
        fs::create_dir_all(&outdir)?;

        let pack_start = self.pack_offsets[idx];
        println!(
            "Extracting {} {}: ({:#x}, {}, {})",
            pack_id, pack_info.name, pack_start, pack_info.size, pack_info.num_files
        );

        self.file.seek(SeekFrom::Start(pack_start))?;
        let header_size = file_read_u32(&mut self.file)?;
        self.file.seek(SeekFrom::Current(header_size as i64))?;
        let hashes_start = file_read_u32(&mut self.file)? + header_size + 4;

        self.file.seek(SeekFrom::Current(4))?;
        let file_offsets = file_read_abs_offsets(&mut self.file, header_size + 12)?;
        assert_eq!(file_offsets.len(), pack_info.num_files);

        self.file
            .seek(SeekFrom::Start(pack_start + hashes_start as u64))?;
        let hashes = file_read_vec_u64(&mut self.file)?;
        assert_eq!(hashes.len(), pack_info.num_files);

        for (i, off) in file_offsets.into_iter().enumerate() {
            print!("  {:016x}: ", hashes[i]);
            self.file.seek(SeekFrom::Start(pack_start + off as u64))?;
            TrpfsCompression::extract(&mut self.file, &outdir.join(format!("{:016x}", hashes[i])))?;
        }

        Ok(())
    }
}
