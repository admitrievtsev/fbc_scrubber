pub mod fbc_chunker;
pub mod frequency_analyser;
pub mod storage;

use crate::fbc_chunker::{ChunkerFBC, FBCHash};
use crate::frequency_analyser::FrequencyAnalyser;
use crate::storage::{FBCKey, FBCMap};
use chunkfs::{
    ChunkHash, Data, DataContainer, Database, IterableDatabase, Scrub, ScrubMeasurements,
};

use std::hash::{DefaultHasher, Hash, Hasher};
use std::time::Instant;
// ChunkFS scrubber implementation

const THREADS_COUNT: usize = 16;

pub struct FBCScrubber {
    pub analyser: FrequencyAnalyser,
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
            analyser: FrequencyAnalyser::new(),
            chunker: ChunkerFBC::default(),
        }
    }
}
impl<Hash: ChunkHash + 'static, B> Scrub<Hash, B, FBCKey, FBCMap> for FBCScrubber
where
    B: IterableDatabase<Hash, DataContainer<FBCKey>>,
    for<'a> &'a mut B: IntoIterator<Item = (&'a Hash, &'a mut DataContainer<FBCKey>)>,
{
    fn scrub<'a>(
        &mut self,
        database: &mut B,
        target_map: &mut FBCMap,
    ) -> Result<ScrubMeasurements, std::io::Error>
    where
        Hash: 'a,
    {
        let processed_data = 0;
        let data_left = 0;
        let mut cdc_data = 0;
        let start_time = Instant::now();
        let mut kdata = 0;
        let mut pointers_vec: Vec<[Option<&Vec<u8>>; THREADS_COUNT]> =
            vec![[None; THREADS_COUNT]; 1];
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
                    None => {
                        for thread_num in 0..THREADS_COUNT {
                            if pointers_vec[pointers_vec.len() - 1][thread_num].is_none() {
                                let ln = pointers_vec.len();
                                pointers_vec[ln - 1][thread_num] = Some(data_ptr);
                                break;
                            }
                        }
                    }
                }
            }
        }
        println!("Packs collected");
        for data_ptr in pointers_vec.into_iter() {
            //println!("Pack encounting");
            if cdc_data > 166925888 {
                break;
            }
            for ptr in data_ptr.iter().take(THREADS_COUNT).flatten() {
                cdc_data += ptr.len() + 8;
            }
            self.analyser.analyse_pack(data_ptr);

            //println!("Packs analyzed");
            //println!("THREAD_ID: {} [{:?}, {:?}, {:?}, {:?}]", *i, ptr[0], ptr[1], ptr[2], ptr[3]);

            for ptr in data_ptr.iter().take(THREADS_COUNT).flatten() {
                self.chunker.add_cdc_chunk(ptr);
                let tmp_key = FBCKey::new(hash_chunk(ptr), false);
                target_map.insert(tmp_key, ptr.to_vec().clone())?
            }
            if cdc_data % 40 == 0 {
                println!(
                    "Data Left: ({}/{}) Scrubbed: % {}, dups/size: ({}, {})",
                    cdc_data,
                    kdata,
                    (cdc_data as f32 / kdata as f32) * 100.0,
                    self.analyser.count_candidates(2),
                    self.analyser.dict.len()
                );
                // println!("{}", cdc_data % 1024 * 1024 * 32 - 1024 * 256 * THREADS_COUNT)
            }
            if cdc_data % (500 * 16 / THREADS_COUNT) == 0 {
                println!("ENTERED REDUCTION");
                self.analyser.reduce_low_occur(2);
            }
        }

        self.analyser.reduce_low_occur(2);
        // let dct = self.analyser.get_dict();
        //data_left = self.chunker.fbc_dedup(&dct);
        let running_time = start_time.elapsed();

        Ok(ScrubMeasurements {
            processed_data,
            running_time,
            data_left,
        })
    }
}

//Hashcode that uses chunker to put it into target_map
fn hash_chunk(data_ptr: &[u8]) -> FBCHash {
    // let mut hasher = DefaultHasher::new();
    // Hash::hash_slice(data_ptr, &mut hasher);
    // hasher.finish();

    let mut hasher = DefaultHasher::new();
    let delimer = data_ptr.len() / 2;
    Hash::hash_slice(&data_ptr[..delimer], &mut hasher);
    let hash1 = hasher.finish() as FBCHash;
    Hash::hash_slice(&data_ptr[delimer..], &mut hasher);
    let hash2 = hasher.finish() as FBCHash;
    hash1 << 64 | hash2
}
