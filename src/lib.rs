pub mod fbc_chunker;
pub mod frequency_analyser;
pub mod storage;

use std::cell::RefCell;
use std::thread;

use crate::fbc_chunker::ChunkerFBC;
use crate::frequency_analyser::{count_deps, DictRecord, FrequencyAnalyser};
use frequency_analyser::append_dict;
use crate::storage::{FBCKey, FBCMap};
use chunkfs::{
    ChunkHash, Data, DataContainer, Database, IterableDatabase, Scrub, ScrubMeasurements,
};
use std::collections::HashMap;
use std::fs;
use std::sync::{Arc, Mutex};

use std::hash::{DefaultHasher, Hash, Hasher};
use std::time::Instant;
// ChunkFS scrubber implementation

const THREADS_COUNT: usize = 16;

pub struct FBCScrubber {
    pub analyser: Mutex<FrequencyAnalyser>,
    pub chunker: ChunkerFBC,
}
impl Default for FBCScrubber {
    fn default() -> Self {
        Self::new()
    }
}

impl FBCScrubber {
    pub fn new() -> FBCScrubber {
        FBCScrubber {
            analyser: Mutex::new(FrequencyAnalyser::new()),
            chunker: ChunkerFBC::default(),
        }
    }
}
impl<Hash: ChunkHash + 'static, B> Scrub<Hash, B, FBCKey, FBCMap> for FBCScrubber
where
    B: IterableDatabase<Hash, DataContainer<FBCKey>>,
    for<'a> &'a mut B: IntoIterator<Item=(&'a Hash, &'a mut DataContainer<FBCKey>)>,
{
    fn scrub<'a>(
        &mut self,
        database: &mut B,
        target_map: &mut FBCMap,
    ) -> Result<ScrubMeasurements, std::io::Error>
    where
        Hash: 'a,
    {
        let mut processed_data = 0;
        let mut data_left = 0;
        let mut cdc_data = 0;
        let start_time = Instant::now();
        let mut kdata = 0;
        let mut pointers_vec: Vec<[Option<&Vec<u8>>; THREADS_COUNT]> = vec![[None; THREADS_COUNT]; 1];
        for (_, data_container) in database.into_iter() {
            let chunk = data_container.extract();
            if let Data::Chunk(data_ptr) = chunk {
                kdata += data_ptr.len() + 8;
                match pointers_vec[pointers_vec.len() - 1][THREADS_COUNT - 1] {
                    Some(_) => {
                        let mut tmp_thread_array = [None; THREADS_COUNT];
                        tmp_thread_array[0] = Some(data_ptr);
                        pointers_vec.push(tmp_thread_array);
                    }
                    None => for thread_num in 0..THREADS_COUNT {
                        if pointers_vec[pointers_vec.len() - 1][thread_num].is_none() {
                            let ln = pointers_vec.len();
                            pointers_vec[ln - 1][thread_num] = Some(data_ptr);
                            break;
                        }
                    },
                }
                /*
                println!("{:?}", pointers_vec[pointers_vec.len() - 1].map(|x| {
                    match x {
                        Some(data_ptr) => Some(data_ptr[0..4].to_vec()),
                        None => None
                    }
                }))
                */
            }
        }
        let mut kmap: Arc<Mutex<HashMap<u64, DictRecord>>> = Arc::new(Mutex::new(HashMap::new()));
        for data_ptr in pointers_vec.into_iter() {
            for i in 0..THREADS_COUNT {
                cdc_data += data_ptr[i].unwrap().len() + 8;
            }
            let mut thread_ids = vec![];
            for i in 0..THREADS_COUNT {
                thread_ids.push(i);
            }
            thread::scope(|s| {
                for i in &thread_ids {
                    let tmap = kmap.clone();
                    s.spawn(move || {
                        match data_ptr[*i] {
                            Some(ptr) => {
                                append_dict(tmap, ptr);
                                //println!("THREAD_ID: {} [{:?}, {:?}, {:?}, {:?}]", *i, ptr[0], ptr[1], ptr[2], ptr[3]);
                            }
                            None => {}
                        }
                    });
                }
            });
            for i in 0..THREADS_COUNT {
                match data_ptr[i] {
                    Some(ptr) => {
                        self.chunker.add_cdc_chunk(ptr);
                        let tmp_key = FBCKey::new(hash_chunk(ptr), false);
                        target_map
                            .insert(tmp_key, ptr.to_vec().clone())?
                    }
                    None => {}
                }
            }
            if cdc_data % 4 == 0 {
                println!(
                    "Data Left: ({}/{}) Scrubbed: % {}, dups/size: {:?}",
                    cdc_data,
                    kdata,
                    (cdc_data as f32 / kdata as f32) * 100.0,
                    count_deps(2, &kmap.lock().unwrap()),
                );
            }
        }
        /*
            self.analyser.process_dictionary();
            self.analyser.reduce_low_occur();
            let dct = self.analyser.get_dict();
            data_left = self.chunker.fbc_dedup(&dct);
        */
        let running_time = start_time.elapsed();

        Ok(ScrubMeasurements {
            processed_data,
            running_time,
            data_left,
        })
    }
}

//Hashcode that uses chunker to put it into target_map
fn hash_chunk(data_ptr: &Vec<u8>) -> u64 {
    let mut hasher = DefaultHasher::new();
    Hash::hash_slice(data_ptr.to_vec().as_slice(), &mut hasher);
    hasher.finish()
}
