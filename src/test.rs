#[cfg(test)]
mod tests {
    use crate::analyser::Analyser;
    use std::fs;

    #[test]
    //BIG-SIZE TEXT
    // (~16000 symbols, 26.3% dedupled)
    fn fbc_topic_test() {
        let mut analyser = Analyser::default();
        let file_in = "test_files_input/fbc_topic_input.txt";
        let file_out = "fbc_topic_output.txt";
        analyser.deduplicate(file_in, file_out);
        let expected = fs::read_to_string(file_in).expect("Should have been able to read the file");
        let actual = fs::read_to_string(file_out).expect("Should have been able to read the file");
        assert_eq!(expected, actual);
        fs::remove_file("fbc_topic_output.txt").expect("File lowout.txt not exists in current directory");
    }

    #[test]
    //SMALL-SIZE TEXT
    // (~8000 symbols, 16.9% dedupled)
    fn fbc_orient_express() {
        let mut analyser = Analyser::default();
        let file_in = "test_files_input/orient_express_input.txt";
        let file_out = "orient_express_output.txt";
        analyser.deduplicate(file_in, file_out);
        let expected = fs::read_to_string(file_in).expect("Should have been able to read the file");
        let actual = fs::read_to_string(file_out).expect("Should have been able to read the file");
        assert_eq!(expected, actual);
        fs::remove_file("orient_express_output.txt").expect("File lowout.txt not exists in current directory");
    }
}
