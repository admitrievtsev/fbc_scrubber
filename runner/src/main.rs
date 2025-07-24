use fbc_scrubber::fbc_chunker::ChunkerFBC;
use fbc_scrubber::frequency_analyser::{FrequencyAnalyser, DictRecord};
use std::fs::{self, File};

use std::hash::{DefaultHasher, Hasher};
use std::io::{BufWriter, Write};
use std::str::FromStr;
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

fn f(name: &str, dt: usize, analize_sizes: Vec<usize>) -> Option<f64> {
    let mut analyser = FrequencyAnalyser::new_with_sizes(analize_sizes.clone());
    let mut chunker = ChunkerFBC::default();
    let path_string = "../test_files_input/".to_string() + name;
    let path = std::path::Path::new(path_string.as_str());
    let contents = fs::read(&path)
        .expect("Should have been able to read the file");
    analyser.append_dict(&contents);
    
    let mut i = 0;
    while i < contents.len() - dt {
        chunker.add_cdc_chunk(&contents[i..i + dt]);
        i += dt;
    }
    chunker.add_cdc_chunk(&contents[i..contents.len()]);
    let a = analyser.get_dict();

    // for (k, v) in a.iter() {
    //     if v.get_occurrence_num() > 1 {
    //         println!("hash: {k} size: {} occ_num: {} ", v.get_size(), v.get_occurrence_num());
    //     }
    //     // println!("chnk:\n{:?}", v.get_chunk());
    // }

    let dedup = chunker.fbc_dedup(&a, analyser.get_chunck_partitioning());

    let rededup = chunker.reduplicate("out.txt");
    
    println!("dedup: {}", rededup as f64 / dedup as f64);
    print!("name: {}\ndt: {}\nsizes: ", name, dt);
    for it in analize_sizes {
        print!("{it} ");
    }
    println!("");
    println!("");
    println!("");
    
    if fs::read(path)
            .expect("Should have been able to read lowinput")
        != 
        fs::read("out.txt")
            .expect("Should have been able to read out file")
    {
        let mut name = String::new();
        println!("NOT MATCH");
        std::io::stdin().read_line(&mut name);
        None
    } else {
        Some(rededup as f64 / dedup as f64)
    }

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

    let all_sizes = [
        vec![32], vec![64], vec![128], vec![256], 
        vec![64, 32], vec![128, 64, 32], vec![128, 64], vec![256, 128, 64], vec![256, 128],
        vec![256, 64], vec![128, 32]
    ];

    let mut str_out = String::from_str("file_name;dt;sizes;res\n").unwrap();

    for name in names {
        for dt in dts {
            for sizes in all_sizes.iter() {
                print!("name: {}\ndt: {}\nsizes: ", name, dt);
                for it in sizes {
                    print!("{it} ");
                }
                print!("\n\n");
                str_out.push_str(name);
                str_out.push_str(";");
                str_out.push_str(dt.to_string().as_str());
                str_out.push_str(";");
                for s in sizes {
                    str_out.push_str(s.to_string().as_str());
                    str_out.push_str(" ");
                }
                str_out.push_str(";");

                if name.to_string() == "lowinput.txt" &&
                    sizes.len() > 1 {
                    str_out.push_str("STACK OVERFLOW");
                    println!("STACK OVERFLOW\n");
                } else {
                    match f(name, dt, sizes.clone()) {
                        Some(res) => {
                            str_out.push_str(res.to_string().as_str());
                        }
                        None => {
                            str_out.push_str("NOT MATCH");
                        }
                    }
                }

                str_out.push_str("\n");
            }
        }
    }
    fs::write("experement_result.csv", str_out.as_bytes()).unwrap();
}
