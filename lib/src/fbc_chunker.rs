use std::collections::HashMap;
use std::fs;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::string::String;
use std::vec::Vec;

use crate::frequency_analyser::DictRecord;
use chunkfs::Chunk;

use crate::frequency_analyser::FrequencyAnalyser;
//Max size of frequent chunk that can be found by analyser
const MAX_CHUNK_SIZE: usize = 64;
//DEBUG OLNY || Parameter of FSChunker
const AGING_CONST: u64 = 1024 * 1024;

const FIXED_CHUNKER_SIZE: usize = 128;

//Min size of frequent chunk that can be found by analyser
const MIN_CHUNK_SIZE: usize = 63;

//Macros that I use to increase value by 1
macro_rules! inc {
    ($x:expr) => {
        $x += 1
    };
}

//Struct that provide frequency analysis for chunks
#[derive(Default)]
pub struct ChunkerFBC {
    chunk_ids: Vec<usize>,
    chunks: Vec<Vec<u8>>,
}

impl ChunkerFBC {
    pub fn add_cdc_chunk(&mut self, first_stage_chunk: &Vec<u8>) {
        self.chunk_ids.push(self.chunks.len());
        self.chunks.push(first_stage_chunk.to_vec().clone());
    }
    //This method is to print chunking results out
    fn tostr(word: &Vec<u8>) -> String {
        String::from_utf8(word.to_vec()).expect("UTF-8 formatting failure")
    }

    //Updating analyser occurrences counter

    //The main method which makes text dedup || DEBUG ONLY
    /*
    pub fn deduplicate(&mut self, file_in: &str, file_out: &str) {
        self.simple_dedup(file_in);
        self.throw_chunks_to_maker();
        self.reduplicate(file_out);
    }

     */

    //Method that write text dedup out || DEBUG ONLY
    pub fn reduplicate(&self, file_out: &str) {
        let mut string_out = String::new();
        for id in self.chunk_ids.iter() {
            string_out.push_str(&Self::tostr(&self.chunks[*id]));
        }
        //println!("PRINT TO FILE");
        println!("{}", string_out.len());
        fs::write(file_out, string_out).expect("Unable to write the file");
    }
    //This method contains FBC chunker implementation
    pub fn fbc_dedup(&mut self, mut dict: &HashMap<u64, DictRecord>) -> usize {
        //self.process_dictionary();
        //self.reduce_low_occur();
        let mut k = 0;

        let dict = dict.clone();
        let mut chunk_index = 0;
        while chunk_index < self.chunks.len() {
            k += 1;

            if (k % 10 == 0) {
                println!("Checked: {}", (k as f32 / self.chunks.len() as f32) * 100.0)
            }
            let mut strr:String = "".to_string();

            let mut chunk_char  = 0;
            while !(chunk_char as i128 >= self.chunks[chunk_index].len() as i128 - MAX_CHUNK_SIZE as i128) {
                //println!("{}", self.dict_count_size());
                //println!("{} {} {}", chunk_index, self.chunks.len(), chunk_char);
                let mut k_state = false;
                //println!("{} {} {}",  chunk_char, self.chunks[chunk_index].len(), strr);

                for dict_rec in dict {
                    //println!("{} {} {} {} {} {}", dict_rec.1.get_chunk().len(), chunk_char, self.chunks[chunk_index].len(), dict_rec.0, chunk_index, strr);

                    if (chunk_char as i128 > self.chunks[chunk_index].len() as i128 - MAX_CHUNK_SIZE as i128){
                        k_state = true;
                        break;
                    }

                    if dict_rec.1.get_chunk().len() < self.chunks[chunk_index].len() {
                        let mut is_chunk_correct = true;
                        for char_index in 0..dict_rec.1.get_chunk().len() {
                            if dict_rec.1.get_chunk()[char_index]
                                != self.chunks[chunk_index][chunk_char + char_index]
                            {
                                is_chunk_correct = false;
                                break;
                            }
                        }

                        if is_chunk_correct {
                            let mut is_found = false;
                            let mut cut_out = self.chunks.len();
                            strr.push("d".parse().unwrap());


                            for chunk_index in 0..self.chunks.len() {
                                if self.chunks[chunk_index] == dict_rec.1.get_chunk() {
                                    is_found = true;
                                    cut_out = chunk_index;
                                    break;
                                }
                            }
                            if chunk_char == 0 {
                                if !is_found {
                                    self.chunks.push(dict_rec.1.get_chunk().clone());
                                }
                                self.chunks[chunk_index] = self.chunks[chunk_index]
                                    [dict_rec.1.get_chunk().len()..self.chunks[chunk_index].len()]
                                    .to_owned();
                                self.replace_all_two(chunk_index, cut_out, chunk_index);
                                chunk_char = 0;
                                break
                            } else {
                                if !is_found {
                                    self.chunks.push(dict_rec.1.get_chunk().clone());
                                }
                                self.chunks.push(
                                    self.chunks[chunk_index][dict_rec.1.get_size() + chunk_char
                                        ..self.chunks[chunk_index].len()]
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

                        }
                    }
                    else {break}
                }
                if k_state{
                    //println!("KSTATE");
                    break
                }
                chunk_char += 1;

            }
            chunk_index += 1;
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
    fn dict_count_size(&self) -> usize {
        return self.chunks.iter().fold(0, |acc, x| acc + x.len());
    }

    // FSDedup Chunker
    /*
    fn simple_dedup(&mut self, f_in: &str) {
        let contents = fs::read(f_in).expect("Should have been able to read the file");
        println!("{}", contents.len());
            self.make_dict(&contents);

    }

     */

    fn throw_chunks_to_maker(&mut self) {
        /*
        //self.print_dict();
        self.dict = self
            .dict
            .drain()
            .filter(|x| x.1.occurrence_num > 1)
            .collect();
        self.fbc_dedup();
        println!("{:?}", self.chunk_ids.len());
        println!("{:?}", self.chunks.len());
        */
    }
}
