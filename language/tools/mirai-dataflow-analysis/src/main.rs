// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use mirai_annotations::unrecoverable;
use regex::Regex;
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    process::Command,
};
use structopt::StructOpt;

mod configuration;
mod datalog;
mod ddlog_output;
mod souffle_output;
mod util;

use configuration::generate_config;
use datalog::{decode_analysis_output, DatalogBackend, DatalogRelations};
use ddlog_output::{parse_ddlog_output, run_ddlog_analysis};
use souffle_output::{parse_souffle_output, run_souffle_analysis};
use util::{get_child_output, NodeMap, TypeMap};

// Path to the current Rust nightly toolchain version
// that MIRAI is using
const RUST_TOOLCHAIN_PATH: &str = "./rust-toolchain";

// Run the Datalog analysis on the test file and
// capture output
fn run_mirai(config_path: &Path, crate_path: &Path, no_clean: bool) -> Result<(), String> {
    if !no_clean {
        // cargo clean
        println!("Executing cargo clean on {}", crate_path.display());
        let out1 = Command::new("cargo")
            .current_dir(&crate_path)
            .arg("clean")
            .output()
            .map_err(|msg| format!("Failed to clean crate: {}", msg))?;
        println!("Result: {}", get_child_output(&out1));
        // Set nightly
        println!(
            "Setting nightly toolchain version for {}",
            crate_path.display()
        );
        let nightly_str = fs::read_to_string(Path::new(RUST_TOOLCHAIN_PATH))
            .map_err(|msg| format!("Failed to toolchain nightly version: {}", msg))
            .map(|nightly_str| nightly_str.replace('\n', ""))?;
        let out2 = Command::new("rustup")
            .current_dir(&crate_path)
            .arg("override")
            .arg("set")
            .arg(nightly_str)
            .output()
            .map_err(|msg| format!("Failed to set nightly: {}", msg))?;
        println!("Result: {}", get_child_output(&out2));
        // cargo build
        println!("Executing cargo build on {}", crate_path.display());
        let out3 = Command::new("cargo")
            .current_dir(&crate_path)
            .arg("build")
            .output()
            .map_err(|msg| format!("Failed to clean crate: {}", msg))?;
        println!("Result: {}", get_child_output(&out3));
    }
    // 'RUSTFLAGS="-Z always_encode_mir" cargo build'
    println!("Executing MIR cargo build on {}", crate_path.display());
    let out4 = Command::new("cargo")
        .current_dir(&crate_path)
        .env("RUSTFLAGS", "-Z always_encode_mir")
        .arg("build")
        .output()
        .map_err(|msg| format!("Failed to build crate with MIR: {}", msg))?;
    println!("Result: {}", get_child_output(&out4));
    // touch src/lib.rs
    println!("Executing touch src/lib.rs {}", crate_path.display());
    let out5 = Command::new("touch")
        .current_dir(&crate_path)
        .arg("src/lib.rs")
        .output()
        .map_err(|msg| format!("Failed to execute touch: {}", msg))?;
    println!("Result: {}", get_child_output(&out5));
    /*
        RUSTFLAGS="-Z always_encode_mir"
        RUSTC_WRAPPER=mirai
        MIRAI_FLAGS="--call_graph_config=$CGG_PATH"
        MIRAI_LOG=info
        cargo build
    */
    println!("Executing MIRAI on {}", crate_path.display());
    let out6 = Command::new("cargo")
        .current_dir(&crate_path)
        .env("RUSTFLAGS", "-Z always_encode_mir")
        .env("RUSTC_WRAPPER", "mirai")
        .env(
            "MIRAI_FLAGS",
            format!("--call_graph_config={}", config_path.to_str().unwrap()),
        )
        .env("MIRAI_LOG", "info")
        .arg("build")
        .output()
        .map(|output| get_child_output(&output))
        .map_err(|msg| format!("Failed to execute MIRAI: {}", msg))?;
    println!("Result: {}", out6);
    Ok(())
}

/// Read in a mapping from type indexes to type strings
fn parse_type_map(type_map_path: &Path) -> Result<TypeMap, String> {
    fs::read_to_string(type_map_path)
        .map_err(|msg| format!("Failed to parse type map: {}", msg))
        .and_then(|out| {
            serde_json::from_str::<HashMap<u32, String>>(&out)
                .map_err(|msg| format!("Failed to parse type map: {}", msg))
        })
}

