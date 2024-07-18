
use chunkfs::{ChunkHash, Database, DataContainer, Scrub};
use crate::analyser::Analyser;
use crate::storage::FBCKey;

pub struct FBCScrubber {
    analyser: Analyser,
}
impl FBCScrubber {
    pub fn new() -> FBCScrubber {
        FBCScrubber {
            analyser: Analyser::default()
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
    )
    where Hash: 'a,
    {
        for (_, data_container) in database.into_iter() {
            let chunk = data_container.extract();
            self.analyser.deduplicate("database", target_map);
            //target_map.insert(hash(), chunk.clone() as Vec<u8>).unwrap()
        }
    }
}