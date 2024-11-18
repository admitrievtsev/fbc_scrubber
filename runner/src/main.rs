extern crate chunkfs;
extern crate fbc_scrubber;

use chunkfs::chunkers::SuperChunker;
use chunkfs::hashers::Sha256Hasher;
use chunkfs::FileSystem;
use fbc_scrubber::analyser::Analyser;
use fbc_scrubber::storage::FBCMap;
use fbc_scrubber::FBCScrubber;
use std::collections::HashMap;
use std::{fs, io};

fn main() -> io::Result<()> {
    /*
        let mut analyser = Analyser::default();
    analyser.deduplicate("files/lowinput.txt", "lowout.txt");
     */

    let mut fs = FileSystem::new_with_scrubber(
        HashMap::default(),
        Box::new(FBCMap::new()),
        Box::new(FBCScrubber::new()),
        Sha256Hasher::default(),
    );
    let mut handle = fs.create_file("file".to_string(), SuperChunker::new(), true)?;
    let data = fs::read("files/emails_test.csv").expect("Should have been able to read the file");

    //fs::remove_file("lowout.txt").expect("File lowout.txt not exists in current directory");


    fs.write_to_file(&mut handle, &data)?;
    fs.close_file(handle)?;

    let res = fs.scrub()?;
    println!("{res:?}");

    let mut handle = fs.open_file("file", SuperChunker::new())?;
    let read = fs.read_file_complete(&mut handle)?;
    assert_eq!(read.len(), data.len());

    Ok(())
}
