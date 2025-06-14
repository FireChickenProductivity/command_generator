mod action_records;
mod action_utilities;
mod data_output;
mod input_parsing;
mod pool;
mod recommendation_generation;
mod text_separation;

use action_records::{Argument, BasicAction, Command, Entry, read_file_record};
use data_output::{create_data_directory, output_recommendations};
use recommendation_generation::{
    PotentialCommandInformation, compute_recommendations_from_record, create_sorted_info,
};
use std::time::Instant;

fn print_record(record: Result<Vec<Entry>, String>) {
    match record {
        Ok(record) => {
            for entry in record {
                match entry {
                    Entry::RecordingStart => println!("Recording started."),
                    Entry::Command(command) => {
                        println!("Command: {}", command.to_string());
                    }
                }
            }
        }
        Err(e) => println!("Error reading record file:\n	{}", e),
    }
}

fn main() {
    match create_data_directory() {
        Ok(_) => {}
        Err(e) => {
            println!("Error creating data directory: {}", e);
            return;
        }
    }
    let parameters = input_parsing::get_input_parameters_from_user();
    let start_time = Instant::now();
    println!("Reading file");
    let record = read_file_record(parameters.record_file);
    match record {
        Ok(record) => {
            println!("Generating recommendations");
            let recommendations =
                compute_recommendations_from_record(&record, parameters.max_chain_size);
            let elapsed_time = start_time.elapsed();
            let output = create_sorted_info(&recommendations);
            println!(
                "Time taken to compute recommendations: {:.3?}",
                elapsed_time
            );
            println!(
                "Created {} recommendations.",
                recommendations.concrete.len() + recommendations.abs.len()
            );
            return;
            output_recommendations(&output, "file.txt")
                .unwrap_or_else(|e| println!("Error writing recommendations to file: {}", e));
            println!("Recommendations written to file.");
        }
        Err(e) => println!("Error reading record file:\n	{}", e),
    }
}
