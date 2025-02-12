use chunkfs::Database;
use std::collections::HashMap;
use std::hash::Hash;
use std::io;

#[derive(Hash, PartialEq, Eq, Clone)]
pub struct FBCKey {
    key: u64,
    state: bool,
}
impl FBCKey {
    pub fn new(key: u64, state: bool) -> FBCKey {
        FBCKey { key, state }
    }
}

pub struct FBCMap {
    fbc_hashmap: HashMap<FBCKey, Vec<u8>>,
}

impl Default for FBCMap {
    fn default() -> Self {
        Self::new()
    }
}

impl Database<FBCKey, Vec<u8>> for FBCMap {
    fn insert(&mut self, fbc_hash: FBCKey, chunk: Vec<u8>) -> io::Result<()> {
        self.fbc_hashmap.insert(fbc_hash, chunk);
        Ok(())
    }

    fn get(&self, hash: &FBCKey) -> io::Result<Vec<u8>> {
        let chunk = self.fbc_hashmap.get(hash).cloned().unwrap();
        Ok(chunk)
    }
    fn contains(&self, key: &FBCKey) -> bool {
        self.fbc_hashmap.contains_key(key)
    }
}

impl FBCMap {
    pub fn new() -> FBCMap {
        FBCMap {
            fbc_hashmap: HashMap::default(),
        }
    }
}
