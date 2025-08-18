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
mod user_command_parsing;

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
        "\nType a command and press enter. y means keep the current command.\nn or pressing enter without typing anything means reject the current command.\nc means do not provide this recommendation again.\nd followed by a space and a number means do not recommend the action with that number again during this session.\nr followed by a space and a number means do not recommend the action of that number again.\na means accept all commands\nmultiple commands can be provided at the same time by combining letters.\n{}\n",
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

fn find_actions_to_remove<'a>(
    input_numbers: &Vec<usize>,
    recommendation: &'a recommendation_generation::CommandStatistics,
) -> Vec<&'a action_records::BasicAction> {
    let mut actions_to_remove = Vec::new();
    for number in input_numbers {
        if let Some(action) = recommendation.actions.get(*number - 1) {
            actions_to_remove.push(action);
        } else {
            println!("Invalid action number. Please input one of the provided options.",);
            return Vec::new();
        }
    }
    actions_to_remove
}

fn update_to_remove_containing(
    input_numbers: &Vec<usize>,
    recommendation: &recommendation_generation::CommandStatistics,
    to_remove_containing: &mut ActionSet,
) {
    let possible_actions = find_actions_to_remove(input_numbers, recommendation);
    if !possible_actions.is_empty() {
        for action in possible_actions {
            to_remove_containing.insert_action(action);
        }
    }
}

fn persistently_reject_actions(
    input_numbers: &Vec<usize>,
    recommendation: &recommendation_generation::CommandStatistics,
    to_remove_containing: &mut ActionSet,
    to_persistently_reject_containing: &mut Vec<action_records::BasicAction>,
) {
    let possible_actions = find_actions_to_remove(input_numbers, recommendation);
    if !possible_actions.is_empty() {
        for action in possible_actions {
            to_persistently_reject_containing.push(action.clone());
            to_remove_containing.insert_action(action);
        }
    }
}

