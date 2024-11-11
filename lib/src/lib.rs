mod analyser;
pub mod storage;
mod test;

use crate::analyser::Analyser;
use crate::storage::FBCKey;
use chunkfs::{ChunkHash, Data, DataContainer, Database, Scrub, ScrubMeasurements};
use std::hash::{DefaultHasher, Hash, Hasher};
use std::time::Instant;

// ChunkFS scrubber implementation
pub struct FBCScrubber {
    analyser: Analyser,
}
impl FBCScrubber {
    pub fn new() -> FBCScrubber {
        FBCScrubber {
            analyser: Analyser::default(),
        }
    }
}
impl<Hash: ChunkHash, B> Scrub<Hash, B, FBCKey> for FBCScrubber
where
    B: Database<Hash, DataContainer<FBCKey>>,
    for<'a> &'a mut B: IntoIterator<Item = (&'a Hash, &'a mut DataContainer<FBCKey>)>,
{
    fn scrub<'a>(
        &mut self,
        database: &mut B,
        target_map: &mut Box<dyn Database<FBCKey, Vec<u8>>>,
    ) -> Result<ScrubMeasurements, std::io::Error>
    where
        Hash: 'a,
    {
        let mut processed_data = 0;
        let mut data_left = 0;
        let start_time = Instant::now();
        for (_, data_container) in database.into_iter() {
            let mut chunk = data_container.extract();
            match chunk {
                Data::Chunk(data_ptr) => {
                    self.analyser.make_dict(data_ptr);
                    let y = data_ptr.to_vec().as_slice();
                    let tmp_key = FBCKey::new(hash_chunk(data_ptr), false);
                    target_map
                        .insert(tmp_key, data_ptr.to_vec().clone())
                        .unwrap()
                }
                _ => {}
            }
        }
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
    return hasher.finish();
}
