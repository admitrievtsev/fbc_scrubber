use std::collections::{self, HashMap, BTreeSet};
use std::sync::{Arc, Mutex, MutexGuard};
use dashmap::{DashMap, TryReserveError};
//Struct that provide occurrences counting during analysis of data
pub const MAX_CHUNK_SIZE: usize = 128;
const MIN_CHUNK_SIZE: usize = 127;
use std::hash::{DefaultHasher, Hasher};
use std::{default, mem, thread};
use std::io::{self, Read, Seek, Write};
use std::fs;
use std::path;
use crate::THREADS_COUNT;
use crate::FBCHash;


#[derive(Default)]
#[derive(PartialEq)]
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
    pub fn get_occurrence_num(&self) -> u32 {
        self.occurrence_num
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
        // println!("input:\n{:?}", first_stage_chunk[0..2 * 128].to_vec());

        let mut start_index = 1;
        while start_index <= first_stage_chunk.len() - MAX_CHUNK_SIZE {
            for chunk_size in MIN_CHUNK_SIZE..MAX_CHUNK_SIZE {
                // for char_index in 1..chunk_size + 1 {
                //     temp_chunks[chunk_size - MIN_CHUNK_SIZE][char_index - 1] =
                //         temp_chunks[chunk_size - MIN_CHUNK_SIZE][char_index]
                // }
                // temp_chunks[chunk_size - MIN_CHUNK_SIZE][chunk_size] =
                //     first_stage_chunk[start_index + chunk_size];
                for char_index in 0..=chunk_size {
                    temp_chunks[chunk_size - MIN_CHUNK_SIZE][char_index] = first_stage_chunk[start_index + char_index];
                }

                // println!("chunsk:\n{:?}", temp_chunks[chunk_size - MIN_CHUNK_SIZE]);
                start_index += FrequencyAnalyser::add_chunk(temp_chunks[chunk_size - MIN_CHUNK_SIZE].clone(), self.dict.clone());
                
                // let mut s = String::default();
                // std::io::stdin().read_line(&mut s);
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
                        x.occurrence_num += 1;
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


impl FrequencyAnalyser {
    /* retunt count unique saved recored */
    pub fn save_to_file(&self, path: &path::Path) -> Result<usize, io::Error> {
        /* create file */
        let mut file = fs::File::create(path)?;

        /* write count of records */
        file.write(usize::to_be_bytes(self.dict.len()).as_slice())?;
        /* write all records */
        for it in self.dict.iter() {
            it.save_to_file(&mut file)?;
        }

        Ok((self.dict.len()))
    }

    pub fn load_from_file(path: &path::Path) -> Result<Self, io::Error> {
        /* create file, map */
        let mut file = fs::File::open(path)?;
        let mut analyser = FrequencyAnalyser::new();
        
        let count_records = Self::load_count_records(&mut file)?;
        /* resize map */
        let err = Arc::get_mut(&mut analyser.dict)
            .unwrap()
            .try_reserve(count_records);
        
        /* return if error */
        if let Err(_) = err {
            return Err(io::Error::new(io::ErrorKind::Other, "error reserve memory!"));
        }
        
        /* read records */
        for _i in 0..count_records {
            let recored = DictRecord::load_from_file(&mut file)?;
            analyser.dict.insert(recored.hash, recored);
        }
        
        Ok(analyser)
    }

    /* read count of records in file */
    fn load_count_records(file: &mut fs::File) -> Result<usize, io::Error> {
        const SIZE: usize = size_of::<usize>();
        let mut buffer =  [0; SIZE];
        /* read buffer */
        file.read(&mut buffer)?;
        /* read count of records from buffer */
        Ok(usize::from_be_bytes(buffer))
    }

    fn load_hashes(file: &mut fs::File) -> Result<BTreeSet<FBCHash>, io::Error> {
        let mut result = collections::BTreeSet::<FBCHash>::new();

        let count_records = Self::load_count_records(file)?;

        /* read records */
        for _i in 0..count_records {
            let recored = DictRecord::load_with_out_chunk(file)?;
            result.insert(recored.hash);
            file.seek_relative(recored.size as i64)?;
        }

        Ok(result)
    }

    /* return count unique saved records */
    pub fn update(path: &path::Path, new_records: &[DictRecord]) ->  Result<usize, io::Error> {
        let mut file = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(path)?;
        let existed_hashes = Self::load_hashes(&mut file)?;
        let mut unique_records_indexes = Vec::new();

        /* count unique records */
        for i in 0..new_records.len() {
            if !existed_hashes.contains(&new_records[i].hash) {
                unique_records_indexes.push(i);
            }
        }
        /* delta_len add to existing len and write to file */
        let new_len = unique_records_indexes.len() + existed_hashes.len();

        /* set write to start of file */
        file.seek(io::SeekFrom::Start(0))?;
        /* write new len */
        file.write(new_len.to_be_bytes().as_slice())?;
        /* set write to end of file */
        file.seek(io::SeekFrom::End(0))?;
        /* write all unique records */
        for i in unique_records_indexes.iter() {
            new_records.get(*i).unwrap().save_to_file(&mut file)?;
        }
        
        Ok(unique_records_indexes.len())
    }
}

impl DictRecord {
    pub fn print(&self) {
        println!("{} {} {}\n{:?}", self.hash, self.occurrence_num, self.size, self.chunk);
    }

    pub fn save_to_file(&self, file: &mut fs::File) -> Result<(), io::Error> {
        file.write(u64::to_be_bytes(self.hash).as_slice())?;
        file.write(u32::to_be_bytes(self.occurrence_num).as_slice())?;
        file.write(usize::to_be_bytes(self.size).as_slice())?;
        file.write(self.chunk.as_slice())?;

        Ok(())
    }

    fn load_with_out_chunk(file: &mut fs::File) -> Result<Self, io::Error> {
        let mut result = DictRecord::default();

        /* indexes of data in buffer */
        const HASH: usize = size_of::<u64>();
        const OCC_NUM: usize = size_of::<u32>() + HASH;
        const SIZE: usize = size_of::<usize>() + OCC_NUM;
        /* create buffer */
        let mut buffer =  [0; SIZE];
        /* read buffer */
        file.read(&mut buffer)?;
        /* read data from buffer by indexes */
        result.hash = u64::from_be_bytes(buffer[0..HASH].try_into().unwrap());
        result.occurrence_num = u32::from_be_bytes(buffer[HASH..OCC_NUM].try_into().unwrap());
        result.size = usize::from_be_bytes(buffer[OCC_NUM..SIZE].try_into().unwrap());

        Ok(result)
    }
    
    pub fn load_from_file(file: &mut fs::File) -> Result<Self, io::Error> {
        let mut result = Self::load_with_out_chunk(file)?;

        /* resize chunk */
        result.chunk.resize(result.size, 0);

        /* read chunk */
        file.read(&mut result.chunk)?;

        Ok(result)
    }
}

#[test]
fn fbc_save_load_record_test() {
    let save_record = DictRecord { 
        chunk: vec![0; 12], 
        occurrence_num: 3, 
        size: 12,
        hash: 1234 
    };
    let file = path::Path::new("./record_save.txt");
    save_record.save_to_file(
            &mut fs::File::create(file).expect("file create fail")
        ).expect("save record fail");

    let load_record = DictRecord::load_from_file(
            &mut fs::File::open(file).expect("file open fail")
        ).expect("load record fail");

    let eq = save_record == load_record;
    save_record.print();
    load_record.print();

    assert_eq!(eq, true, "load and save records not equal!");
}

#[test]
fn fbc_save_load_analizer_test() {
    let save_analyser = FrequencyAnalyser::new();
    let contents = fs::read("test_files_input/lowinput.txt")
        .expect("Should have been able to read the file");
    save_analyser.append_dict(&contents);

    println!("save analyser: {}", save_analyser.dict.len());
    for it in save_analyser.dict.iter().take(5) {
        println!(
            "{} {} {}",
            it.occurrence_num, it.size, it.hash
        );
    }
    println!("");
    
    let file = path::Path::new("./save_load_analizer.txt");
    save_analyser.save_to_file(file).expect("fail to save!");

    let load_analyser = FrequencyAnalyser::load_from_file(file).expect("fail to load!");

    println!("load analyser: {}", load_analyser.dict.len());
    for it in load_analyser.dict.iter().take(5) {
        println!(
            "{} {} {}",
            it.occurrence_num, it.size, it.hash
        );
    }
    println!("");

    let eq = save_analyser.dict == load_analyser.dict;

    assert!(eq, "source and loaded analizer not equal!");
}

#[test]
fn fbc_load_hashes_analizer_test() {
    let save_analyser = FrequencyAnalyser::new();
    let contents = fs::read("test_files_input/lowinput.txt")
        .expect("Should have been able to read the file");
    save_analyser.append_dict(&contents);

    let file = path::Path::new("./load_hashes_analizer.txt");
    save_analyser.save_to_file(file).expect("fail to save!");

    let load_hashes = FrequencyAnalyser::load_hashes(
            &mut fs::File::open(file).expect("file open fail")
        ).expect("get hashes fail");

    for it in load_hashes.iter() {
        if save_analyser.dict.contains_key(it) == false {
            panic!("load key not exist in source");
        }
    }

    for it in save_analyser.dict.iter() {
        if load_hashes.contains(&it.hash) == false {
            panic!("source key not load");
        }
    }
}

#[test]
fn fbc_update_analizer_test() {
    let path = path::Path::new("./update_analizer.txt");
    let contents = fs::read("test_files_input/lowinput.txt")
        .expect("Should have been able to read the file");
    println!("content size: {}", contents.len());

    let save_analyser = FrequencyAnalyser::new();
    save_analyser.append_dict(&contents);
    let len = save_analyser.save_to_file(path).expect("fail to save!");

    let mut k = true;
    let new_content = contents.into_iter()
    .filter(|a| {
        k = !k;
        k
    })
    .collect::<Vec<u8>>();
    println!("new_content size: {}", new_content.len());

    let mut other_analizer = FrequencyAnalyser::new();
    other_analizer.append_dict(&new_content);
    let d_len = FrequencyAnalyser::update(path, &other_analizer
        .get_dict()
        .into_iter()
        .map(|(a, b)| b)
        .collect::<Vec<DictRecord>>()
    ).expect("upgrade fail");

    let analizer = FrequencyAnalyser::load_from_file(path)
        .expect("error to load updated file");
    let eq = len + d_len == analizer.dict.len();
    assert!(eq, "the number of records does not converge")
}




