mod action_records;
mod action_utilities;
mod current_time;
mod data_output;
mod input_parsing;
mod pool;
mod recommendation_generation;
mod recommendation_scoring;
mod text_separation;

use action_records::{Argument, BasicAction, Command, Entry, read_file_record};
use current_time::compute_timestamp;
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

fn find_best(
    recommendations: Vec<recommendation_generation::CommandStatistics>,
    number_of_recommendations: usize,
) -> Vec<recommendation_generation::CommandStatistics> {
    println!(
        "Finding the best {} recommendations.",
        number_of_recommendations
    );
    let start_time = Instant::now();
    let recommendations =
        recommendation_scoring::find_best(recommendations, number_of_recommendations as usize);
    println!(
        "Time taken to find best recommendations: {:.3?}",
        start_time.elapsed()
    );
    recommendations
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
            let mut recommendations =
                compute_recommendations_from_record(record, parameters.max_chain_size);
            let elapsed_time = start_time.elapsed();
            println!(
                "Time taken to compute recommendations: {:.3?}",
                elapsed_time
            );
            println!("Created {} recommendations.", recommendations.len());
            if parameters.number_of_recommendations > 0 {
                recommendations = find_best(recommendations, parameters.number_of_recommendations);
            }
            create_sorted_info(&mut recommendations);
            let file_name = format!("recommendations {}.txt", compute_timestamp());
            output_recommendations(&recommendations, &file_name)
                .unwrap_or_else(|e| println!("Error writing recommendations to file: {}", e));
            println!("Recommendations written to file.");
        }
        Err(e) => println!("Error reading record file:\n	{}", e),
    }
}
