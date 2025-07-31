use fbc_scrubber::fbc_chunker::ChunkerFBC;
use fbc_scrubber::frequency_analyser::FrequencyAnalyser;
use std::fs::{self};

use std::io::Write;
use std::str::FromStr;
use std::time::{Duration, Instant};

fn f(
    name: &str,
    dt: usize,
    analize_sizes: Vec<usize>,
) -> Option<(f64, f64, usize, Duration, Duration, Duration, Duration)> {
    let path_string = "../test_files_input/".to_string() + name;
    let path = std::path::Path::new(path_string.as_str());
    let contents = fs::read(path).expect("Should have been able to read the file");

    let save_file_string = "./save_analizer_".to_string() + name + ".txt";
    let save_file_path = std::path::Path::new(save_file_string.as_str());

    println!("Loading...");
    let now: Instant = Instant::now();
    let mut analyser =
        FrequencyAnalyser::load_from_file_with_sizes(save_file_path, analize_sizes.clone())
            .unwrap();
    let analizer_preparation_time = now.elapsed();

    // if analize_sizes[0] == 256 {
    //     analyser.reduce_low_occur(3);
    // }

    let analyser_dict = analyser.get_dict();
    println!("Load complete: {}", analyser_dict.len());

    let mut chunker = ChunkerFBC::default();

    println!("Add cdc...");
    let now = Instant::now();
    let mut i = 0;
    while i < contents.len() - dt {
        chunker.add_cdc_chunk(&contents[i..i + dt]);
        i += dt;
    }
    chunker.add_cdc_chunk(&contents[i..contents.len()]);
    let cdc_chunk_add_time = now.elapsed();

    println!("Add cdc complete: {}", i + 1);

    // for (k, v) in a.iter() {
    //     if v.get_occurrence_num() > 1 {
    //         println!("hash: {k} size: {} occ_num: {} ", v.get_size(), v.get_occurrence_num());
    //     }
    //     // println!("chnk:\n{:?}", v.get_chunk());
    // }

    println!("Dedup...");
    let now = Instant::now();
    chunker.fbc_dedup(analyser_dict, analyser.get_chunk_partitioning());
    let dedup_time = now.elapsed();
    println!("Dedup comlete");

    println!("Calculate dedup coef...");
    let dedup = chunker.get_dedup_len();
    println!("Dedup coef: {}", dedup);

    println!("Rededup...");
    let now = Instant::now();
    let rededup = chunker.reduplicate(contents.len());
    let rededup_time = now.elapsed();

    println!("Rededup comlete: {}", rededup.len());

    let pure_size = chunker.get_size_pure_chunks();
    let count_chuncks = chunker.get_count_chunks();

    let eq = rededup != contents;

    if eq {
        let mut name = String::new();
        println!();
        println!("NOT MATCH {} {}", contents.len(), rededup.len());
        let _ = fs::write("_out.txt", chunker.reduplicate_by_chunks());
        let _ = std::io::stdin().read_line(&mut name);
        println!();
        println!();
        None
    } else {
        Some((
            rededup.len() as f64 / dedup as f64,
            rededup.len() as f64 / pure_size as f64,
            count_chuncks,
            analizer_preparation_time,
            cdc_chunk_add_time,
            dedup_time,
            rededup_time,
        ))
    }
}

fn pre_f(name: &str, all_sizes: Vec<usize>) {
    let path_string = "../test_files_input/".to_string() + name;
    let path = std::path::Path::new(path_string.as_str());
    let contents = fs::read(path).expect("Should have been able to read the file");

    let save_file_string = "./save_analizer_".to_string() + name + ".txt";
    let save_file_path = std::path::Path::new(save_file_string.as_str());

    if fs::exists(save_file_path).unwrap() {
        println!("file exist");
        return;
    }
    println!("Analize...");

    let mut analyser = FrequencyAnalyser::new_with_sizes(all_sizes);
    println!("{:?}", analyser.get_chunk_partitioning());
    analyser.append_dict(&contents);

    println!("Analize complete");
    for i in 1..40 {
        println!("{} {}", i, analyser.count_candidates(i))
    }
    println!("Reduce...");
    analyser.reduce_low_occur(2);
    println!("Save...");
    analyser.save_to_file(save_file_path).unwrap();
    println!("Save complete");
}

