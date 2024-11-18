use std::{io::Write, process::{Command, ExitStatus}, thread::sleep};

use strsim::hamming;

use super::{mesh::Mesh, sequencer::{generate_sequence, MotifStorage}};

pub fn run_fold(sequence: &str) {
    // Write sequence to file for RNAFold input
    let mut file: std::fs::File = std::fs::File::create("output/sequence.fa").unwrap();
    file.write(sequence.as_bytes()).unwrap();

    // Wait a bit to ensure contents are written
    sleep(std::time::Duration::from_millis(10));

    // Run RNAFold
    log::debug!("Running RNAFold");
    let status: ExitStatus = if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(["/C", "rnafold -p -d2 --noLP --circ -i output/sequence.fa > output/sequence.out"])
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

pub fn fetch_brackets() -> String {
    let output_brackets: String = std::fs::read_to_string("output/sequence.out").unwrap();
    let output: &str = output_brackets.lines().nth(1).unwrap().split_whitespace().nth(0).unwrap();
    output.to_string()
}

pub fn compare_brackets(target: &str, similarity_threshold: f64) -> bool {
    let output: String = fetch_brackets();
    let similarity: f64 = 1.0 - (hamming(target, &output).unwrap() as f64 / 100.0);
    similarity >= similarity_threshold
}

pub fn run_until_successful(
    mesh: &Mesh, 
    path: &[(bool, usize)], 
    motifs: &MotifStorage,
    target: &str,
    similarity_threshold: f64,
) {
    let mut is_same: bool = false;
    let mut attempt: usize = 1;
    while !is_same {
        log::info!("Sequence attempt {attempt}");
        let sequence: String = generate_sequence(mesh, path, motifs);
        run_fold(&sequence);
        is_same = compare_brackets(target, similarity_threshold);
        attempt += 1;
    }

    let output: String = fetch_brackets();

    log::info!("Target: \n{target}\n");
    log::info!("Output: \n{output}\n");
}
