use fbc_scrubber::fbc_chunker::ChunkerFBC;
use fbc_scrubber::frequency_analyser::{DictRecord, FrequencyAnalyser};
use std::fs::{self, File};

use dashmap::DashMap;
use std::hash::{DefaultHasher, Hasher};
use std::io::{BufWriter, Write};
use std::str::FromStr;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Duration, Instant};

fn save_map(file_name: &str, saved_map: Arc<DashMap<u64, DictRecord>>) -> std::io::Result<()> {
    let mut string_out = String::new();
    string_out.push_str(saved_map.len().to_string().as_str());

    for element in saved_map.iter() {
        string_out.push_str("1");
    }

    let file = std::fs::write(file_name, string_out)?;
    Ok(())
}

fn f(name: &str, dt: usize, analize_sizes: Vec<usize>) -> Option<(f64, f64, usize, Duration, Duration, Duration, Duration)> {
    let path_string = "../test_files_input/".to_string() + name;
    let path = std::path::Path::new(path_string.as_str());
    let contents = fs::read(&path).expect("Should have been able to read the file");
    
    let mut analyser;
    let save_file_string = "./save_analizer_".to_string() + name + ".txt";
    let save_file_path = std::path::Path::new(save_file_string.as_str());

    let now = Instant::now();
    if !fs::exists(save_file_path).unwrap() {
        println!("file not exist");

        let all_sizes = [
            512, 256, 64
            // 512,
            // 1024,
            // 2048,
            // 4096,
        ].to_vec();
        analyser = FrequencyAnalyser::new_with_sizes(all_sizes);
        println!("{:?}", analyser.get_chunk_partitioning());
        analyser.append_dict(&contents);

        for i in 1..40 {
            println!("{} {}", i, analyser.count_candidates(i))
        }
        analyser.reduce_low_occur(3);
        analyser.save_to_file(save_file_path).unwrap();

        println!("file saved");
    } else {
        println!("file exist");

        analyser = FrequencyAnalyser::load_from_file(save_file_path).unwrap();
        
        println!("file load");
    }
    analyser.trim_to_sizes(analize_sizes.as_slice());

    let analizer_preparation_time = now.elapsed();
    
    let mut chunker = ChunkerFBC::default();

    let now = Instant::now();
    let mut i = 0;
    while i < contents.len() - dt {
        chunker.add_cdc_chunk(&contents[i..i + dt]);
        i += dt;
    }
    chunker.add_cdc_chunk(&contents[i..contents.len()]);
    let cdc_chunk_add_time = now.elapsed();

    let a = analyser.get_dict();

    // for (k, v) in a.iter() {
    //     if v.get_occurrence_num() > 1 {
    //         println!("hash: {k} size: {} occ_num: {} ", v.get_size(), v.get_occurrence_num());
    //     }
    //     // println!("chnk:\n{:?}", v.get_chunk());
    // }

    let now = Instant::now();
    let dedup = chunker.fbc_dedup(&a, analyser.get_chunk_partitioning());
    let dedup_time = now.elapsed();

    let now = Instant::now();
    let rededup = chunker.reduplicate("out.txt");
    let rededup_time = now.elapsed();

    let pure_size = chunker.get_size_pure_chunks();
    let count_chuncks = chunker.get_count_chunks();

    let eq = fs::read(path).expect("Should have been able to read lowinput") != fs::read("out.txt").expect("Should have been able to read out file");
    fs::remove_file("out.txt").unwrap();
    
    if eq
    {
        let mut name = String::new();
        println!("");
        println!(
            "NOT MATCH {} {}",
            fs::metadata(path).unwrap().len(),
            fs::read("out.txt").unwrap().len()
        );
        let _ = fs::write("_out.txt", chunker.reduplicate_by_chunks());
        let _ = std::io::stdin().read_line(&mut name);
        println!("");
        println!("");
        None
    } else {
        Some((
            rededup as f64 / dedup as f64,
            rededup as f64 / pure_size as f64,
            count_chuncks,
            analizer_preparation_time,
            cdc_chunk_add_time,
            dedup_time,
            rededup_time,
        ))
    }
}

fn main() {
    const KB: usize = 1024 * 8;
    let names = [
        "linux-3.4.6-7.tar"
        // "fbc_topic_input.txt",
        // "lowinput.txt",
        // "orient_express_input.txt",
    ];
    let dts = [
        1024,
        2048,
        4096
        // KB
        // 6 * KB,
        // 8 * KB,
        // 10 * KB,
        // 12 * KB,
        // 16 * KB,
        // 32 * KB,
    ];

    let all_sizes: &[Vec<usize>] = &[
        [512, 256, 64].to_vec(),
        [512, 256].to_vec(),
        // [4096, 2048, 1024, 512].to_vec(),
        // [512].to_vec(),
        // [1024].to_vec(),
        // [2048].to_vec(),
        // [4096].to_vec(),
        // [8192].to_vec(),
    ];

    let mut str_out =
        String::from_str("file_name\tdt\tsizes\tdedup_coef\tpure_size_ratio\tcount_chunks\tanalizer_preparetion_time\tadd_cdc_chunck_time\tdedup_time\trededup_time\n")
            .unwrap();

    for name in names {
        for dt in dts {
            for sizes in all_sizes.iter() {
                let mut this_out = String::new();
                this_out.push_str(name);
                this_out.push_str("\t");
                this_out.push_str(dt.to_string().as_str());
                this_out.push_str("\t");
                for s in sizes {
                    this_out.push_str(s.to_string().as_str());
                    this_out.push_str(" ");
                }
                this_out.push_str("\t");

                println!("{this_out}");
                match f(name, dt, sizes.clone()) {
                    Some(res) => {
                        this_out.push_str(res.0.to_string().as_str());
                        this_out.push_str("\t");
                        this_out.push_str(res.1.to_string().as_str());
                        this_out.push_str("\t");
                        this_out.push_str(res.2.to_string().as_str());
                        // analizer
                        this_out.push_str("\t");
                        this_out.push_str(res.3.as_secs_f64().to_string().as_str());
                        // add cdc
                        this_out.push_str("\t");
                        this_out.push_str(res.4.as_secs_f64().to_string().as_str());

                        // dedup
                        this_out.push_str("\t");
                        this_out.push_str(res.5.as_secs_f64().to_string().as_str());
                        // rededup
                        this_out.push_str("\t");
                        this_out.push_str(res.6.as_secs_f64().to_string().as_str());
                    }
                    None => {
                        this_out.push_str("NOT MATCH");
                    }
                }

                this_out.push_str("\n");
                println!("{this_out}");

                str_out.push_str(&this_out);
            }
        }
    }

    fs::write("experement_result.csv", str_out.as_bytes()).unwrap();

    for name in names {
        fs::remove_file("./save_analizer_".to_string() + name + ".txt").unwrap();
    }
}