fn main() {
    const KB: usize = 1024;
    let names = [
        "linux-3.4.6-7.tar", // "fbc_topic_input.txt",
                             // "lowinput.txt",
                             // "orient_express_input.txt",
    ];
    let dts = [
        512,
        KB,
        2 * KB,
        4 * KB,
        6 * KB,
        8 * KB,
        10 * KB,
        12 * KB,
        16 * KB,
        20 * KB,
        24 * KB,
        30 * KB,
        // 2048,
        // 4096
        // KB
        // 6 * KB,
        // 8 * KB,
        // 10 * KB,
        // 12 * KB,
        // 16 * KB,
        // 32 * KB,
    ];

    let all_sizes: &[Vec<usize>] = &[
        // [64].to_vec(),
        // [128].to_vec(),
        [256].to_vec(),
        [512].to_vec(),
        [1024].to_vec(),
        [2048].to_vec(),
        [4096].to_vec(),
        [8192].to_vec(),
        // [256].to_vec(),
        // [1024, 512, 256].to_vec(),
        // [1024, 512, 256, 64].to_vec(),

        // [4096, 2048, 1024, 512].to_vec(),
    ];

    let all_sizes_flat = [
        8192, 4096, 2048, 1024, 512,
        256,
        // 128, 64
        // 512,
        // 1024,
        // 2048,
        // 4096,
    ]
    .to_vec();

    let mut str_out =
        String::from_str("file_name\tdt\tsizes\tdedup_coef\tpure_size_ratio\tcount_chunks\tanalizer_preparetion_time\tadd_cdc_chunck_time\tdedup_time\trededup_time\n")
            .unwrap();

    let mut file = fs::OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open("experement_result.csv")
        .unwrap();

    file.write_all(str_out.as_bytes()).unwrap();

    for name in names {
        pre_f(name, all_sizes_flat.clone());
        for sizes in all_sizes.iter() {
            for dt in dts {
                if dt < sizes[0] {
                    continue;
                }
                let mut this_out = String::new();
                this_out.push_str(name);
                this_out.push('\t');
                this_out.push_str(dt.to_string().as_str());
                this_out.push('\t');
                for s in sizes {
                    this_out.push_str(s.to_string().as_str());
                    this_out.push(' ');
                }
                this_out.push('\t');

                println!("{this_out}");
                match f(name, dt, sizes.clone()) {
                    Some(res) => {
                        this_out.push_str(res.0.to_string().as_str());
                        this_out.push('\t');
                        this_out.push_str(res.1.to_string().as_str());
                        this_out.push('\t');
                        this_out.push_str(res.2.to_string().as_str());
                        // analizer
                        this_out.push('\t');
                        this_out.push_str(res.3.as_secs_f64().to_string().as_str());
                        // add cdc
                        this_out.push('\t');
                        this_out.push_str(res.4.as_secs_f64().to_string().as_str());

                        // dedup
                        this_out.push('\t');
                        this_out.push_str(res.5.as_secs_f64().to_string().as_str());
                        // rededup
                        this_out.push('\t');
                        this_out.push_str(res.6.as_secs_f64().to_string().as_str());
                    }
                    None => {
                        this_out.push_str("NOT MATCH");
                    }
                }

                this_out.push('\n');

                println!("{this_out}");
                file.write_all(this_out.as_bytes()).unwrap();
                str_out.push_str(&this_out);
            }
        }
    }

    println!("{str_out}");
    for name in names {
        fs::remove_file("./save_analizer_".to_string() + name + ".txt").unwrap();
    }
}
