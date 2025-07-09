use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard};
use dashmap::DashMap;
//Struct that provide occurrences counting during analysis of data
pub const MAX_CHUNK_SIZE: usize = 128;
const MIN_CHUNK_SIZE: usize = 127;
use std::hash::{DefaultHasher, Hasher};
use std::thread;
use crate::THREADS_COUNT;

#[derive(Default)]
pub struct DictRecord {
    chunk: Vec<u8>,
    occurrence_num: u32,
    size: usize,
    hash: u64,
}
impl DictRecord {
    pub fn get_chunk(&self) -> Vec<u8> {
        self.chunk.clone()
    }
    pub fn get_size(&self) -> usize {
        self.size
    }
}
impl Clone for DictRecord {
    fn clone(&self) -> Self {
        DictRecord { chunk: self.chunk.clone(), occurrence_num: self.occurrence_num, size: self.size, hash: self.hash }
    }
}

pub struct FrequencyAnalyser {
    pub(crate) dict: Arc<DashMap<u64, DictRecord>>,
}

impl FrequencyAnalyser {
    pub fn new() -> Self {
        FrequencyAnalyser {
            dict: Arc::new(DashMap::new()),
        }
    }

    pub fn get_dict(&mut self) -> HashMap<u64, DictRecord> {
        let mut tmp_hmap = HashMap::new();
        for i in self.dict.clone().iter() {
            tmp_hmap.insert(i.hash, i.clone());
        }
        tmp_hmap
    }

    /*
    pub fn process_dictionary(&self) {
        let mut sum_of_records = 0;

        for cur_freq in 1..20 {
            sum_of_records = 0;
            for record in self.dict.iter() {
                if record.1.occurrence_num == cur_freq {
                    sum_of_records += 1;
                }
            }
            println!(
                "FOR FREQUENCY {} NUMBER OF RECORDS IS {}",
                cur_freq, sum_of_records
            )
        }
        sum_of_records = 0;
        for record in self.dict.iter() {
            if record.1.occurrence_num >= 20 {
                println!("{}", record.1.occurrence_num);
                sum_of_records += 1;
            }
        }
        println!(
            "FOR FREQUENCY >= 20 NUMBER OF RECORDS IS {}",
            sum_of_records
        )
    }

     */
    pub fn print_dict(&self) {
        for i in self.dict.iter() {
            if i.occurrence_num > 1 {
                println!("chunk: {:?} occurrence: {}", i.chunk, i.occurrence_num)
            }
        }
    }
    pub fn reduce_low_occur(&mut self, occurrence: u32) {
        self.dict.clone().retain(|_, v| v.occurrence_num >= occurrence);
        self.dict.shrink_to_fit();
        println!("REDUCED")
    }

    pub fn count_candidates(&mut self, occurrence: u32) -> u32 {
        let mut dups = 0;
        for i in self.dict.iter() {
            if i.occurrence_num >= occurrence {
                dups += 1;
            }
        };
        dups
    }

    pub fn analyse_pack(&self, first_stage_chunks: [Option<&Vec<u8>>; THREADS_COUNT]) {
        thread::scope(|s| {
            for i in first_stage_chunks.iter() {
                match *i {
                    Some(chunk) => {
                        s.spawn(move ||
                            {
                                FrequencyAnalyser::append_dict(&self, chunk);
                            });
                    }
                    _ => {}
                }
            }
        });
    }

    pub fn append_dict(&self, first_stage_chunk: &Vec<u8>) {
        let mut temp_chunks: Vec<Vec<u8>> = vec![vec![]; MAX_CHUNK_SIZE - MIN_CHUNK_SIZE];
        for slice_index in MIN_CHUNK_SIZE..MAX_CHUNK_SIZE {
            for char in first_stage_chunk.iter().take(slice_index + 1) {
                temp_chunks[slice_index - MIN_CHUNK_SIZE].push(*char);
            }
        }
        let mut start_index = 1;
        while start_index <= first_stage_chunk.len() - MAX_CHUNK_SIZE {
            for chunk_size in MIN_CHUNK_SIZE..MAX_CHUNK_SIZE {
                for char_index in 1..chunk_size + 1 {
                    temp_chunks[chunk_size - MIN_CHUNK_SIZE][char_index - 1] =
                        temp_chunks[chunk_size - MIN_CHUNK_SIZE][char_index]
                }
                temp_chunks[chunk_size - MIN_CHUNK_SIZE][chunk_size] =
                    first_stage_chunk[start_index + chunk_size];
                start_index += FrequencyAnalyser::add_chunk(temp_chunks[chunk_size - MIN_CHUNK_SIZE].clone(), self.dict.clone());
            }
        }
    }

    pub fn add_chunk(chunk: Vec<u8>, target_map: Arc<DashMap<u64, DictRecord>>) -> usize {
        //println!("Add started");
        let str_size = chunk.len();
        let mut hasher = DefaultHasher::new();
        hasher.write(chunk.as_slice());
        let chunk_hash = hasher.finish();
        //println!("Ready to check");
        if target_map.contains_key(&chunk_hash) {
            //println!("Key contains");
            match target_map.get_mut(&chunk_hash) {
                Some(mut x) => return match x.occurrence_num {
                    1 => {
                        //println!("FFOUNd");
                        x.occurrence_num += 1;
                        MIN_CHUNK_SIZE
                    }
                    _ => {
                        //println!("ALREADY WRITTEN");
                        MIN_CHUNK_SIZE
                    }
                },
                None => panic!("chunk hash does not exists"),
            }

            println!("Return from append");
        }
        //println!("Add finished, insert then");

        //println!("CHUNK {:?}", chunk);

        target_map.insert(
            chunk_hash,
            DictRecord {
                chunk: chunk.to_vec(),
                occurrence_num: 1,
                size: str_size,
                hash: chunk_hash,
            },
        );

        //println!("Func finished");
        16
    }
}