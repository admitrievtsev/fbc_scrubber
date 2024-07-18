mod analyser;
mod test;
mod storage;
mod scrubber;

use analyser::Analyser;

pub fn main() {
    let mut analyser = Analyser::default();
    analyser.deduplicate(
        "test_files_input/lowinput.txt",
        "test_files_output/lowout.txt",
    );
}
