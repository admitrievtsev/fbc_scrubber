use std::collections::{HashMap, VecDeque};
use std::fs;
use std::hash::{DefaultHasher, Hasher};
use std::string::String;
use std::vec::Vec;

use crate::frequency_analyser::DictRecord;

use crate::hash_chunk;

//Max size of frequent chunk that can be found by analyser

//DEBUG OLNY || Parameter of FSChunker
const AGING_CONST: u64 = 1024 * 1024;
pub type FBCHash = u64;

const FIXED_CHUNKER_SIZE: usize = 64;

//Min size of frequent chunk that can be found by analyser
pub const MAX_CHUNK_SIZE: usize = 128;
const MIN_CHUNK_SIZE: usize = 7;
//Macros that I use to increase value by 1
macro_rules! inc {
    ($x:expr) => {
        $x += 1
    };
}

//Struct that provide frequency analysis for chunks
#[derive(Default)]
pub struct ChunkerFBC {
    chunk_ids: Vec<FBCHash>,
    chunks: HashMap<FBCHash, Vec<u8>>,
}

impl ChunkerFBC {
    fn insert_chunk(&mut self, chunk: Vec<u8>) -> FBCHash {
        self.chunks.insert(hash_chunk(&chunk), chunk.clone());
        hash_chunk(&chunk)
    }
    pub fn add_cdc_chunk(&mut self, first_stage_chunk: &Vec<u8>) {
        let res = self.insert_chunk(first_stage_chunk.clone());
        self.chunk_ids.push(res);
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
    pub fn reduplicate(&self, file_out: &str) -> usize {
        let mut string_out = String::new();
        for id in self.chunk_ids.iter() {
            string_out.push_str(&Self::tostr(&self.chunks[id]));
        }
        //println!("PRINT TO FILE");
        println!("{}", string_out.len());
        fs::write(file_out, &string_out).expect("Unable to write the file");
        string_out.len()
    }
    //This method contains FBC chunker implementation
    pub fn fbc_dedup(&mut self, dict: &HashMap<u64, DictRecord>) -> usize {
        //self.process_dictionary();
        //self.reduce_low_occur();
        let mut k = 0;
        let mut chunk_deque: VecDeque<FBCHash> = VecDeque::new();
        for i in self.chunks.iter() {
            chunk_deque.push_front(*i.0)
        }

        //println!("{:?}", self.chunk_ids);

        while !chunk_deque.is_empty() {
            //println!("{:?}", self.chunks.keys());
            k += 1;
            let chunk_index = chunk_deque.pop_back().unwrap();
            if k % 100 == 0 {
                println!("Checked: {}", (chunk_deque.len()))
            }
            let mut chunk_char = 0;
            if !self.chunks.contains_key(&chunk_index) {
                continue;
            }
            while (chunk_char as i128)
                < self.chunks[&chunk_index].len() as i128 - MAX_CHUNK_SIZE as i128
            {
                //println!("{}", self.dict_count_size());
                //println!("{} {} {}", chunk_index, self.chunks.len(), chunk_char);
                let mut tmp_vec: Vec<u8> = Vec::with_capacity(MAX_CHUNK_SIZE);

                for i in 0..MAX_CHUNK_SIZE {
                    tmp_vec.push(self.chunks[&chunk_index][chunk_char + i]);
                }

                let mut k_state = false;
                let chunk_hash = hash_chunk(&tmp_vec);
                let mut split_two = false;
                if dict.contains_key(&chunk_hash) {
                    let dict_rec = (chunk_hash, dict.get(&chunk_hash).unwrap());
                    if chunk_char as i128
                        > self.chunks[&chunk_index].len() as i128 - MAX_CHUNK_SIZE as i128
                    {
                        k_state = true;
                        println!("some thing strange!!!(k_state if)");
                        break;
                    }
                    //println!("{} {} {} {} {}", dict_rec.1.get_chunk().len(), chunk_char, self.chunks[&chunk_index].len(), dict_rec.0, chunk_index);

                    if dict_rec.1.get_chunk().len() < self.chunks[&chunk_index].len() {
                        let is_chunk_correct = true;
                        /*
                        for char_index in 0..dict_rec.1.get_chunk().len() {
                            if dict_rec.1.get_chunk()[char_index]
                                != self.chunks[chunk_index][chunk_char + char_index]
                            {
                                is_chunk_correct = false;
                                break;
                            }
                        }
                        */

                        if is_chunk_correct {
                            let mut cut_out = 0;
                            if self.chunks.contains_key(&chunk_hash) {
                                cut_out = chunk_hash;
                            } else {
                                cut_out = self.insert_chunk(dict_rec.1.get_chunk().clone());
                            }
                            if chunk_char == 0 {
                                /*
                                 * if chunk start from is known
                                 */
                                let new_hash = self.insert_chunk(
                                    self.chunks[&chunk_index][dict_rec.1.get_chunk().len()
                                        ..self.chunks[&chunk_index].len()]
                                        .to_owned(),
                                );
                                chunk_deque.push_front(new_hash);
                                //self.chunks.remove(&chunk_index);
                                if chunk_index != cut_out && chunk_index != new_hash {
                                    self.chunks.remove(&chunk_index);
                                }
                                self.replace_all_two(chunk_index, cut_out, new_hash);
                                chunk_char = 0;
                                split_two = true;
                                //break
                            } else {
                                /*
                                 * if is known chunk in midle of chunk
                                 */
                                let new_hash_2nd = self.insert_chunk(
                                    self.chunks[&chunk_index][dict_rec.1.get_size() + chunk_char
                                        ..self.chunks[&chunk_index].len()]
                                        .to_owned(),
                                );
                                let new_hash_1st = self.insert_chunk(
                                    self.chunks[&chunk_index][0..chunk_char].to_owned(),
                                );
                                if chunk_index != cut_out
                                    && chunk_index != new_hash_2nd
                                    && chunk_index != new_hash_1st
                                {
                                    self.chunks.remove(&chunk_index);
                                }
                                chunk_deque.push_front(new_hash_2nd);
                                self.replace_all_three(
                                    chunk_index,
                                    new_hash_1st,
                                    cut_out,
                                    new_hash_2nd,
                                );
                            }
                            break;
                        }
                    } else {
                        break;
                    }
                }
                if k_state {
                    //println!("KSTATE");
                    break;
                }
                if !split_two {
                    chunk_char += 1;
                }
            }
        }
        println!(
            "{}",
            self.dict_count_size() + self.chunk_ids.len() * 8 + self.chunks.len() * 8
        );
        self.dict_count_size() + self.chunk_ids.len() * 8 + self.chunks.len() * 8
    }
    fn hash_chunk(tmp_vec: &Vec<u8>) -> u64 {
        let mut hasher = DefaultHasher::new();
        hasher.write(tmp_vec.as_slice());
        hasher.finish()
    }
    // Slicing chunk on 2 different
    fn replace_all_two(&mut self, to_change: u64, first: u64, second: u64) {
        let mut temp_vec: Vec<u64> = Vec::with_capacity(self.chunks.len() + 1);
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
    fn replace_all_three(&mut self, to_change: u64, first: u64, second: u64, third: u64) {
        let mut temp_vec: Vec<u64> = Vec::with_capacity(self.chunks.len() + 2);
        //println!("{} {} {}", first, second, third);

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
        self.chunks.values().fold(0, |acc, x| acc + x.len())
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
