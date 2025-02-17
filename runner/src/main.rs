use fbc_scrubber::fbc_chunker::ChunkerFBC;
use fbc_scrubber::frequency_analyser::FrequencyAnalyser;
use std::fs;

fn main() {
    let mut analyser = FrequencyAnalyser::new();
    let mut chunker = ChunkerFBC::default();
    let contents = fs::read("../test_files_input/lowinput.txt")
        .expect("Should have been able to read the file");
    analyser.make_dict(&contents);
    chunker.add_cdc_chunk(&contents[0..1000].to_vec());
    chunker.add_cdc_chunk(&contents[1000..3000].to_vec());
    chunker.add_cdc_chunk(&contents[3000..4000].to_vec());
    chunker.add_cdc_chunk(&contents[4000..5000].to_vec());
    chunker.add_cdc_chunk(&contents[5000..5500].to_vec());
    chunker.add_cdc_chunk(&contents[5500..6000].to_vec());
    chunker.add_cdc_chunk(&contents[6000..7000].to_vec());
    chunker.add_cdc_chunk(&contents[7000..contents.len()].to_vec());
    chunker.fbc_dedup(&analyser.get_dict());
    chunker.reduplicate("out.txt");
    if (fs::read("../test_files_input/lowinput.txt")
        .expect("Should have been able to read lowinput")
        == fs::read("out.txt").expect("Should have been able to read out file"))
    {
        println!("MATCH")
    }

    fs::remove_file("out.txt").expect("File out.txt not exists in current directory");
}
