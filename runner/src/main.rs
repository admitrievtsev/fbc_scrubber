extern crate fbc_scrubber;
extern crate chunkfs;

use fbc_chunker::{FBCMap, FBCScrubber};
use std::io;
use chunkfs::FileSystem;
use std::collections::HashMap;
use chunkfs::hashers::Sha256Hasher;
use chunkfs::chunkers::SuperChunker;

fn main() {
    /*
    let mut analyser = Analyser::default();
    analyser.deduplicate("test_files_input/lowinput.txt", "lowout.txt");
    fs::remove_file("lowout.txt").expect("File lowout.txt not exists in current directory");
    */


}
