use std::collections::{HashMap, VecDeque};
use std::hash::{DefaultHasher, Hasher};
use std::string::String;
use std::vec::Vec;

use dashmap::DashMap;
use std::sync::Arc;

use crate::fbc_chunker::FBCChunk::{Sharped, Solid};
use crate::frequency_analyser::DictRecord;
use crate::hash_chunk;

// Parameter of FSChunker
pub type FBCHash = u128;

enum FBCChunk {
    Solid(Vec<u8>),
    Sharped(Vec<FBCHash>),
}

#[allow(dead_code)]
impl FBCChunk {
    /// calculate len of chunk
    fn len(&self, map: &HashMap<FBCHash, FBCChunk>) -> usize {
        match &self {
            Solid(sl) => sl.len(),
            Sharped(sh) => sh.iter().fold(0, |acc, element| {
                acc + Self::len(map.get(element).expect("Chunk NPE"), map)
            }),
        }
    }
}

/// Struct that provides frequency analysis for chunks
#[derive(Default)]
pub struct ChunkerFBC {
    chunk_ids: Vec<FBCHash>,
    chunks: HashMap<FBCHash, FBCChunk>,
}

#[allow(dead_code)]
impl ChunkerFBC {
    /// hash chunk and insert solid chunk in chunks
    // maybe add_chunk ?
    fn insert_chunk(&mut self, chunk: &[u8]) -> FBCHash {
        let hash = hash_chunk(chunk);
        self.chunks.entry(hash).or_insert(Solid(chunk.to_vec()));
        hash
    }
    fn insert_chunk_vec(&mut self, chunk: Vec<u8>) -> FBCHash {
        let hash = hash_chunk(chunk.as_slice());
        self.chunks.entry(hash).or_insert(Solid(chunk));
        hash
    }
    // maybe insert_cdc_chunk ?
    pub fn add_cdc_chunk(&mut self, first_stage_chunk: &[u8]) {
        let res = self.insert_chunk(first_stage_chunk);
        self.chunk_ids.push(res);
    }

    //This method is to print chunking results out
    fn to_str(word: &[u8]) -> String {
        String::from_utf8_lossy(word).to_string()
    }

    //Updating analyser occurrences counter

