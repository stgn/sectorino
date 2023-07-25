#![feature(seek_stream_len)]

use std::fs::File;
use std::io::{BufReader, BufWriter, Write};
use std::time::Instant;

use num_format::{Locale, ToFormattedString};
use structopt::StructOpt;

mod dedupe;
mod rolling;

#[derive(StructOpt)]
struct Cli {
    #[structopt(parse(from_os_str))]
    path: std::path::PathBuf,
    #[structopt(short = "b", default_value = "11")]
    block_size_log2: u8,
}

fn main() -> Result<(), std::io::Error> {
    let args = Cli::from_args();

    let file = File::open(&args.path)?;
    let mut reader = BufReader::new(file);

    eprintln!("indexing blocks");
    let block_index = dedupe::hash_blocks(&mut reader, args.block_size_log2).unwrap();

    eprintln!("deduplicating");
    let start = Instant::now();
    let remap = dedupe::dedupe(&mut reader, args.block_size_log2, block_index).unwrap();
    let duration = start.elapsed();

    let remap_file = File::create(&args.path.with_extension("remap"))?;

    let mut remap_writer = BufWriter::new(remap_file);
    for offset in &remap {
        remap_writer.write(&offset.to_le_bytes()[..])?;
    }

    let bytes_saved = remap.iter().filter(|&&x| x > 0).count() << args.block_size_log2;

    eprintln!(
        "deduplicated {} bytes in {:?}",
        bytes_saved.to_formatted_string(&Locale::en),
        duration
    );

    Ok(())
}
