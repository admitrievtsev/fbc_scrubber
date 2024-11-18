use std::fs;
use std::string::String;
use std::vec::Vec;
use std::hash::{DefaultHasher, Hash, Hasher};

use chunkfs::Chunk;

//Max size of frequent chunk that can be found by analyser
const MAX_CHUNK_SIZE: usize = 8;
//DEBUG OLNY || Parameter of FSChunker
const FIXED_CHUNKER_SIZE: usize = 128;

//Min size of frequent chunk that can be found by analyser
const MIN_CHUNK_SIZE: usize = 7;

//Macros that I use to increase value by 1
macro_rules! inc {
    ($x:expr) => {
        $x += 1
    };
}

//Struct that provide occurrences counting during analysis of data
#[derive(Default)]
struct DictRecord {
    chunk: Vec<u8>,
    occurrence_num: u32,
    size: usize,
    hash: u64
}

//Struct that provide frequency analysis for chunks
#[derive(Default)]
pub struct Analyser {
    dict: Vec<DictRecord>,
    chunk_ids: Vec<usize>,
    chunks: Vec<Vec<u8>>,
}

impl Analyser {
    pub fn make_dict(&mut self, first_stage_chunk: &Vec<u8>) {
        let mut temp_chunks: Vec<Vec<u8>> = vec![vec![]; MAX_CHUNK_SIZE - MIN_CHUNK_SIZE];
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
        self.chunk_ids.push(self.chunks.len());
        self.chunks.push(first_stage_chunk.to_vec().clone());

    }
    pub fn print_dict(&self) {
        for i in self.dict.iter() {
            if(i.occurrence_num > 1) {
                println!("chunk: {:?} occurence: {}", i.chunk, i.occurrence_num)
            }
        }
    }
    //This method is to print chunking results out
    fn tostr(word: &Vec<u8>) -> String {
        String::from_utf8(word.to_vec()).expect("UTF-8 formatting failure")
    }

    //Updating analyser occurrences counter
    fn add_chunk(&mut self, chunk: Vec<u8>) {
        let str_size = chunk.len();
        let mut chunk_dict_id = 0;
        let mut hasher = DefaultHasher::new();
        hasher.write(chunk.as_slice());
        let chunk_hash = hasher.finish();

        for dict_chunk in self.dict.iter() {
            if dict_chunk.hash == chunk_hash {
                inc!(self.dict[chunk_dict_id].occurrence_num);
                return;
            }
            inc!(chunk_dict_id);
        }
        self.dict.push(DictRecord {
            chunk: chunk.to_vec(),
            occurrence_num: 1,
            size: str_size,
            hash: chunk_hash
        });
    }

    //The main method which makes text dedup || DEBUG ONLY
    pub fn deduplicate(&mut self, file_in: &str, file_out: &str) {
        self.simple_dedup(file_in);
        self.throw_chunks_to_maker();
        self.fbc_dedup();
        self.reduplicate(file_out);
    }

    //Method that write text dedup out || DEBUG ONLY
    fn reduplicate(&self, file_out: &str) {
        let mut string_out = String::new();
        for id in self.chunk_ids.iter() {
            string_out.push_str(&Self::tostr(&self.chunks[*id]));
        }
        //println!("PRINT TO FILE");
        println!("{}", string_out.len());
        fs::write(file_out, string_out).expect("Unable to write the file");
    }

    //This method contains FBC chunker implementation
    pub fn fbc_dedup(&mut self) -> usize {
        for dict_index in 0..self.dict.len() {
            if (dict_index % 1000 == 0){
                println!("Checked: {}", (dict_index as f32 / self.dict.len() as f32) * 100.0)
            }
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
        return self.dict_count_size() + self.chunk_ids.len() * 8;
    }

    // Slicing chunk on 2 different
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

    // Slicing chunk on 3 different
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
        self.chunk_ids = temp_vec;
        //println!("{:?}",self.chunk_ids);
    }

    // Optimization Method
    // You can call this method to reduce analyser records with low frequency
    // It will force scrubber to run faster but also will reduce dedup gain
    pub fn reduce_low_occur(&mut self) {
        self.dict = self
            .dict
            .drain(..)
            .filter(|x| x.occurrence_num > 1)
            .collect();
    }
    fn dict_count_size(&self) -> usize {
        return self.chunks.iter().fold(0, |acc, x| acc + x.len());
    }

    // FSDedup Chunker
    fn simple_dedup(&mut self, f_in: &str) {
        let contents = fs::read(f_in).expect("Should have been able to read the file");
        println!("{}", contents.len());
            self.make_dict(&contents);

    }

    fn throw_chunks_to_maker(&mut self) {

        //self.print_dict();
        self.dict = self
            .dict
            .drain(..)
            .filter(|x| x.occurrence_num > 1)
            .collect();
        self.fbc_dedup();
        println!("{:?}", self.chunk_ids.len());
        println!("{:?}", self.chunks.len());

    }
}
