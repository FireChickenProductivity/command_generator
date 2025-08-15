mod action_records;
mod action_utilities;
mod configuration;
mod current_time;
mod data_output;
mod input_parsing;
mod monte_carlo_tree_search;
mod paths;
mod pool;
mod random;
mod recommendation_filtering;
mod recommendation_generation;
mod recommendation_scoring;
mod text_separation;

use action_records::read_file_record;
use current_time::compute_timestamp;
use data_output::{create_data_directory, output_recommendations};
use recommendation_generation::{
    ActionSet, compute_recommendations_from_record, create_sorted_info,
};
use std::io;
use std::time::Instant;

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

fn prompt_user_about_recommendation(
    recommendation: &recommendation_generation::CommandStatistics,
) -> String {
    println!(
        "\nType a command and press enter. y means keep the current command. ya means accept all commands.\nr(action number here) removes all commands containing that action from future batches of recommendations.\nAnything else removes the current command.\n{}\n",
        recommendation
            .actions
            .iter()
            .enumerate()
            .map(|(index, action)| format!("{}. {}", index + 1, action.compute_talon_script()))
            .collect::<Vec<String>>()
            .join("\n")
    );
    loop {
        let mut input = String::new();
        let _result = io::stdin().read_line(&mut input);
        match _result {
            Ok(_) => {
                return input.trim().to_lowercase();
            }
            Err(_) => {
                println!("Error reading input! Please try again.");
            }
        }
    }
}

fn perform_removals(
    start: &mut Vec<usize>,
    recommendations: &mut Vec<recommendation_generation::CommandStatistics>,
    to_keep: &ActionSet,
    to_remove: &ActionSet,
    to_remove_containing: ActionSet,
) {
    start.clear();
    recommendations.retain(|r| {
        to_keep.contains(&r.actions)
            || (!to_remove.contains(&r.actions)
                && !r
                    .actions
                    .iter()
                    .any(|action| to_remove_containing.contains_action(action)))
    });
    println!(
        "Narrowed it down to {} recommendations",
        recommendations.len()
    );
    for (i, recommendation) in recommendations.iter().enumerate() {
        if to_keep.contains(&recommendation.actions) {
            start.push(i);
        }
    }
}

fn update_to_remove_containing(
    input_text: &str,
    recommendation: &recommendation_generation::CommandStatistics,
    to_remove_containing: &mut ActionSet,
) {
    if input_text.len() < 2 {
        println!("Invalid input. Please enter a valid number.");
    }

    if let Ok(number) = input_text[1..].parse::<usize>() {
        if number <= recommendation.actions.len() {
            let action_to_remove = &recommendation.actions[number - 1];
            to_remove_containing.insert_action(action_to_remove);
        } else {
            println!("Invalid action number. Please input one of the provided options.");
        }
    } else {
        println!("Invalid action number. Please enter a valid number.");
    }
}

fn find_best_until_user_satisfied(
    mut recommendations: Vec<recommendation_generation::CommandStatistics>,
    number_of_recommendations: usize,
) -> Vec<recommendation_generation::CommandStatistics> {
    let mut start: Vec<usize> = Vec::new();
    let mut to_keep = ActionSet::new();
    loop {
        let best = find_best(recommendations.clone(), &start, number_of_recommendations);
        let mut to_remove = ActionSet::new();
        let mut to_remove_containing = ActionSet::new();
        for recommendation in best.iter() {
            if !to_keep.contains(&recommendation.actions) {
                let mut done = false;
                while !done {
                    let input_text = prompt_user_about_recommendation(recommendation);
                    if input_text == "y" {
                        to_keep.insert(&recommendation.actions);
                        done = true;
                    } else if input_text == "ya" {
                        return best;
                    } else if input_text.starts_with("r") {
                        update_to_remove_containing(
                            &input_text,
                            recommendation,
                            &mut to_remove_containing,
                        );
                    } else {
                        to_remove.insert(&recommendation.actions);
                        done = true;
                    }
                }
            }
        }
        if to_remove.get_size() == 0 {
            return best;
        }
        perform_removals(
            &mut start,
            &mut recommendations,
            &to_keep,
            &to_remove,
            to_remove_containing,
        );
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
    match configuration::create_configuration_directory() {
        Ok(_) => {}
        Err(e) => {
            println!("Error creating configuration directory: {}", e);
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
            let actions_to_reject = configuration::get_actions_to_reject();
            if actions_to_reject.get_size() > 0 {
                recommendation_filtering::filter_out_recommendations_containing_actions(
                    &mut recommendations,
                    &actions_to_reject,
                );
                println!(
                    "{} recommendations after remaining filtering out rejected actions",
                    recommendations.len()
                );
            }
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
