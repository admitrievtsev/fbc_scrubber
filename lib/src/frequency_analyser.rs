use std::collections::HashMap;
use qfilter::Filter;

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

pub struct FrequencyAnalyser {
    dict: HashMap<u64, DictRecord>,
    filter: qfilter::Filter
}

impl FrequencyAnalyser {
    pub fn new() -> Self{FrequencyAnalyser{ dict: HashMap::default(), filter: qfilter::Filter::new(10000000, 0.01).unwrap() }}
    pub fn get_dict(&self) -> &HashMap<u64, DictRecord> {
        &self.dict
    }
    pub fn make_dict(&mut self, first_stage_chunk: &Vec<u8>) {
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
                start_index += self.add_chunk(temp_chunks[chunk_size - MIN_CHUNK_SIZE].clone());
            }
        }
        //self.process_dictionary();
        /*
        for x in 0..self.dict.len(){
            if self.dict[x].occurrence_num > 1 && self.dict[x].age > AGING_CONST * self.dict[x].occurrence_num as u64{
                self.dict[x].occurrence_num = 1;

            }
            else{
                self.dict[x].age += first_stage_chunk.len() as u64;
            }
        }

         */
        //println!("{}", self.dict.len())
    }

    fn add_chunk(&mut self, chunk: Vec<u8>) -> usize {
        let str_size = chunk.len();
        let mut hasher = DefaultHasher::new();
        hasher.write(chunk.as_slice());
        let chunk_hash = hasher.finish();

        if self.filter.contains(chunk.clone()) {
            match self.dict.get_mut(&chunk_hash) {
                Some(_) => {self.dict.get_mut(&chunk_hash).unwrap().occurrence_num += 1;}
                None => {}
            }
            /*
            if(self.dict.get_mut(&chunk_hash).unwrap().occurrence_num > 2){
            println!("{}",self.dict.get_mut(&chunk_hash).unwrap().occurrence_num);}
             */
            return MIN_CHUNK_SIZE;
        }

        self.dict.insert(
            chunk_hash,
            DictRecord {
                chunk: chunk.to_vec(),
                occurrence_num: 1,
                size: str_size,
                hash: chunk_hash,
            },
        );
        self.filter.insert(chunk).expect("Error with Bloom filter");
        1
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
                println!(
                    "{} {:?}",
                    record.1.occurrence_num,
                    record.1.chunk.as_ascii().unwrap()
                );
                sum_of_records += 1;
            }
        }
        println!(
            "FOR FREQUENCY >= 20 NUMBER OF RECORDS IS {}", sum_of_records
        )
    }
    pub fn print_dict(&self) {
        for i in self.dict.iter() {
            if i.1.occurrence_num > 1 {
                println!("chunk: {:?} occurence: {}", i.1.chunk, i.1.occurrence_num)
            }
        }
    }
    pub fn reduce_low_occur(&mut self) {
        for i in self.dict.iter_mut() {
            if (i.1.occurrence_num <= 1){
                self.filter.remove(i.1.chunk.clone());
            }
        }
        self.dict = self
            .dict
            .drain()
            .filter(|x| x.1.occurrence_num > 1)
            .collect();

    }
}
