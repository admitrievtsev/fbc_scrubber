#![feature(ascii_char)]

pub mod fbc_chunker;
pub mod frequency_analyser;
pub mod storage;
mod test;

use crate::fbc_chunker::ChunkerFBC;
use crate::frequency_analyser::FrequencyAnalyser;
use crate::storage::{FBCKey, FBCMap};
use chunkfs::{
    ChunkHash, Data, DataContainer, Database, IterableDatabase, Scrub, ScrubMeasurements,
};
use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::time::Instant;

// ChunkFS scrubber implementation
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
impl<Hash: ChunkHash, B> Scrub<Hash, B, FBCKey, FBCMap> for FBCScrubber
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
        let mut processed_data = 0;
        let mut data_left = 0;
        let mut cdc_data = 0;
        let start_time = Instant::now();
        let mut kdata = 0;
        for (_, data_container) in database.into_iter() {
            let chunk = data_container.extract();
            if let Data::Chunk(data_ptr) = chunk {
                kdata += data_ptr.len() + 8;
            }
        }

        for (_, data_container) in database.into_iter() {
            let chunk = data_container.extract();
            if let Data::Chunk(data_ptr) = chunk {
                if cdc_data % 4 == 0 {
                    println!(
                        "Data Left: ({}/{}) Scrubbed: % {}",
                        cdc_data,
                        kdata,
                        (cdc_data as f32 / kdata as f32) * 100.0
                    );
                }
                cdc_data += data_ptr.len() + 8;

                self.analyser.make_dict(data_ptr);
                self.chunker.add_cdc_chunk(data_ptr);

                if cdc_data > 30000000 {
                    break;
                }

                if data_ptr.len() % 20 == 0 {
                    self.analyser.reduce_low_occur()
                }

                let y = data_ptr.to_vec();
                let tmp_key = FBCKey::new(hash_chunk(data_ptr), false);
                target_map
                    .insert(tmp_key, data_ptr.to_vec().clone())
                    .unwrap()
            }
        }
        self.analyser.process_dictionary();

        processed_data = cdc_data;
        self.analyser.reduce_low_occur();
        data_left = self.chunker.fbc_dedup(self.analyser.get_dict());
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