    //The main method which makes text dedup || DEBUG ONLY
    /*
    pub fn deduplicate(&mut self, file_in: &str, file_out: &str) {
        self.simple_dedup(file_in);
        self.throw_chunks_to_maker();
        self.reduplicate(file_out);
    }

     */
    fn reconstruct_chunk_from_hash<'a>(
        &'a self,
        mut prev: Vec<&'a Vec<u8>>,
        hash: &FBCHash,
        mut _depth: u32,
    ) -> Vec<&'a Vec<u8>> {
        match self.chunks.get(hash).expect("Chunk NPE") {
            Solid(chunk) => prev.push(chunk),
            Sharped(chunks) => {
                // print!("{depth}, {} {}", chunks.len(), hash);
                // for it in chunks.iter() {
                //     print!(" {it}");
                // }
                // println!();

                _depth += 1;
                for it in chunks {
                    prev = Self::reconstruct_chunk_from_hash(self, prev, it, _depth);
                }
            }
        }
        prev
    }

    // Method that write text dedup out
    pub fn reduplicate(&self, expected_size: usize) -> Vec<u8> {
        // let mut file = fs::OpenOptions::new()
        //     .create(true)
        //     .write(true)
        //     .open(file_out)
        //     .expect("Unable to write the file");
        // let mut all_len = 0;
        // for id in self.chunk_ids.iter() {
        //     let out_string = Self::to_str(Self::reconstruct_chunk_from_hash(self, id, 0));
        //     all_len += out_string.len();
        //     file.write(out_string.as_bytes()).expect("Unable to write the file");
        // }
        // println!("{}", all_len);
        // all_len
        let mut all = Vec::with_capacity(expected_size);
        for id in self.chunk_ids.iter() {
            for it in Self::reconstruct_chunk_from_hash(self, Vec::new(), id, 0) {
                for c in it {
                    all.push(*c);
                }
            }
        }
        all
    }
    pub fn reduplicate_by_chunks(&self) -> String {
        let mut string_out = String::new();
        for id in self.chunk_ids.iter() {
            string_out.push_str("{\n");
            for it in Self::reconstruct_chunk_from_hash(self, Vec::new(), id, 0) {
                string_out.push_str(&String::from_utf8_lossy(it.clone().as_slice()));
            }
            string_out.push_str("\n}\n");
        }
        //println!("PRINT TO FILE");
        string_out
    }
    // This method contains FBC chunker implementation
    /// chunk_partitioning - size, offset
    /// first places get first check
    /// if approach big and small sizes win is the first one specified
    pub fn fbc_dedup(
        &mut self,
        dict: Arc<DashMap<FBCHash, DictRecord>>,
        chunk_partitioning: &[(usize, usize)],
    ) {
        let mut fs = 0;
        let mut fd = 0;
        let min_chunk_size = chunk_partitioning
            .iter()
            .map(|x| x.0)
            .min()
            .expect("panic on calculate min chunk size, fbs deduplicate");
        let mut k = 0;
        let mut chunk_deque: VecDeque<FBCHash> = VecDeque::new();
        for i in self.chunk_ids.iter() {
            chunk_deque.push_front(*i);
        }

        while !chunk_deque.is_empty() {
            if k % 100000 == 0 {
                println!("Checked: {}", chunk_deque.len())
            }
            k += 1;

            // get hash
            let chunk_index = chunk_deque.pop_back().unwrap();

            // if current chunk already exist in dict
            if dict.contains_key(&chunk_index) {
                break;
            }

            // create reference to chunk to cut
            let unchecked_chunk = match &self.chunks.get(&chunk_index).expect("Chunk NPE") {
                Sharped(_) => continue,
                Solid(chunk) => chunk,
            };
            // move in chunk
            let mut chunk_char = 0;
            while (chunk_char as i128) < unchecked_chunk.len() as i128 - min_chunk_size as i128 {
                let mut chunk_hash = 0;
                let mut chunck_len = None;

                for (size, _) in chunk_partitioning.iter() {
                    if (chunk_char as i128) + (*size as i128) < unchecked_chunk.len() as i128 + 1 {
                        chunk_hash = hash_chunk(&unchecked_chunk[chunk_char..chunk_char + size]);
                        if dict.contains_key(&chunk_hash) {
                            // dist record have hash
                            // println!("found in dict {fd}");
                            chunck_len = Some(*size);

                            fd += 1;

                            break;
                        }
                        if self.chunks.contains_key(&chunk_hash) {
                            // dist record have hash
                            // println!("found in self.chunks {fs}");
                            chunck_len = Some(*size);

                            fs += 1;

                            break;
                        }
                    }
                }

                if let Some(chunck_len) = chunck_len {
                    if chunk_char == 0 {
                        // if big chunk start from is known

                        // let clone = unchecked_chunk.clone();
                        let new_chunk = unchecked_chunk[chunck_len..].to_vec();
                        let new_hash = self.insert_chunk_vec(new_chunk.clone());

                        // add new chunk for analize
                        chunk_deque.push_front(new_hash);

                        self.replace_all_two(chunk_index, chunk_hash, new_hash);
                    } else if chunk_char + chunck_len == unchecked_chunk.len() {
                        // if is known chunk in end of big chunk

                        let new_chunk = unchecked_chunk[..chunk_char].to_vec();
                        let new_hash = self.insert_chunk_vec(new_chunk);

                        // not add chunk for analize

                        self.replace_all_two(chunk_index, new_hash, chunk_hash);
                    } else {
                        // if is known chunk in midle of big chunk

                        // start
                        let new_chunk_1st = unchecked_chunk[..chunk_char].to_vec();
                        //end
                        let new_chunk_2st = unchecked_chunk[chunk_char + chunck_len..].to_vec();

                        let new_hash_1st = self.insert_chunk_vec(new_chunk_1st);
                        let new_hash_2nd = self.insert_chunk_vec(new_chunk_2st);

                        // add new chunk for analize
                        chunk_deque.push_front(new_hash_2nd);

                        self.replace_all_three(chunk_index, new_hash_1st, chunk_hash, new_hash_2nd);
                    }

                    if !self.chunks.contains_key(&chunk_hash) {
                        let _ = self.insert_chunk(dict.get(&chunk_hash).unwrap().get_chunk_ref());
                    }

                    break;
                } else {
                    chunk_char += 1;
                }
            }
        }
        println!("f self {fs} f dict {fd}");
    }

    pub fn get_dedup_len(&self) -> usize {
        let mut all_size = 0;
        let mut all_len = 0;

        for it in &self.chunk_ids {
            let mut q_res = Vec::with_capacity(50);
            q_res.push(*it);

            while !q_res.is_empty() {
                match self.chunks.get(&q_res.pop().unwrap()).unwrap() {
                    FBCChunk::Solid(chunk) => {
                        all_size += chunk.len();
                        all_len += 1;
                    }
                    FBCChunk::Sharped(chunk) => {
                        for it in chunk {
                            q_res.push(*it);
                        }
                    }
                }
            }
        }
        all_size + all_len * 8
    }

    /// return size of solid chunks
    pub fn get_size_pure_chunks(&self) -> usize {
        let mut size = 0;
        for it in &self.chunks {
            if let FBCChunk::Solid(chunk) = &it.1 {
                size += chunk.len();
            }
        }
        size
    }
    pub fn get_count_chunks(&self) -> usize {
        self.chunks.len()
    }

    fn hash_chunk(tmp_vec: &Vec<u8>) -> u64 {
        let mut hasher = DefaultHasher::new();
        hasher.write(tmp_vec.as_slice());
        hasher.finish()
    }

    // Slicing chunk on 2 different
    fn replace_all_two(&mut self, to_change: FBCHash, first: FBCHash, second: FBCHash) {
        *self.chunks.get_mut(&to_change).unwrap() = FBCChunk::Sharped(vec![first, second]);
    }

    // Slicing chunk on 3 different
    fn replace_all_three(
        &mut self,
        to_change: FBCHash,
        first: FBCHash,
        second: FBCHash,
        third: FBCHash,
    ) {
        *self.chunks.get_mut(&to_change).unwrap() = FBCChunk::Sharped(vec![first, second, third]);
    }

    // Optimization Method
    // You can call this method to reduce analyser records with low frequency
    // It will force scrubber to run faster but also will reduce dedup gain
    fn dict_count_size(&self) -> usize {
        self.chunks
            .values()
            .fold(0, |acc, x| acc + x.len(&self.chunks))
    }

    // compute size to storage chunks
    fn dict_size(&self) -> usize {
        self.chunks.values().fold(0, |acc, x| {
            acc + match &x {
                FBCChunk::Solid(chunk) => chunk.len() + size_of::<FBCHash>(),
                // len of chunk + self hash
                FBCChunk::Sharped(chunks) => {
                    chunks.len() * size_of::<FBCHash>() + size_of::<usize>() + size_of::<FBCHash>()
                } // len of chunks    self hash
            }
        })
    }
}

