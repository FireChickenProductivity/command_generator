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

const REJECT_ACTION_PREFIX: &str = "r";
const PERSISTENTLY_REJECT_ACTION_PREFIX: &str = "ar";
const PERSISTENTLY_REJECT_COMMAND_PREFIX: &str = "arc";
const ACCEPT_RECOMMENDATION_COMMAND: &str = "y";
const ACCEPT_ALL_RECOMMENDATIONS_COMMAND: &str = "ya";

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
        "\nType a command and press enter. y means keep the current command. ya means accept all commands.\nr(action number here) removes all commands containing that action from future batches of recommendations.\nar(action number here) will reject any command containing that action in future uses of the program.\narc will reject the command during future uses of the program.\nAnything else removes the current command.\n{}\n",
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

fn find_action_to_remove<'a>(
    input_text: &str,
    recommendation: &'a recommendation_generation::CommandStatistics,
) -> Option<&'a action_records::BasicAction> {
    if input_text.len() < 1 {
        println!("Invalid input. Please enter a valid number.");
    }

    if let Ok(number) = input_text.parse::<usize>() {
        match recommendation.actions.get(number - 1) {
            Some(action) => {
                return Some(action);
            }
            None => {
                println!("Invalid action number. Please input one of the provided options.");
            }
        }
    } else {
        println!("Invalid action number. Please enter a valid number.");
    }
    None
}

fn update_to_remove_containing(
    input_text: &str,
    recommendation: &recommendation_generation::CommandStatistics,
    to_remove_containing: &mut ActionSet,
) {
    let possible_action = find_action_to_remove(input_text, recommendation);
    if let Some(action) = possible_action {
        to_remove_containing.insert_action(&action);
    }
}

fn persistently_reject_action(
    input_text: &str,
    recommendation: &recommendation_generation::CommandStatistics,
    to_remove_containing: &mut ActionSet,
    to_persistently_reject_containing: &mut Vec<action_records::BasicAction>,
) {
    let possible_action = find_action_to_remove(input_text, recommendation);
    if let Some(action) = possible_action {
        to_persistently_reject_containing.push(action.clone());
        to_remove_containing.insert_action(&action);
    }
}

struct UserCommand {
    encountered_no: bool,
    encountered_yes: bool,
    encountered_reject_command_persistently: bool,
    action_number_to_reject: Option<usize>,
    action_number_reject_persistently: Option<usize>,
    encountered_accept_the_rest_of_the_commands: bool,
}

impl UserCommand {
    fn new(input_text: String) -> Result<UserCommand, String> {
        if input_text.is_empty() {
            return Ok(UserCommand {
                encountered_no: true,
                encountered_yes: false,
                encountered_reject_command_persistently: false,
                action_number_to_reject: None,
                action_number_reject_persistently: None,
                encountered_accept_the_rest_of_the_commands: false,
            });
        }
        let white_space_separated_tokens: Vec<&str> = input_text.split_whitespace().collect();
        if white_space_separated_tokens.len() > 2 {
            return Err(format!(
                "A valid command would only have one or fewer spaces. You entered: {}",
                input_text
            ));
        }
        let mut encountered_no = false;
        let mut encountered_yes = false;
        let mut encountered_reject_command_persistently = false;
        let mut action_number_to_reject = None;
        let mut action_number_reject_persistently = None;
        let mut expecting_action_number_to_reject = false;
        let mut expecting_action_number_reject_persistently = false;
        let mut encountered_accept_the_rest_of_the_commands = false;
        let mut invalid_character = None;
        white_space_separated_tokens[0]
            .chars()
            .for_each(|c| match c {
                'y' => {
                    encountered_yes = true;
                }
                'n' => {
                    encountered_no = true;
                }
                'r' => {
                    expecting_action_number_reject_persistently = true;
                }
                'd' => {
                    expecting_action_number_to_reject = true;
                }
                'c' => {
                    encountered_reject_command_persistently = true;
                }
                'a' => {
                    encountered_accept_the_rest_of_the_commands = true;
                }
                _ => {
                    invalid_character = Some(c);
                }
            });
        if invalid_character.is_some() {
            return Err(format!(
                "Received invalid command character: {}.",
                invalid_character.unwrap()
            ));
        }
        if encountered_no && encountered_yes {
            return Err("You cannot accept and reject a command at the same time.".to_string());
        }
        if expecting_action_number_to_reject && white_space_separated_tokens.len() < 2 {
            return Err("You need to provide an action number to reject.".to_string());
        }
        if expecting_action_number_reject_persistently && white_space_separated_tokens.len() < 2 {
            return Err("You need to provide an action number to reject persistently.".to_string());
        }
        if expecting_action_number_reject_persistently && encountered_reject_command_persistently {
            match white_space_separated_tokens[1].parse::<usize>() {
                Ok(action_number) => {
                    if expecting_action_number_reject_persistently {
                        action_number_reject_persistently = Some(action_number);
                    }
                    if expecting_action_number_to_reject {
                        action_number_to_reject = Some(action_number);
                    }
                }
                Err(_) => {
                    return Err(format!(
                        "Invalid action number: {}",
                        white_space_separated_tokens[1]
                    ));
                }
            }
        }
        return Ok(Self {
            encountered_no,
            encountered_yes,
            encountered_reject_command_persistently,
            action_number_to_reject,
            action_number_reject_persistently,
            encountered_accept_the_rest_of_the_commands,
        });
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

    loop {
        let best = find_best(recommendations.clone(), &start, number_of_recommendations);
        let mut to_remove = ActionSet::new();
        let mut to_remove_containing = ActionSet::new();
        for recommendation in best.iter() {
            if !to_keep.contains(&recommendation.actions) {
                let mut done = false;
                while !done {
                    let input_text = prompt_user_about_recommendation(recommendation);
                    // let user_command = match UserCommand::new(input_text) {
                    //     Ok(command) => command,
                    //     Err(e) => {
                    //         println!("{}", e);
                    //         continue;
                    //     }
                    // };

                    if input_text == ACCEPT_RECOMMENDATION_COMMAND {
                        to_keep.insert(&recommendation.actions);
                        done = true;
                    } else if input_text == ACCEPT_ALL_RECOMMENDATIONS_COMMAND {
                        return best;
                    } else if input_text == PERSISTENTLY_REJECT_COMMAND_PREFIX {
                        to_persistently_reject_commands.push(recommendation.actions.clone());
                    } else if input_text.starts_with(REJECT_ACTION_PREFIX) {
                        update_to_remove_containing(
                            &input_text.strip_prefix(REJECT_ACTION_PREFIX).unwrap_or(""),
                            recommendation,
                            &mut to_remove_containing,
                        );
                    } else if input_text.starts_with(PERSISTENTLY_REJECT_ACTION_PREFIX) {
                        persistently_reject_action(
                            &input_text
                                .strip_prefix(PERSISTENTLY_REJECT_ACTION_PREFIX)
                                .unwrap_or(""),
                            recommendation,
                            &mut to_remove_containing,
                            to_persistently_reject_containing,
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
