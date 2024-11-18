pub mod analyser;
pub mod storage;
mod test;

use crate::analyser::Analyser;
use crate::storage::FBCKey;
use chunkfs::{ChunkHash, Data, DataContainer, Database, Scrub, ScrubMeasurements};
use std::hash::{DefaultHasher, Hash, Hasher};
use std::time::Instant;

// ChunkFS scrubber implementation
pub struct FBCScrubber {
    pub analyser: Analyser,
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
        let mut cdc_data = 0;
        let start_time = Instant::now();
        let mut kdata = 0;
        for (_, data_container) in database.into_iter() {
            let mut chunk = data_container.extract();
            match chunk {
                Data::Chunk(data_ptr) => {
                    kdata += data_ptr.len() + 8;
                }
                _ => {}
            }
        }

        for (_, data_container) in database.into_iter() {
            let mut chunk = data_container.extract();
            match chunk {
                Data::Chunk(data_ptr) => {
                    println!("Data Left: ({}/{}) Scrubbed: % {}", cdc_data, kdata, (cdc_data as f32 / kdata as f32) * 100.0);

                    cdc_data += data_ptr.len() + 8;


                    self.analyser.make_dict(data_ptr);
                    if(cdc_data > 150000){
                        break
                    }
                    let y = data_ptr.to_vec();
                    let tmp_key = FBCKey::new(hash_chunk(data_ptr), false);
                    target_map
                        .insert(tmp_key, data_ptr.to_vec().clone())
                        .unwrap()
                }
                _ => {}
            }
        }
        self.analyser.print_dict();
        processed_data = cdc_data;
        data_left = self.analyser.fbc_dedup();
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
