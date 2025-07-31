#[cfg(test)]
mod tests {
    use fbc_scrubber::fbc_chunker::ChunkerFBC;
    use fbc_scrubber::frequency_analyser::FrequencyAnalyser;
    use std::fs;

    #[test]
    #[ignore]
    fn fbc_topic_test() {
        let mut analyser = FrequencyAnalyser::new();
        let mut chunker = ChunkerFBC::default();
        let contents = fs::read("test_files_input/lowinput.txt")
            .expect("Should have been able to read the file");

        analyser.append_dict(&contents);

        chunker.add_cdc_chunk(&contents[0..1000]);
        chunker.add_cdc_chunk(&contents[1000..3000]);
        chunker.add_cdc_chunk(&contents[3000..4000]);
        chunker.add_cdc_chunk(&contents[4000..5000]);
        chunker.add_cdc_chunk(&contents[5000..5500]);
        chunker.add_cdc_chunk(&contents[5500..6000]);
        chunker.add_cdc_chunk(&contents[6000..7000]);
        chunker.add_cdc_chunk(&contents[7000..contents.len()]);

        chunker.fbc_dedup(analyser.get_dict(), analyser.get_chunk_partitioning());

        let out = chunker.reduplicate(contents.len());

        assert_eq!(
            fs::read("test_files_input/lowinput.txt")
                .expect("Should have been able to read lowinput"),
            out
        );

        fs::remove_file("out.txt").expect("File out.txt not exists in current directory");
    }
}
