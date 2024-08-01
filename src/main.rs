mod analyser;
mod scrubber;
mod storage;
mod test;
use analyser::Analyser;
use std::fs;

pub fn main() {
    let mut analyser = Analyser::default();
    analyser.deduplicate("test_files_input/lowinput.txt", "lowout.txt");
    fs::remove_file("lowout.txt").expect("File lowout.txt not exists in current directory");
}
