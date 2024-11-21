// This does not work yet

use std::{io::Write, process::{Command, ExitStatus}};

use indicatif::{ProgressBar, ProgressStyle};
use strsim::hamming;

use super::{mesh::Mesh, sequencer::{generate_sequence, MotifStorage}};

pub fn run_fold(sequence: &str) {
    // Write sequence to file for RNAFold input
    let mut file: std::fs::File = std::fs::File::create("output/sequence.fa").unwrap();
    file.write(sequence.as_bytes()).unwrap();

    // Wait a bit to ensure contents are written
    //sleep(std::time::Duration::from_millis(10));

    // Run RNAFold
    log::debug!("Running RNAFold");
    let status: ExitStatus = if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(["/C", "rnafold -p -d2 --noLP --circ −−log−level=4 -i output/output.txt > output/sequence.out"])
            .status()
            .expect("failed to execute process")
    } else {
        Command::new("sh")
            .arg("-c")
            .arg("echo 'running RNAFold'")
            .arg(";")
            .arg("rnafold -p -d2 --noLP --circ -i ./output/sequence.fa > ./output/sequence.out")
            .status()
            .expect("failed to execute process")
    };

    let mut is_status_success: bool = false;
    while !is_status_success {
        is_status_success = status.success();
    }
}

#[inline(always)]
pub fn fetch_brackets() -> String {
    let output_brackets: String = std::fs::read_to_string("output/sequence.out").unwrap();
    let output: &str = output_brackets.lines().nth(1).unwrap().split_whitespace().nth(0).unwrap();
    output.to_string()
}

#[inline(always)]
pub fn compare_brackets(target: &str) -> f64 {
    let output: String = fetch_brackets();
    let similarity: f64 = 100.0 * (1.0 - (hamming(target, &output).unwrap() as f64 / target.len() as f64));
    //log::info!("Similarity: {similarity:.2}%");
    similarity
}

#[inline(always)]
pub fn similarity_test(
    runs: usize,
    mesh: &Mesh,
    path: &[(bool, usize)],
    motifs: &MotifStorage
) -> (f64, ([String;2], f64)) {
    // Output them
    let mut mean_similarity: f64 = 0.0;
    let mut item_count: f64 = 0.0;
    let mut best_sequence: String = String::new();
    let mut best_sequence_db: String = String::new();
    let mut highest_similarity: f64 = 0.0;

    let pb: ProgressBar = ProgressBar::new(runs as u64);
    pb.set_style(
        ProgressStyle::with_template("{spinner:.green} [{wide_bar:.cyan/blue}] {pos}/{len} | {msg}").unwrap()
    );

    (0..runs).for_each(|_| {
        // Generate sequence
        let [db_sequence, nt_sequence]: [String; 2] = generate_sequence(mesh, path, motifs);
        log::debug!("Sequence:\n {nt_sequence}");
        log::debug!("Brackets:\n {db_sequence}");  

        let mut file = std::fs::File::create("output/output.txt").unwrap();
        writeln!(file, "{nt_sequence}").unwrap();

        run_fold(&nt_sequence);
        let similarity: f64 = compare_brackets(&db_sequence);

        if similarity > highest_similarity {
            highest_similarity = similarity;
            best_sequence = nt_sequence;
            best_sequence_db = fetch_brackets();
        }

        // Average precisions
        item_count += 1.0;
        mean_similarity += (similarity - mean_similarity) / item_count;

        log::debug!("Mean similarity: {mean_similarity:.2}%");

        // Update progress bar
        pb.set_message(format!("Mean similarity: {similarity:.2}%"));
        pb.set_position(item_count as u64);
    });
    pb.finish_with_message("done");
    (mean_similarity, ([best_sequence, best_sequence_db], highest_similarity))
}

pub fn run_until_successful(
    mesh: &Mesh, 
    path: &[(bool, usize)], 
    motifs: &MotifStorage,
    similarity_threshold: f64,
) {
    let mut is_same: bool = false;
    let mut attempt: usize = 1;

    let pb: ProgressBar = ProgressBar::new(20);
    pb.set_style(
        ProgressStyle::with_template("{spinner:.green} | {msg}").unwrap()
    );
    let mut similarity: f64 = 0.0;
    let mut final_sequence: String = String::new();

    while !is_same {
        //log::info!("Sequence attempt {attempt}");
        let [db_sequence, sequence]: [String; 2] = generate_sequence(mesh, path, motifs);
        run_fold(&sequence);
        let local_similarity: f64 = compare_brackets(&db_sequence);
        is_same = local_similarity >= similarity_threshold;
        attempt += 1;
        pb.set_message(format!("Attempt {attempt} | Similarity: {local_similarity:.2}%"));
        if is_same {
            final_sequence = sequence;
            similarity = local_similarity;
        }
    }

    pb.finish_with_message(
        format!("Viable structure found after {attempt} attempts. Similarity = {similarity:.2}")
    );

    let output: String = fetch_brackets();

    log::info!("Sequence: \n{final_sequence}\n");
    log::info!("Structure: \n{output}\n");
}

pub fn clean_output() {
    let sequence_out: String = std::fs::read_to_string("output/sequence.out").unwrap();
    let sequence = sequence_out.lines().nth(0).unwrap();
    let dots_brackets = sequence_out.lines().nth(1).unwrap().split_whitespace().nth(0).unwrap();

    std::fs::write("output/output.txt", format!("{sequence}\n{dots_brackets}")).unwrap();
}