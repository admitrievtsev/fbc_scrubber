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
use crate::{hash_chunk, THREADS_COUNT};
use crate::FBCHash;


#[derive(Default)]
#[derive(PartialEq)]
pub struct DictRecord {
    chunk: Vec<u8>,
    occurrence_num: u32,
    size: usize,
    hash: FBCHash,
}

impl DictRecord {
    pub fn get_chunk(&self) -> Vec<u8> {
        self.chunk.clone()
    }
    pub fn get_chunk_ref(&self) -> &Vec<u8> {
        &self.chunk
    }
    pub fn get_size(&self) -> usize {
        self.size
    }
    pub fn get_len(&self) -> usize {
        Self::get_size(&self)
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
    pub(crate) dict: Arc<DashMap<FBCHash, DictRecord>>,
    /// size, offset
    chunck_partitioning: Vec<(usize, usize)>,
}

impl FrequencyAnalyser {
    pub fn new() -> Self {
        FrequencyAnalyser {
            dict: Arc::new(DashMap::new()),
            chunck_partitioning: vec![(64, 16)],
        }
    }

    /// chunck_partitioning - (size, offset)
    pub fn new_with_distribution(chunck_partitioning: Vec<(usize, usize)>) -> Self {
        FrequencyAnalyser {
            dict: Arc::new(DashMap::new()),
            chunck_partitioning,
        }
    }

    pub fn new_with_sizes(chunck_sizes: Vec<usize>) -> Self {
        FrequencyAnalyser {
            dict: Arc::new(DashMap::new()),
            chunck_partitioning: chunck_sizes.into_iter().map(|size| {(size, size / 4)}).collect(),
        }
    }

    pub fn get_dict(&mut self) -> HashMap<FBCHash, DictRecord> {
        let mut tmp_hmap = HashMap::new();
        for i in self.dict.clone().iter() {
            tmp_hmap.insert(i.hash, i.clone());
        }
        tmp_hmap
    }

