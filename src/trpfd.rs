use std::collections::BTreeMap;
use std::fs::File;
use std::io::{self, prelude::*, SeekFrom};
use std::path::Path;

use crate::utils::*;

pub struct Trpfd {
    pack_infos: Vec<PackInfo>,
    file_infos: Vec<FileInfo>,
    file_hash_to_id: BTreeMap<u64, usize>,
}

pub struct PackInfo {
    pub name: String,
    pub size: u64,
    pub num_files: usize,
    pub hash: u64,
}

struct FileInfo {
    pack_id: usize,
    hash: u64,
}

impl Trpfd {
    fn read_pack_infos(file: &mut File, offsets_start: u32) -> io::Result<Vec<PackInfo>> {
        file.seek(SeekFrom::Start(offsets_start as u64))?;
        let offsets = file_read_abs_offsets(file, offsets_start)?;
        let mut infos = Vec::with_capacity(offsets.len());
        for off in offsets {
            file.seek(SeekFrom::Start(off as u64))?;
            let (_, size, num_files, _) = (
                file_read_u32(file)?,
                file_read_u64(file)?,
                file_read_u64(file)? as usize,
                file_read_u32(file)?,
            );
            infos.push(PackInfo {
                name: String::new(),
                size,
                num_files,
                hash: 0,
            });
        }
        Ok(infos)
    }

    fn read_pack_names(file: &mut File, offsets_start: u32) -> io::Result<Vec<String>> {
        file.seek(SeekFrom::Start(offsets_start as u64))?;
        let offsets = file_read_abs_offsets(file, offsets_start)?;
        let mut names = Vec::with_capacity(offsets.len());
        for off in offsets {
            file.seek(SeekFrom::Start(off as u64))?;
            names.push(file_read_string(file)?);
        }
        Ok(names)
    }

    fn read_file_infos(file: &mut File, offsets_start: u32) -> io::Result<Vec<FileInfo>> {
        file.seek(SeekFrom::Start(offsets_start as u64))?;
        let offsets = file_read_abs_offsets(file, offsets_start)?;
        let mut infos = Vec::with_capacity(offsets.len());
        for off in offsets {
            file.seek(SeekFrom::Start(off as u64))?;
            let (_, len) = (file_read_u32(file)?, file_read_u32(file)?);
            let pack_id = if len > 4 {
                file_read_u64(file)? as usize
            } else {
                0
            };
            infos.push(FileInfo { pack_id, hash: 0 })
        }
        Ok(infos)
    }

    pub fn open(fname: &str, load_file_info: bool) -> io::Result<Self> {
        let mut file = File::open(fname)?;

        file.seek(SeekFrom::Start(24))?;
        let hashes_offset = 0x18 + file_read_u32(&mut file)?;
        let packname_offsets_start = 0x1C + file_read_u32(&mut file)?;
        let fileinfo_offsets_start = 0x20 + file_read_u32(&mut file)?;
        let packinfo_offsets_start = 0x24 + file_read_u32(&mut file)?;

        let mut pack_infos = Self::read_pack_infos(&mut file, packinfo_offsets_start)?;
        let mut file_infos = if load_file_info {
            Self::read_file_infos(&mut file, fileinfo_offsets_start)?
        } else {
            vec![]
        };
        let names = Self::read_pack_names(&mut file, packname_offsets_start)?;
        let hashes = if load_file_info {
            file.seek(SeekFrom::Start(hashes_offset as u64))?;
            file_read_vec_u64(&mut file)?
        } else {
            vec![]
        };

        assert_eq!(pack_infos.len(), names.len());
        assert_eq!(hashes.len(), file_infos.len());

        for (i, name) in names.into_iter().enumerate() {
            pack_infos[i].hash = fnv1a64_hash(name.as_bytes());
            pack_infos[i].name = name;
        }

        let mut file_hash_to_id = BTreeMap::new();
        for (i, h) in hashes.into_iter().enumerate() {
            file_infos[i].hash = h;
            file_hash_to_id.insert(h, i);
        }

        Ok(Self {
            pack_infos,
            file_infos,
            file_hash_to_id,
        })
    }

    pub fn num_packs(&self) -> usize {
        self.pack_infos.len()
    }

    pub fn num_files(&self) -> usize {
        self.file_infos.len()
    }

    pub fn pack_info_at(&self, id: usize) -> &PackInfo {
        &self.pack_infos[id]
    }

    #[allow(unused)]
    pub fn lookup_file(&self, file_name: &str) -> Option<(&PackInfo, u64)> {
        let hash = fnv1a64_hash(file_name.as_bytes());
        let file_id = self.file_hash_to_id.get(&hash)?;
        let pack_id = self.file_infos[*file_id].pack_id;
        Some((&self.pack_infos[pack_id], hash))
    }

    pub fn save_pack_info(&self, out_fname: &Path) -> io::Result<()> {
        let mut f = File::create(out_fname)?;
        writeln!(&mut f, "id,size,num_files,name,hash")?;
        for (i, info) in self.pack_infos.iter().enumerate() {
            writeln!(
                &mut f,
                "{},{},{},{},{:016x}",
                i, info.size, info.num_files, info.name, info.hash
            )?;
        }
        Ok(())
    }

    pub fn save_file_info(&self, out_fname: &Path) -> io::Result<()> {
        let mut f = File::create(out_fname)?;
        writeln!(&mut f, "id,pack_id,hash")?;
        for (i, info) in self.file_infos.iter().enumerate() {
            writeln!(&mut f, "{},{},{:016x}", i, info.pack_id, info.hash)?;
        }
        Ok(())
    }
}