fn find_best_until_user_satisfied(
    mut recommendations: Vec<recommendation_generation::CommandStatistics>,
    number_of_recommendations: usize,
    to_persistently_reject_containing: &mut Vec<action_records::BasicAction>,
    to_persistently_reject_commands: &mut Vec<Vec<action_records::BasicAction>>,
) -> Vec<recommendation_generation::CommandStatistics> {
    let mut start: Vec<usize> = Vec::new();
    let mut to_keep = ActionSet::new();
    let mut should_keep_everything_else = false;

    loop {
        let best = find_best(recommendations.clone(), &start, number_of_recommendations);
        let mut to_remove = ActionSet::new();
        let mut to_remove_containing = ActionSet::new();
        for recommendation in best.iter() {
            if should_keep_everything_else {
                to_keep.insert(&recommendation.actions);
            } else if !to_keep.contains(&recommendation.actions) {
                let mut done = false;
                while !done {
                    let input_text = prompt_user_about_recommendation(recommendation);
                    let user_command = match user_command_parsing::UserCommand::new(input_text) {
                        Ok(command) => command,
                        Err(e) => {
                            println!("{}", e);
                            continue;
                        }
                    };
                    if user_command.encountered_yes {
                        to_keep.insert(&recommendation.actions);
                    } else if user_command.encountered_no {
                        to_remove.insert(&recommendation.actions);
                    }
                    if user_command.encountered_reject_command_persistently {
                        to_persistently_reject_commands.push(recommendation.actions.clone());
                    }
                    if !user_command.action_numbers_to_reject.is_empty() {
                        update_to_remove_containing(
                            &user_command.action_numbers_to_reject,
                            recommendation,
                            &mut to_remove_containing,
                        );
                    }
                    if !user_command
                        .action_numbers_to_reject_persistently
                        .is_empty()
                    {
                        persistently_reject_actions(
                            &user_command.action_numbers_to_reject_persistently,
                            recommendation,
                            &mut to_remove_containing,
                            to_persistently_reject_containing,
                        );
                    }
                    if user_command.encountered_accept_the_rest_of_the_commands {
                        should_keep_everything_else = true;
                    }
                    if user_command.action_numbers_to_reject.is_empty()
                        && user_command
                            .action_numbers_to_reject_persistently
                            .is_empty()
                    {
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

fn initialize_directories() -> Result<(), std::io::Error> {
    create_data_directory()?;
    match configuration::create_configuration_directory() {
        Ok(_) => {}
        Err(e) => {
            println!("Error creating configuration directory: {}", e);
        }
    }
    Ok(())
}

fn filter_recommendations(recommendations: &mut Vec<recommendation_generation::CommandStatistics>) {
    let actions_to_reject = configuration::get_actions_to_reject();
    if actions_to_reject.get_size() > 0 {
        recommendation_filtering::filter_out_recommendations_containing_actions(
            recommendations,
            &actions_to_reject,
        );
        println!(
            "{} recommendations after remaining filtering out rejected actions",
            recommendations.len()
        );
    }
    let commands_to_reject = configuration::get_commands_to_reject();
    if commands_to_reject.get_size() > 0 {
        recommendation_filtering::filter_out_recommendations(recommendations, |recommendation| {
            commands_to_reject.contains(&recommendation.actions)
        });
        println!(
            "{} recommendations after remaining filtering out rejected commands",
            recommendations.len()
        );
    }
}

fn create_initial_recommendations(
    record: Vec<action_records::Entry>,
    parameters: &input_parsing::InputParameters,
    start_time: Instant,
) -> Vec<recommendation_generation::CommandStatistics> {
    println!("Generating recommendations");
    let recommendations = compute_recommendations_from_record(record, parameters.max_chain_size);
    let elapsed_time = start_time.elapsed();
    println!(
        "Time taken to compute recommendations: {:.3?}",
        elapsed_time
    );
    println!("Created {} recommendations.", recommendations.len());
    recommendations
}

fn let_user_run_commands_on_recommendations(
    recommendations: Vec<recommendation_generation::CommandStatistics>,
    parameters: &input_parsing::InputParameters,
) -> Vec<recommendation_generation::CommandStatistics> {
    let mut recommendations =
        recommendation_scoring::filter_out_recommendations_redundant_smaller_commands(
            recommendations,
        );
    println!(
        "Narrowed it down to {} recommendations",
        recommendations.len()
    );
    let mut to_persistently_reject_containing: Vec<action_records::BasicAction> = Vec::new();
    let mut commands_to_persistently_reject = Vec::new();
    recommendations = find_best_until_user_satisfied(
        recommendations,
        parameters.number_of_recommendations,
        &mut to_persistently_reject_containing,
        &mut commands_to_persistently_reject,
    );
    configuration::append_actions_to_reject(&to_persistently_reject_containing);
    configuration::append_commands_to_reject(&commands_to_persistently_reject);
    recommendations
}

fn create_user_recommendations(
    record: Vec<action_records::Entry>,
    parameters: &input_parsing::InputParameters,
    start_time: Instant,
) {
    if record.is_empty() {
        println!("No actions in the record. Exiting.");
        return;
    }
    let mut recommendations = create_initial_recommendations(record, parameters, start_time);
    filter_recommendations(&mut recommendations);

    if parameters.number_of_recommendations > 0 {
        recommendations = let_user_run_commands_on_recommendations(recommendations, parameters);
    }

    create_sorted_info(&mut recommendations);
    let file_name = format!("recommendations {}.txt", compute_timestamp());
    output_recommendations(&recommendations, &file_name)
        .unwrap_or_else(|e| println!("Error writing recommendations to file: {}", e));
    println!("Recommendations written to file.");
}

fn main() {
    match initialize_directories() {
        Ok(_) => {}
        Err(e) => {
            println!("Directory creation error: {}", e);
            return;
        }
    }

    let (record_file, parameters) = input_parsing::get_input_parameters_from_user();
    let start_time = Instant::now();
    println!("Reading file");
    let record = read_file_record(record_file);
    match record {
        Ok(record) => {
            create_user_recommendations(record, &parameters, start_time);
        }
        Err(e) => println!("Error reading record file:\n	{}", e),
    }
}