mod test {
    #[test]
    #[ignore]
    fn fbc_chunker_dedup_test() {
        use crate::fbc_chunker::FBCChunk;
        use crate::{hash_chunk, ChunkerFBC, FrequencyAnalyser};

        let mut chunker = ChunkerFBC::default();

        {
            let data_1: &[u8] = &[1, 2, 3, 4, 5, 6];
            chunker.add_cdc_chunk(data_1);

            let mut analyser = FrequencyAnalyser::new();
            let data_1_hash = hash_chunk(data_1);
            analyser.append_dict(data_1);

            chunker.fbc_dedup(analyser.get_dict(), analyser.get_chunk_partitioning());

            let res = if let FBCChunk::Solid(res) = chunker.chunks.get(&data_1_hash).unwrap() {
                res
            } else {
                panic!("Not expected chunk type")
            };
            assert_eq!(res, data_1, "chunk not dedup");
            assert_eq!(chunker.chunks.len(), 1, "chunks len not expected");
            assert_eq!(
                chunker.chunk_ids,
                vec![data_1_hash],
                "chunk id not expected"
            );
            // [1, 2, 3, 4, 5, 6]
        }
        {
            let mut analyser = FrequencyAnalyser::new_with_partitioning(vec![(3, 1)]);
            let data_1: &[u8] = &[1, 2, 3];
            let data_2: &[u8] = &[4, 5, 6];
            let data_prev_hash = hash_chunk(&[1, 2, 3, 4, 5, 6]);
            let data_1_hash = hash_chunk(data_1);
            let data_2_hash = hash_chunk(data_2);

            analyser.append_dict(data_1);
            analyser.append_dict(data_2);

            chunker.fbc_dedup(analyser.get_dict(), analyser.get_chunk_partitioning());

            assert_eq!(chunker.chunks.len(), 3, "chunks len not expected");
            let res = if let FBCChunk::Sharped(res) = chunker.chunks.get(&data_prev_hash).unwrap() {
                res
            } else {
                panic!("Not expected chunk type")
            };
            assert_eq!(
                *res,
                vec![data_1_hash, data_2_hash],
                "chunk id not expected"
            );
        }
        {
            let mut chunker = ChunkerFBC::default();
            let mut analyser = FrequencyAnalyser::new_with_partitioning(vec![(3, 1)]);

            //                  [     ]  [              ]
            //                           [     ]  [     ]
            let data: &[u8] = &[1, 1, 2, 1, 2, 1, 1, 2, 1];
            let data_hash = hash_chunk(data);
            let data_slice_hash = hash_chunk(&data[3..]);
            let hash_1 = hash_chunk(&[1, 1, 2]);
            let hash_2 = hash_chunk(&[1, 2, 1]);

            chunker.add_cdc_chunk(data);

            analyser.append_dict(data);

            chunker.fbc_dedup(analyser.get_dict(), analyser.get_chunk_partitioning());

            assert_eq!(chunker.chunks.len(), 3 + 1, "chunks len not expected");
            let res = if let FBCChunk::Sharped(res) = chunker.chunks.get(&data_hash).unwrap() {
                res
            } else {
                panic!("Not expected chunk type")
            };
            assert_eq!(*res, vec![hash_1, data_slice_hash], "chunk id not expected");

            let res = if let FBCChunk::Sharped(res) = chunker.chunks.get(&data_slice_hash).unwrap()
            {
                res
            } else {
                panic!("Not expected chunk type")
            };
            assert_eq!(*res, vec![hash_2, hash_2], "chunk id not expected");
        }
    }
}
