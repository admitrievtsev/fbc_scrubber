mod analyser;
mod test;

use analyser::Analyser;

pub fn main() {
    let mut analyser = Analyser::default();
    analyser.deduplicate(
        "test_files_input/fbc_topic_input.txt",
        "test_files_output/fbc_topic_output.txt",
    );
}
