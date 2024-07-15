#[cfg(test)]
mod tests {
    use crate::analyser::Analyser;
    use std::fs;

    #[test]
    fn fbc_topic_test() {
        let mut analyser = Analyser::default();
        let file_in = "test_files_input/fbc_topic_input.txt";
        let file_out = "test_files_output/fbc_topic_output.txt";
        analyser.deduplicate(file_in, file_out);
        let expected = fs::read_to_string(file_in).expect("Should have been able to read the file");
        let actual = fs::read_to_string(file_out).expect("Should have been able to read the file");
        assert_eq!(expected, actual);
    }

    #[test]
    fn fbc_orient_express() {
        let mut analyser = Analyser::default();
        let file_in = "test_files_input/orient_express_input.txt";
        let file_out = "test_files_output/orient_express_output.txt";
        analyser.deduplicate(file_in, file_out);
        let expected = fs::read_to_string(file_in).expect("Should have been able to read the file");
        let actual = fs::read_to_string(file_out).expect("Should have been able to read the file");
        assert_eq!(expected, actual);
    }
}
