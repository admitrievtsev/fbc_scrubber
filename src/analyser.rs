use std::fs;
use std::string::String;
use std::vec::Vec;

const MAX_CHUNK_SIZE: usize = 24;
const FIXED_CHUNKER_SIZE: usize = 128;
const MIN_CHUNK_SIZE: usize = 7;

macro_rules! inc {
    ($x:expr) => {
        $x += 1
    };
}

type Chunk = Vec<char>;

#[derive(Default)]
struct DictRecord {
    chunk: Chunk,
    occurrence_num: u32,
    size: usize,
}

#[derive(Default)]
pub struct Analyser {
    dict: Vec<DictRecord>, // hashmap? chunk_map
    chunk_ids: Vec<usize>, // hashset?
    chunks: Vec<Chunk>,    // u8 instead
}

impl Analyser {
    fn make_dict(&mut self, first_stage_chunk: Chunk) {
        //rewrite with return dictionary
        let mut temp_chunks: Vec<Chunk> = vec![vec![]; MAX_CHUNK_SIZE - MIN_CHUNK_SIZE];
        for slice_index in MIN_CHUNK_SIZE..MAX_CHUNK_SIZE {
            for char in first_stage_chunk.iter().take(slice_index + 1) {
                temp_chunks[slice_index - MIN_CHUNK_SIZE].push(*char);
            }
        }

        for start_index in 1..first_stage_chunk.len() - MAX_CHUNK_SIZE {
            for chunk_size in MIN_CHUNK_SIZE..MAX_CHUNK_SIZE {
                for char_index in 1..chunk_size + 1 {
                    temp_chunks[chunk_size - MIN_CHUNK_SIZE][char_index - 1] =
                        temp_chunks[chunk_size - MIN_CHUNK_SIZE][char_index]
                }
                temp_chunks[chunk_size - MIN_CHUNK_SIZE][chunk_size] =
                    first_stage_chunk[start_index + chunk_size];
                self.add_chunk(temp_chunks[chunk_size - MIN_CHUNK_SIZE].clone());
            }
        }
    }

    fn tostr(word: &[char]) -> String {
        word.iter().collect()
    }

    fn add_chunk(&mut self, chunk: Chunk) {
        let str_size = chunk.len();
        let mut chunk_dict_id = 0;
        for dict_chunk in self.dict.iter() {
            if dict_chunk.size == str_size {
                for char_index in 0..str_size {
                    if char_index == str_size {
                        inc!(self.dict[chunk_dict_id].occurrence_num);
                        return;
                    }
                    if dict_chunk.chunk[char_index] != chunk[char_index] {
                        break;
                    }
                }
            }
            inc!(chunk_dict_id);
        }
        self.dict.push(DictRecord {
            chunk,
            occurrence_num: 1,
            size: str_size,
        })
    }

    pub fn deduplicate(&mut self, file_in: &str, file_out: &str) {
        self.simple_dedup(file_in);
        self.throw_chunks_to_maker();
        self.fbc_dedup();
        self.reduplicate(file_out);
    }

    fn reduplicate(&self, file_out: &str) {
        let mut string_out = String::new();
        for id in self.chunk_ids.iter() {
            string_out.push_str(&Self::tostr(&self.chunks[*id]));
        }
        fs::write(file_out, string_out).expect("Unable to write the file");
    }

    fn fbc_dedup(&mut self) {
        for dict_index in 0..self.dict.len() {
            for chunk_index in 0..self.chunks.len() {
                if self.dict[dict_index].chunk.len() < self.chunks[chunk_index].len() {
                    for chunk_char in
                        0..self.chunks[chunk_index].len() - self.dict[dict_index].chunk.len()
                    {
                        let mut is_chunk_correct = true;
                        for char_index in 0..self.dict[dict_index].chunk.len() {
                            if self.dict[dict_index].chunk[char_index]
                                != self.chunks[chunk_index][chunk_char + char_index]
                            {
                                is_chunk_correct = false;
                                break;
                            }
                        }

                        if is_chunk_correct {
                            let mut is_found = false;
                            let mut cut_out = self.chunks.len();

                            for chunk_index in 0..self.chunks.len() {
                                if self.chunks[chunk_index] == self.dict[dict_index].chunk {
                                    is_found = true;
                                    cut_out = chunk_index;
                                    break;
                                }
                            }
                            if chunk_char == 0 {
                                if !is_found {
                                    self.chunks.push(self.dict[dict_index].chunk.clone());
                                }
                                self.chunks[chunk_index] =
                                    self.chunks[chunk_index][self.dict[dict_index].chunk.len()
                                        ..self.chunks[chunk_index].len()]
                                        .to_owned();
                                self.replace_all_two(chunk_index, cut_out, chunk_index);
                            } else {
                                if !is_found {
                                    self.chunks.push(self.dict[dict_index].chunk.clone());
                                }
                                self.chunks.push(
                                    self.chunks[chunk_index]
                                        [self.dict[dict_index].size + chunk_char..self.chunks[chunk_index].len()]
                                        .to_owned(),
                                );
                                self.chunks[chunk_index] =
                                    self.chunks[chunk_index][0..chunk_char].to_owned();
                                self.replace_all_three(
                                    chunk_index,
                                    chunk_index,
                                    cut_out,
                                    self.chunks.len() - 1,
                                );
                            }
                            break;
                        }
                    }
                }
            }
        }
    }
    fn replace_all_two(&mut self, to_change: usize, first: usize, second: usize) {
        let mut temp_vec: Vec<usize> = Vec::with_capacity(self.chunks.len() + 1);
        for index in 0..self.chunk_ids.len() {
            if self.chunk_ids[index] == to_change {
                temp_vec.push(first);
                temp_vec.push(second);
            } else {
                temp_vec.push(self.chunk_ids[index]);
            }
        }
        self.chunk_ids = temp_vec
    }

    fn replace_all_three(&mut self, to_change: usize, first: usize, second: usize, third: usize) {
        let mut temp_vec: Vec<usize> = vec![];
        for index in 0..self.chunk_ids.len() {
            if self.chunk_ids[index] == to_change {
                temp_vec.push(first);
                temp_vec.push(second);
                temp_vec.push(third);
            } else {
                temp_vec.push(self.chunk_ids[index]);
            }
        }
        self.chunk_ids = temp_vec
    }
    #[warn(dead_code)]
    fn dict_count_size(&self) -> usize {
        return self.chunks.iter().fold(0, |acc, x| acc + x.len());
    }

    fn simple_dedup(&mut self, f_in: &str) {
        let contents = fs::read_to_string(f_in).expect("Should have been able to read the file");
        let input_length = contents.len();
        let chars: Chunk = contents.chars().collect();
        let mut chunk_num = 0;
        for index_input in 0..input_length {
            if index_input % FIXED_CHUNKER_SIZE == 0 {
                inc!(chunk_num);
                self.chunks.push(vec![]);
                self.chunk_ids.push(chunk_num - 1);
            }
            self.chunks[chunk_num - 1].push(chars[index_input]);
        }
    }

    fn throw_chunks_to_maker(&mut self) {
        for chunk_index in 0..self.chunks.len() {
            self.make_dict((self.chunks[chunk_index]).clone());
        }
        self.dict = self
            .dict
            .drain(..)
            .filter(|x| x.occurrence_num > 1)
            .collect();
    }
}