    pub fn get_chunck_partitioning(&self) -> &Vec<(usize, usize)> {
        &self.chunck_partitioning
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

    pub fn append_dict(&self, first_stage_chunk: &[u8]) {
        let mut index = 0;
        let mut save_index = 0;

        for (size, offset, ) in self.chunck_partitioning.iter() {
            // while index < first_stage_chunk.len() - size + 1 {
            while index + size < first_stage_chunk.len() + 1 {
                let res =  FrequencyAnalyser::add_chunk(
                    &first_stage_chunk[index..index + size], 
                    self.dict.clone()
                );
                if res {
                    save_index = index;
                }
                if index - save_index < *size {
                    index += offset;
                } else {
                    index += size;
                }
            }
        }
    }

    /// return true if chunck was inserted to target map 
    pub fn add_chunk(chunk: &[u8], target_map: Arc<DashMap<FBCHash, DictRecord>>) -> bool {
        //println!("Add started");
        let size = chunk.len();
        let chunk_hash = hash_chunk(chunk);
        //println!("Ready to check");
        match target_map.get_mut(&chunk_hash) {
            Some(mut x) => {
                x.occurrence_num += 1;
                false // size
            },
            None => {
                target_map.insert(
                    chunk_hash,
                    DictRecord {
                        chunk: chunk.to_vec(),
                        occurrence_num: 1,
                        size: size,
                        hash: chunk_hash,
                    },
                );
                true // offset
            }
        }
    }
}

#[test]
fn fbc_add_chunck_analizer_test() {
    let target_map =  Arc::new(DashMap::<FBCHash, DictRecord>::new());
    let chunk1: &[u8] = &[1, 2, 3];
    let chunk2: &[u8] = &[3, 4, 5];
    let chunk3: &[u8] = &[5, 6, 7];

    assert!(FrequencyAnalyser::add_chunk(chunk1, target_map.clone()), "Add not exists chunk 1");
    assert!(FrequencyAnalyser::add_chunk(chunk2, target_map.clone()), "Add not exists chunk 2");
    assert!(FrequencyAnalyser::add_chunk(chunk3, target_map.clone()), "Add not exists chunk 3");
    
    assert!(!FrequencyAnalyser::add_chunk(chunk1, target_map.clone()), "Add exists chunk 1");
    assert!(!FrequencyAnalyser::add_chunk(chunk2, target_map.clone()), "Add exists chunk 2");
    assert!(!FrequencyAnalyser::add_chunk(chunk3, target_map.clone()), "Add exists chunk 3");
}

#[test]
fn fbc_append_dict_analizer_test() {
    let mut analizer = FrequencyAnalyser::new_with_distribution(vec![(8, 1)]);
    
    let content_1: &[u8] = &[1, 2, 3, 4];
    analizer.append_dict(content_1);

    let dict = analizer.get_dict();
    assert_eq!(dict.len(), 0, "Dumb check");


    let mut analizer = FrequencyAnalyser::new_with_distribution(vec![(4, 1)]);
    
    let content_1: &[u8] = &[1, 2, 3, 4];
    analizer.append_dict(content_1);

    let dict = analizer.get_dict();
    assert_eq!(dict.len(), 1, "Dict not append");
    let get_content_1 = &dict[&hash_chunk(content_1)];
    assert_eq!(get_content_1.chunk, content_1, "Appended chunck and chunck in dict not equal");
    

    let content_2: &[u8] = &[1, 1, 1, 1, 1];
    analizer.append_dict(content_2);

    let dict = analizer.get_dict();
    assert_eq!(dict.len(), 2, "Dict not append");
    let get_content_2 = &dict[&hash_chunk(&content_2[..4])];
    assert_eq!(get_content_2.get_chunk(), content_2[..4], "Appended chunck and chunck in dict not equal");
    assert_eq!(get_content_2.get_occurrence_num(), 2, "Appended chunck occurence num not expected");


    /* Check situation
    has [2, 3, 4]

    Check all three an first found [1, 2, 3] not see that [2 3 4] is exist and not do +3 like in end
    1 2 3 4 5 1 2 3 4 5 6
    [   ] . . . . . . .
    . [   ] . . . . . .
    . . [   ] . . . . .
    . . . [   ] . . . .
    . . . . [   ] . . .
    . . . . . [   ] . .
    . . . . . . [   ]
                  [    ]
    | x | | | x x x
    n   n n n                 <- new chuncks

    [1 2 3] 2
    [2 3 4] 3
    [3 4 5] 2
    [4 5 1] 1
    [5 1 2] 1
     */
    let mut analizer = FrequencyAnalyser::new_with_distribution(vec![(3, 1)]);
    
    let content_1: &[u8] = &[2, 3, 4];
    analizer.append_dict(content_1);

    let dict = analizer.get_dict();
    assert_eq!(dict.len(), 1, "Dict not append");
    let get_content_1 = &dict[&hash_chunk(content_1)];
    assert_eq!(get_content_1.chunk, content_1, "Appended chunck and chunck in dict not equal");
    

    let content_2: &[u8] = &[1, 2, 3, 4, 5, 1, 2, 3, 4, 5, 6];
    let expected_add: &[([u8; 3], u32)] = &[
        ([1, 2, 3], 2),
        ([2, 3, 4], 3),
        ([3, 4, 5], 2),
        ([4, 5, 1], 1),
        ([5, 1, 2], 1),
    ];
    analizer.append_dict(content_2);

    let dict = analizer.get_dict();
    assert_eq!(dict.len(), 5, "Not expected dict size");
    
    for (chunsk, occ_num) in expected_add {
        let get_content = &dict[&hash_chunk(chunsk)];
        assert_eq!(get_content.get_chunk(), chunsk, "Appended chunck and chunck in dict not equal");
        assert_eq!(get_content.get_occurrence_num(), *occ_num, "Appended chunck occurence num not expected");
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
    pub fn print_with_chunck(&self) {
        Self::print(self);
        println!("{:?}", self.chunk);
    }
    pub fn print(&self) {
        println!("{} {} {}", self.hash, self.occurrence_num, self.size);
    }

    pub fn save_to_file(&self, file: &mut fs::File) -> Result<(), io::Error> {
        file.write(FBCHash::to_be_bytes(self.hash).as_slice())?;
        file.write(u32::to_be_bytes(self.occurrence_num).as_slice())?;
        file.write(usize::to_be_bytes(self.size).as_slice())?;
        file.write(self.chunk.as_slice())?;

        Ok(())
    }

    fn load_with_out_chunk(file: &mut fs::File) -> Result<Self, io::Error> {
        let mut result = DictRecord::default();

        /* indexes of data in buffer */
        const HASH: usize = size_of::<FBCHash>();
        const OCC_NUM: usize = size_of::<u32>() + HASH;
        const SIZE: usize = size_of::<usize>() + OCC_NUM;
        /* create buffer */
        let mut buffer =  [0; SIZE];
        /* read buffer */
        file.read(&mut buffer)?;
        /* read data from buffer by indexes */
        result.hash = FBCHash::from_be_bytes(buffer[0..HASH].try_into().unwrap());
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




