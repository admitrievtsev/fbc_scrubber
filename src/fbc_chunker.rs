use std::collections::{HashMap, VecDeque};
use std::fs;
use std::hash::{DefaultHasher, Hasher};
use std::string::String;
use std::vec::Vec;
use crate::fbc_chunker::FBCChunk::{Sharped, Solid};
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

enum FBCChunk {
    Solid(Vec<u8>),
    Sharped(Vec<FBCHash>),
}

impl FBCChunk {
    /// calculate len of chunck
    fn len(&self, map: &HashMap<FBCHash, FBCChunk>) -> usize {
        match &self {
            Solid(sl) => { sl.len() },
            Sharped(sh) => { 
                sh.iter().fold(0, |acc, element| { 
                    acc + Self::len(map.get(element).expect("Chunk NPE"), map)
                })
            }
        }
    }
}

//Struct that provide frequency analysis for chunks
#[derive(Default)]
pub struct ChunkerFBC {
    chunk_ids: Vec<FBCHash>,
    chunks: HashMap<FBCHash, FBCChunk>,
}

impl ChunkerFBC {
    /// hash chunck and insert solid chunk in chuncks
    // maybe add_chunk ?
    fn insert_chunk(&mut self, chunk: Vec<u8>) -> FBCHash {
        let hash = hash_chunk(chunk.as_slice());
        self.chunks.insert(hash, Solid(chunk));
        hash
    }
    // maybe insert_cdc_chunk ?
    pub fn add_cdc_chunk(&mut self, first_stage_chunk: &Vec<u8>) {
        let res = self.insert_chunk(first_stage_chunk.clone());
        self.chunk_ids.push(res);
    }
    //This method is to print chunking results out
    fn to_str(word: Vec<u8>) -> String {
        String::from_utf8(word).expect("UTF-8 formatting failure")
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
    fn reconstruct_chunk_from_hash(&self, hash: &FBCHash) -> Vec<u8> {
        let mut main_chunk: Vec<u8> = vec![];
        match self.chunks.get(&hash).expect("Chunk NPE") {
            Solid(chunk) => { main_chunk.append(&mut chunk.clone()) }
            Sharped(chunks) => { main_chunk.append(&mut self.reconstruct_chunk(&chunks)) }
        }
        main_chunk
    }

    fn reconstruct_chunk(&self, hashes: &Vec<FBCHash>) -> Vec<u8> {
        let mut main_chunk: Vec<u8> = vec![];
        for hash in hashes {
            main_chunk.append(&mut Self::reconstruct_chunk_from_hash(self, hash));
        }
        main_chunk
    }
    //Method that write text dedup out || DEBUG ONLY
    pub fn reduplicate(&self, file_out: &str) -> usize {
        let mut string_out = String::new();
        for id in self.chunk_ids.iter() {
            string_out.push_str(&Self::to_str(Self::reconstruct_chunk_from_hash(self, id)));
        }
        //println!("PRINT TO FILE");
        println!("{}", string_out.len());
        fs::write(file_out, &string_out).expect("Unable to write the file");
        string_out.len()
    }
    //This method contains FBC chunker implementation
    pub fn fbc_dedup(&mut self, dict: &HashMap<u64, DictRecord>, chunck_partitioning: &Vec<(usize, usize)>) -> usize {
        let min_chunck_size = chunck_partitioning.iter()
            .map(|x| x.0)
            .max()
            .expect("panic on calculate min chunck size, fbs deduplicate");
        let mut k = 0;
        let mut chunk_deque: VecDeque<FBCHash> = VecDeque::new();
        for i in self.chunk_ids.iter() {
            chunk_deque.push_front(*i);
        }

        while !chunk_deque.is_empty() {
            if k % 100 == 0 {
                println!("Checked: {}", chunk_deque.len())
            }
            k += 1;
            
            // get hash
            let chunk_index = chunk_deque.pop_back().unwrap();
            
            // create reference to chunck to cut
            let unchecked_chunk = match &self.chunks.get(&chunk_index).expect("Chunk NPE") {
                Sharped(_) => { continue }
                Solid(chunk) => { chunk }
            };
            // move in chunck
            let mut chunk_char = 0;
            while (chunk_char as i128)
                < unchecked_chunk.len() as i128 - min_chunck_size as i128
            {
                let mut chunk_hash = 0;
                let mut dict_rec = None;

                for (_, size) in chunck_partitioning.iter() {
                    if (chunk_char as i128)
                        < unchecked_chunk.len() as i128 - *size as i128 {
                        continue;
                    }
                    chunk_hash = hash_chunk(&unchecked_chunk[chunk_char..chunk_char + size]);
                    if dict.contains_key(&chunk_hash) {
                        // dist record have hash
                        dict_rec = dict.get(&chunk_hash);
                    }
                }

                if let Some(dict_rec) = dict_rec {
                    if chunk_char == 0 {
                        // if big chunk start from is known
                        
                        let new_chunck = unchecked_chunk[dict_rec.get_len()..].to_owned();
                        let new_hash = self.insert_chunk(new_chunck);
                        
                        // add new chunck for analize
                        chunk_deque.push_front(new_hash);
                        
                        self.replace_all_two(
                            chunk_index, 
                            chunk_hash, 
                            new_hash);
                    } else if chunk_char + dict_rec.get_len() + 1 == unchecked_chunk.len() {
                        // if is known chunk in end of big chunk

                        let new_chunck = unchecked_chunk[..chunk_char].to_owned();
                        let new_hash = self.insert_chunk(new_chunck);
                        
                        // not add chunck for analize
                        
                        self.replace_all_two(
                            chunk_index,
                            new_hash,
                            chunk_hash,
                        );
                    }
                    else {
                        // if is known chunk in midle of big chunk
                        
                        // start
                        let new_chunck_1st = unchecked_chunk[..chunk_char].to_owned();
                        //end
                        let new_chunck_2st = unchecked_chunk[chunk_char + dict_rec.get_len()..].to_owned();

                        let new_hash_1st = self.insert_chunk(new_chunck_1st);
                        let new_hash_2nd = self.insert_chunk(new_chunck_2st);
                        
                        // add new chunck for analize
                        chunk_deque.push_front(new_hash_2nd);

                        self.replace_all_three(
                            chunk_index,
                            new_hash_1st,
                            chunk_hash,
                            new_hash_2nd,
                        );
                    }
                    
                    if !self.chunks.contains_key(&chunk_hash) {
                        let _ = self.insert_chunk(dict_rec.get_chunk());
                    }
                    break;
                } else {
                    chunk_char += 1;
                }
            }
        }
        println!(
            "{}",
            self.dict_size() + self.chunk_ids.len() * 8
        );
        
        

        self.dict_size() + self.chunk_ids.len() * 8
    }

    fn hash_chunk(tmp_vec: &Vec<u8>) -> u64 {
        let mut hasher = DefaultHasher::new();
        hasher.write(tmp_vec.as_slice());
        hasher.finish()
    }
    // Slicing chunk on 2 different
    fn replace_all_two(&mut self, to_change: FBCHash, first: FBCHash, second: FBCHash) {
        *self.chunks.get_mut(&to_change).unwrap() = FBCChunk::Sharped(vec![first, second]);
    }

    // Slicing chunk on 3 different
    fn replace_all_three(&mut self, to_change: FBCHash, first: FBCHash, second: FBCHash, third: FBCHash) {
        *self.chunks.get_mut(&to_change).unwrap() = FBCChunk::Sharped(vec![first, second, third]);
    }

    // Optimization Method
    // You can call this method to reduce analyser records with low frequency
    // It will force scrubber to run faster but also will reduce dedup gain
    fn dict_count_size(&self) -> usize {
        self.chunks.values().fold(0, |acc, x| acc + x.len(&self.chunks))
    }

    // compute size to storage chuncks
    fn dict_size(&self) -> usize {
        self.chunks.values().fold(0, |acc, x| {
            acc + match &x { 
                FBCChunk::Solid(chunck) => chunck.len(),
                FBCChunk::Sharped(chuncks) => chuncks.len() * size_of::<FBCHash>(),
            }
        })
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