/// Generate a mapping from node indexes to node names
fn parse_dot_graph(graph_path: &Path) -> Result<NodeMap, String> {
    fs::read_to_string(graph_path)
        .map_err(|msg| format!("Failed to parse graph: {}", msg))
        .map(|out| {
            let mut node_map = HashMap::<u32, String>::new();
            let lines = out.split('\n').collect::<Vec<&str>>();
            for line in lines.iter() {
                if line.contains("label = ") {
                    if let Some(captures) =
                        Regex::new(r#"(\d+) \[.*"(.+)\\"#).unwrap().captures(line)
                    {
                        assert!(captures.len() == 3);
                        let node_id = captures[1].to_owned().parse::<u32>().unwrap();
                        let node_name = captures[2].to_owned();
                        assert_eq!(node_map.insert(node_id, node_name), None);
                    }
                }
            }
            node_map
        })
}

/// Process raw analysis output and write it to a file
fn process_datalog_output(
    relations: &mut DatalogRelations,
    type_map_path: &Path,
    dot_path: &Path,
    decoded_path: &Path,
) -> Result<(), String> {
    let type_map = parse_type_map(type_map_path)?;
    let node_map = parse_dot_graph(dot_path)?;
    decode_analysis_output(relations, type_map, node_map);
    let relations_output = serde_json::to_string(&relations)
        .map_err(|msg| format!("Failed to serialize relations: {}", msg))?;
    fs::write(decoded_path, relations_output)
        .map(|_| ())
        .map_err(|msg| format!("Failed to write relations to file: {}", msg))
}

#[derive(Debug, StructOpt)]
#[structopt(
    name = "mirai-dataflow",
    about = "Rust dataflow analyzer built on MIRAI."
)]
struct Opt {
    /// Path to the crate to analyze
    #[structopt(parse(from_os_str))]
    crate_path: PathBuf,

    /// Path to configuration file
    #[structopt(parse(from_os_str))]
    config_path: PathBuf,

    /// Only produce a call graph (no analysis)
    #[structopt(long)]
    call_graph_only: bool,

    /// Datalog backend to use (DifferentialDatalog | Souffle)
    #[structopt(short, long)]
    datalog_backend: Option<DatalogBackend>,

    /// Path to input type relations
    #[structopt(short, long, parse(from_os_str))]
    type_relations_path: Option<PathBuf>,

    /// Do not clean the crate before analysis
    #[structopt(short, long)]
    no_clean: bool,

    /// Rerun the Datalog analysis without running MIRAI
    #[structopt(short, long)]
    reanalyze: bool,
}

fn main() {
    // Process command line arguments
    let opt = Opt::from_args();
    // Canonicalize the crate path
    let crate_path = match fs::canonicalize(opt.crate_path) {
        Ok(crate_path) => crate_path,
        Err(msg) => unrecoverable!("Failed to find crate to analyze: {}", msg),
    };
    // Generate the call graph config
    let (config, new_config_path) = match generate_config(
        &opt.config_path,
        opt.call_graph_only,
        opt.datalog_backend,
        opt.type_relations_path,
    ) {
        Ok(config) => config,
        Err(msg) => unrecoverable!("Failed to generate analysis configuration: {}", msg),
    };
    // Run MIRAI over the crate, producing call graph output
    let config_path = match fs::canonicalize(new_config_path) {
        Ok(crate_path) => crate_path,
        Err(msg) => unrecoverable!("Failed to find call graph config: {}", msg),
    };
    if !opt.reanalyze {
        println!("Running MIRAI...");
        match run_mirai(&config_path, &crate_path, opt.no_clean) {
            Ok(_) => {}
            Err(msg) => unrecoverable!("{}", msg),
        }
        println!("Done");
    }
    // If configured, analyze the call graph output with our
    // Datalog analysis
    if let Some(datalog_config) = config.datalog_config {
        println!("Running Datalog analysis...");
        let analysis_result = match datalog_config.datalog_backend {
            DatalogBackend::DifferentialDatalog => run_ddlog_analysis(
                &datalog_config.ddlog_output_path,
                &datalog_config.analysis_raw_output_path,
            ),
            DatalogBackend::Souffle => {
                run_souffle_analysis(&datalog_config.analysis_raw_output_path)
            }
        };
        match analysis_result {
            Ok(_) => {}
            Err(msg) => unrecoverable!("{}", msg),
        }
        println!("Done");
        // Process the raw analysis output into a decoded form
        println!("Processing output...");
        let relations_result = match datalog_config.datalog_backend {
            DatalogBackend::DifferentialDatalog => {
                parse_ddlog_output(&datalog_config.ddlog_output_path)
            }
            DatalogBackend::Souffle => {
                parse_souffle_output(&datalog_config.analysis_raw_output_path)
            }
        };
        let mut relations = match relations_result {
            Ok(relations) => relations,
            Err(msg) => unrecoverable!("{}", msg),
        };
        assert!(config.dot_output_path.is_some());
        let process_result = process_datalog_output(
            &mut relations,
            &datalog_config.type_map_output_path,
            &config.dot_output_path.unwrap(),
            &datalog_config.analysis_decoded_output_path,
        );
        match process_result {
            Ok(_) => {}
            Err(msg) => unrecoverable!("{}", msg),
        }
        println!("Done");
    }
}
