use std::collections::HashMap;
use std::sync::{Arc, Mutex};

//Struct that provide occurrences counting during analysis of data
pub const MAX_CHUNK_SIZE: usize = 128;
const MIN_CHUNK_SIZE: usize = 127;
use std::hash::{DefaultHasher, Hasher};

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
    pub(crate) dict: HashMap<u64, DictRecord>,
}

impl FrequencyAnalyser {
    pub fn new() -> Self {
        FrequencyAnalyser {
            dict: HashMap::default()
        }
    }
    pub fn get_dict(&mut self) -> HashMap<u64, DictRecord> {
        self.dict.drain().collect()
    }


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
    pub fn print_dict(&self) {
        for i in self.dict.iter() {
            if i.1.occurrence_num > 1 {
                println!("chunk: {:?} occurrence: {}", i.1.chunk, i.1.occurrence_num)
            }
        }
    }
    pub fn reduce_low_occur(&mut self) {
        self.dict = self.dict
            .drain()
            .filter(|x| x.1.occurrence_num > 1)
            .collect();
    }
}
pub fn append_dict(dict_map: Arc<Mutex<HashMap<u64, DictRecord>>>, first_stage_chunk: &Vec<u8>) -> HashMap<u64, DictRecord> {
    let mut hmap: HashMap<u64, DictRecord> = HashMap::new();
    let mut temp_chunks: Vec<Vec<u8>> = vec![vec![]; MAX_CHUNK_SIZE - MIN_CHUNK_SIZE];
    for slice_index in MIN_CHUNK_SIZE..MAX_CHUNK_SIZE {
        for char in first_stage_chunk.iter().take(slice_index + 1) {
            temp_chunks[slice_index - MIN_CHUNK_SIZE].push(*char);
        }
    }
    let mut start_index = 1;
    while start_index < first_stage_chunk.len() - MAX_CHUNK_SIZE {
        for chunk_size in MIN_CHUNK_SIZE..MAX_CHUNK_SIZE {
            for char_index in 1..chunk_size + 1 {
                temp_chunks[chunk_size - MIN_CHUNK_SIZE][char_index - 1] =
                    temp_chunks[chunk_size - MIN_CHUNK_SIZE][char_index]
            }
            temp_chunks[chunk_size - MIN_CHUNK_SIZE][chunk_size] =
                first_stage_chunk[start_index + chunk_size];
            start_index += add_chunk(temp_chunks[chunk_size - MIN_CHUNK_SIZE].clone(), dict_map.clone());
        }
    }
    hmap
}

fn add_chunk(chunk: Vec<u8>, target_map: Arc<Mutex<HashMap<u64, DictRecord>>>) -> usize {
    let str_size = chunk.len();
    let mut hasher = DefaultHasher::new();
    hasher.write(chunk.as_slice());
    let chunk_hash = hasher.finish();
    let mut target_map = target_map.lock().unwrap();
    if target_map.contains_key(&chunk_hash) {
        match target_map.get_mut(&chunk_hash) {
            Some(x) => match x.occurrence_num {
                1 => {
                    target_map.get_mut(&chunk_hash).unwrap().occurrence_num += 1;
                }
                _ => {}
            },
            None => panic!("chunk hash does not exists"),
        }
        return MIN_CHUNK_SIZE;
    }
    target_map.insert(
        chunk_hash,
        DictRecord {
            chunk: chunk.to_vec(),
            occurrence_num: 1,
            size: str_size,
            hash: chunk_hash,
        },
    );
    16
}

pub fn count_deps(frequency: u32, ptr: &HashMap<u64, DictRecord>) -> (u32, u32) {
    let mut count_dups = 0;
    let mut total_count = 0;
    for i in ptr.iter() {
        if i.1.occurrence_num >= frequency {
            count_dups += 1;
        }
        total_count += i.1.occurrence_num;
    }
    (count_dups, total_count)
}