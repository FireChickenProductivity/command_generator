mod action_records;
mod action_utilities;
mod current_time;
mod data_output;
mod input_parsing;
mod monte_carlo_tree_search;
mod pool;
mod random;
mod recommendation_generation;
mod recommendation_scoring;
mod text_separation;

use action_records::{Argument, BasicAction, Command, Entry, read_file_record};
use current_time::compute_timestamp;
use data_output::{create_data_directory, output_recommendations};
use recommendation_generation::{
    ActionSet, PotentialCommandInformation, compute_recommendations_from_record, create_sorted_info,
};
use std::io;
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
    start: &Vec<usize>,
    number_of_recommendations: usize,
) -> Vec<recommendation_generation::CommandStatistics> {
    println!(
        "Finding the best {} recommendations.",
        number_of_recommendations
    );
    let start_time = Instant::now();
    let recommendations = recommendation_scoring::find_best(
        recommendations,
        start,
        number_of_recommendations as usize,
        false,
        false,
    );
    println!(
        "Time taken to find best recommendations: {:.3?}",
        start_time.elapsed()
    );
    recommendations
}

fn find_best_until_user_satisfied(
    mut recommendations: Vec<recommendation_generation::CommandStatistics>,
    number_of_recommendations: usize,
) -> Vec<recommendation_generation::CommandStatistics> {
    let mut start: Vec<usize> = Vec::new();
    let mut best = Vec::new();
    let mut to_keep = ActionSet::new();
    loop {
        best = find_best(recommendations.clone(), &start, number_of_recommendations);
        assert!(best.len() == number_of_recommendations);
        let mut to_remove = ActionSet::new();
        for recommendation in best.iter() {
            if !to_keep.contains(&recommendation.actions) {
                println!(
                    "\nType a command and press enter. y means keep the current command. ya means accept all commands. Anything else removes the current command.\n{}\n",
                    recommendation
                        .actions
                        .iter()
                        .map(|action| action.compute_talon_script())
                        .collect::<Vec<String>>()
                        .join("\n")
                );
                let mut input = String::new();
                let _result = io::stdin().read_line(&mut input);
                let actions = &recommendation.actions;
                match _result {
                    Ok(_) => {
                        let input_text = input.trim().to_lowercase();
                        if input_text == "y" {
                            to_keep.insert(&actions);
                        } else if input_text == "ya" {
                            return best;
                        } else {
                            to_remove.insert(&actions);
                        }
                    }
                    Err(_) => {
                        println!("Error reading input!");
                        to_remove.insert(&actions);
                    }
                }
            }
        }
        if to_remove.get_size() == 0 {
            return best;
        }
        start.clear();
        recommendations.retain(|r| !to_remove.contains(&r.actions));
        for (i, recommendation) in recommendations.iter().enumerate() {
            if to_keep.contains(&recommendation.actions) {
                start.push(i);
            }
        }
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
            let mut recommendations =
                compute_recommendations_from_record(record, parameters.max_chain_size);
            let elapsed_time = start_time.elapsed();
            println!(
                "Time taken to compute recommendations: {:.3?}",
                elapsed_time
            );
            println!("Created {} recommendations.", recommendations.len());
            if parameters.number_of_recommendations > 0 {
                recommendations =
                    recommendation_scoring::filter_out_recommendations_redundant_smaller_commands(
                        recommendations,
                    );
                println!(
                    "Narrowed it down to {} recommendations",
                    recommendations.len()
                );
                recommendations = find_best_until_user_satisfied(
                    recommendations,
                    parameters.number_of_recommendations,
                );
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
