use fbc_scrubber::fbc_chunker::ChunkerFBC;
use fbc_scrubber::frequency_analyser::{FrequencyAnalyser, DictRecord};
use std::fs::{self, File};

use std::hash::{DefaultHasher, Hasher};
use std::io::{BufWriter, Write};
use dashmap::DashMap;
use std::sync::{Arc, Mutex, MutexGuard};

fn save_map(file_name: &str, saved_map: Arc<DashMap<u64, DictRecord>>) -> std::io::Result<()>{
    let mut string_out = String::new();
    string_out.push_str(saved_map.len().to_string().as_str());

    for element in saved_map.iter() {
        string_out.push_str("1");
    }

    let file = std::fs::write(file_name, string_out)?;
    Ok(())
}

fn f(name: &str, dt: usize) {
    let mut analyser = FrequencyAnalyser::new();
    let mut chunker = ChunkerFBC::default();
    let path_string = "../test_files_input/".to_string() + name;
    let path = std::path::Path::new(path_string.as_str());
    let contents = fs::read(&path)
        .expect("Should have been able to read the file");
    analyser.append_dict(&contents);
    
    let mut i = 0;
    while i < contents.len() - dt {
        chunker.add_cdc_chunk(&contents[i..i + dt].to_vec());
        i += dt;
    }
    chunker.add_cdc_chunk(&contents[i..contents.len()].to_vec());
    let a = analyser.get_dict();

    // for (k, v) in a.iter() {
    //     if v.get_occurrence_num() > 1 {
    //         println!("hash: {k} size: {} occ_num: {} ", v.get_size(), v.get_occurrence_num());
    //     }
    //     // println!("chnk:\n{:?}", v.get_chunk());
    // }

    let dedup = chunker.fbc_dedup(&a);

    let rededup = chunker.reduplicate("out.txt");
    if fs::read(path)
            .expect("Should have been able to read lowinput")
        == 
        fs::read("out.txt")
            .expect("Should have been able to read out file")
    {
        println!("1) {}", rededup as f64 / dedup as f64);
        println!("MATCH")
    }
    println!("");


    // fs::remove_file("out.txt").expect("File out.txt not exists in current directory");
}

fn main() {
    let names = [
        "fbc_topic_input.txt",
        "lowinput.txt",
        "orient_express_input.txt",
    ];
    let dts = [
        128 * 2, 128 * 3, 128 * 4, 128 * 5, 128 * 6, 128 * 7, 128 * 8
    ];

    for name in names {
        for dt in dts {
            f(name, dt);
        }
    }
    
}
