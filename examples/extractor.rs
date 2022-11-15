use std::path::PathBuf;

use clap::Parser;
use trpfs_extractor::Trpfd;
use trpfs_extractor::Trpfs;

#[derive(Parser, Debug)]
struct Cli {
    trpfd: String,
    trpfs: String,
    #[arg(short, long, action, help = "Extract data")]
    extract: bool,
    #[arg(
        long,
        id = "PACK_INFO_CSV",
        help = "Save all pack info to the CSV file"
    )]
    save_pack_info: Option<PathBuf>,
    #[arg(
        long,
        id = "FILE_INFO_CSV",
        help = "Save all file info to the CSV file"
    )]
    save_file_info: Option<PathBuf>,
    #[arg(short, long, help = "Only extract data from the specified pack")]
    pack_id: Option<usize>,
    #[arg(
        long,
        default_value = "dump",
        help = "Specify the directory to save extracted data"
    )]
    outdir: PathBuf,
}

fn main() {
    let args = Cli::parse();

    let trpfd = Trpfd::open(&args.trpfd, false).expect("Failed to open TRPFD");
    let num_packs = trpfd.num_packs();
    let _num_files = trpfd.num_files();

    println!("Num packs: {}", num_packs);
    // println!("Num files: {}", num_files);

    if let Some(path) = args.save_pack_info {
        trpfd.save_pack_info(&path).unwrap();
    }
    if let Some(path) = args.save_file_info {
        trpfd.save_file_info(&path).unwrap();
    }

    if args.extract {
        let mut trpfs = Trpfs::open(&args.trpfs, trpfd).expect("Failed to open TRPFS");
        for i in 0..num_packs {
            if let Some(pid) = args.pack_id {
                if i != pid {
                    continue;
                }
            }
            trpfs.extract(i, &args.outdir).unwrap();
        }
    }
}
