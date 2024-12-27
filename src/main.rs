extern crate chunkfs;
extern crate fbc_scrubber;

use chunkfs::chunkers::{SizeParams, SuperChunker};
use chunkfs::hashers::Sha256Hasher;
use chunkfs::FileSystem;
use fbc_scrubber::fbc_chunker::ChunkerFBC;

use fbc_scrubber::frequency_analyser::FrequencyAnalyser;
use fbc_scrubber::storage::FBCMap;
use fbc_scrubber::FBCScrubber;
use std::collections::HashMap;
use std::{fs, io};

fn main() -> io::Result<()> {
    /*
    TO RUN FROM OTHER SOURCE
    let mut fs = FileSystem::new_with_scrubber(
        HashMap::default(),
        FBCMap::new(),
        Box::new(FBCScrubber::new()),
        Sha256Hasher::default(),
    );
    let chunk_size = SizeParams::new(2000, 12000, 17000);
    let mut handle = fs.create_file("file".to_string(), SuperChunker::new(chunk_size))?;
    let data = fs::read("files/emails_test.csv").expect("Should have been able to read the file");

    fs.write_to_file(&mut handle, &data)?;
    fs.close_file(handle)?;

    let res = fs.scrub()?;
    println!("{res:?}");

    let mut handle = fs.open_file("file", SuperChunker::new(chunk_size))?;
    let read = fs.read_file_complete(&mut handle)?;
    assert_eq!(read.len(), data.len());



     */
    Ok(())
}
