use crate::action_records::BasicAction;
use crate::action_utilities::*;
use crate::pool;
use crate::recommendation_generation::{
    CommandStatistics, compute_string_representation_of_actions,
};
use std::sync::{Arc, RwLock};
use std::{collections::HashMap, collections::HashSet};

fn compute_number_of_commands_including_action(
    recommendations: &Vec<CommandStatistics>,
) -> HashMap<String, usize> {
    let mut result = HashMap::new();
    for recommendation in recommendations {
        let mut unique_actions = HashSet::new();
        for action in &recommendation.actions {
            let action_string = action.to_json();
            unique_actions.insert(action_string);
        }
        for unique_action in unique_actions {
            let count = result.entry(unique_action).or_insert(0);
            *count += 1;
        }
    }
    result
}

fn compute_single_inserts_from_commands(
    recommendations: &Vec<CommandStatistics>,
) -> HashSet<String> {
    let mut single_inserts = HashSet::new();
    for recommendation in recommendations {
        let actions = &recommendation.actions;
        if is_insert_only_actions(actions) {
            let insert_text = get_insert_text_from_insert_only_actions(actions);
            single_inserts.insert(insert_text.clone());
        }
    }
    single_inserts
}

fn compute_max_nonidentical_prefix_or_suffix_similarity(
    text: &str,
    others: &HashSet<String>,
) -> usize {
    let mut best = 0;
    for other in others {
        if other != text && other.len() >= best {
            let smallest_size = usize::min(other.len(), text.len());
            for i in 1..=smallest_size {
                let text_sub_string = &text[text.len() - i..];
                if &other[other.len() - i..] == text_sub_string {
                    best = usize::max(text_sub_string.len(), best);
                } else {
                    break;
                }
            }
            for i in 1..=smallest_size {
                let text_sub_string = &text[..i];
                if &other[..i] == text_sub_string {
                    best = usize::max(text_sub_string.len(), best);
                } else {
                    break;
                }
            }
        }
    }
    best
}

fn score_recommendations_weighting_by_inverse_action_frequency(
    recommendations: &Vec<CommandStatistics>,
    num_commands_including_action: &HashMap<String, usize>,
    single_inserts: &HashSet<String>,
) -> f64 {
    let mut score = 0.0;
    for recommendation in recommendations {
        let actions = &recommendation.actions;
        if is_insert_only_actions(actions) && single_inserts.len() > 1 {
            let inserted_text = get_insert_text_from_insert_only_actions(actions);
            let similarity =
                compute_max_nonidentical_prefix_or_suffix_similarity(inserted_text, single_inserts);
            let weight = if similarity == 0 {
                1.0
            } else {
                (inserted_text.len() - similarity) as f64 / inserted_text.len() as f64
            };
            score += weight * recommendation.number_of_words_saved as f64;
        } else {
            let mut weight = 0.0;
            for action in actions {
                let representation = action.to_json();
                weight += 1.0
                    / (*num_commands_including_action
                        .get(&representation)
                        .unwrap_or(&1)) as f64;
            }
            weight /= actions.len() as f64;
            score += weight * recommendation.number_of_words_saved as f64;
        }
    }
    score
}

fn compute_heuristic_recommendation_score(recommendations: &Vec<CommandStatistics>) -> f64 {
    let num_commands_including_action =
        compute_number_of_commands_including_action(recommendations);
    let single_inserts = compute_single_inserts_from_commands(recommendations);
    score_recommendations_weighting_by_inverse_action_frequency(
        recommendations,
        &num_commands_including_action,
        &single_inserts,
    )
}

fn compute_greedy_best_in_parallel(
    recommendations: &Vec<CommandStatistics>,
    max_number_of_recommendations: usize,
) -> Vec<CommandStatistics> {
    let mut pool: pool::ThreadPool<(usize, f64)> = pool::ThreadPool::create_with_max_threads();
    let mut best_recommendations = Vec::new();
    let mut consumed_indexes = HashSet::new();
    let recommendations = recommendations.clone();
    let recommendations = Arc::new(recommendations);
    while best_recommendations.len() < max_number_of_recommendations
        && best_recommendations.len() < recommendations.len()
    {
        let num_workers = pool.compute_number_of_workers();
        let mut starting_index = 0;
        let chunk_size = recommendations.len() / num_workers;
        let consumed_arc = Arc::new(consumed_indexes.clone());
        for _ in 0..num_workers {
            let target_index = recommendations.len().min(starting_index + chunk_size);
            let start = starting_index;
            let recommendations_clone = Arc::clone(&recommendations);
            let consumed_clone = Arc::clone(&consumed_arc);
            let best_recommendations_clone = best_recommendations.clone();
            pool.execute(move || {
                let mut best_score = f64::NEG_INFINITY;
                let mut best_index = 0;
                let mut current_recommendations = best_recommendations_clone.clone();
                for i in start..target_index {
                    if !consumed_clone.contains(&i) {
                        let recommendation = &recommendations_clone[i];
                        current_recommendations.push(recommendation.clone());
                        let score =
                            compute_heuristic_recommendation_score(&current_recommendations);
                        if score > best_score {
                            best_score = score;
                            best_index = i;
                        }
                        current_recommendations.pop();
                    }
                }
                (best_index, best_score)
            });
            starting_index = target_index;
        }
        let (best_index, _best_score) = pool.reduce(|a, b| if a.1 > b.1 { a } else { b });
        best_recommendations.push(recommendations[best_index].clone());
        consumed_indexes.insert(best_index);
    }
    best_recommendations
}

fn compute_greedy_best(
    recommendations: &Vec<CommandStatistics>,
    max_number_of_recommendations: usize,
) -> Vec<CommandStatistics> {
    // Finds the best recommendations by for every n-th recommendation
    // finding the recommendation that has the best score with the ones chosen so far
    let mut best_recommendations = Vec::new();
    let mut consumed_indexes = HashSet::new();
    while best_recommendations.len() < max_number_of_recommendations
        && best_recommendations.len() < recommendations.len()
    {
        let mut best_score = f64::NEG_INFINITY;
        let mut best_index = 0;
        for (index, recommendation) in recommendations.iter().enumerate() {
            if !consumed_indexes.contains(&index) {
                best_recommendations.push(recommendation.clone());
                let score = compute_heuristic_recommendation_score(&best_recommendations);
                if score > best_score {
                    best_score = score;
                    best_index = index;
                }
                best_recommendations.pop();
            }
        }
        best_recommendations.push(recommendations[best_index].clone());
        consumed_indexes.insert(best_index);
    }
    best_recommendations
}

fn compute_string_subsequences(text: &str) -> Vec<String> {
    let mut subsequences = Vec::new();
    for i in 0..text.len() {
        for j in i..text.len() {
            if j - i + 1 < text.len() {
                subsequences.push(text[i..=j].to_string());
            }
        }
    }
    subsequences
}

fn append_insert_subsequences(collection: &mut Vec<String>, action: &BasicAction) {
    let inserted_text = get_insert_text(action);
    for s in compute_string_subsequences(inserted_text) {
        let action = create_insert_action(&s);
        let rep = action.to_json();
        collection.push(rep);
    }
}

fn _append_insert_subsequences_with_multiple_actions(
    collection: &mut Vec<String>,
    sub_actions: &[BasicAction],
) {
    // This assumes that there is more than one action
    let mut beginning_inserts = Vec::new();
    let mut ending_inserts = Vec::new();
    if is_insert(&sub_actions[0]) {
        let inserted_text = get_insert_text(&sub_actions[0]);
        if inserted_text.len() > 1 {
            for i in 1..inserted_text.len() {
                beginning_inserts.push(inserted_text[i..].to_string());
            }
        }
    }
    if is_insert(&sub_actions[sub_actions.len() - 1]) {
        let inserted_text = get_insert_text(&sub_actions[sub_actions.len() - 1]);
        if inserted_text.len() > 1 {
            for i in 1..inserted_text.len() {
                ending_inserts.push(inserted_text[..i].to_string());
            }
        }
    }
    if is_insert(&sub_actions[0]) && !is_insert(&sub_actions[sub_actions.len() - 1]) {
        let other_representation = compute_string_representation_of_actions(&sub_actions[1..]);
        for s in &beginning_inserts {
            let s_rep = create_insert_action(s).to_json();
            collection.push(format!("{}{}", s_rep, other_representation));
        }
    } else if is_insert(&sub_actions[sub_actions.len() - 1]) && !is_insert(&sub_actions[0]) {
        let other_representation =
            compute_string_representation_of_actions(&sub_actions[..sub_actions.len() - 1]);
        for s in &ending_inserts {
            let s_rep = create_insert_action(s).to_json();
            collection.push(format!("{}{}", other_representation, s_rep));
        }
    } else if is_insert(&sub_actions[0]) && is_insert(&sub_actions[sub_actions.len() - 1]) {
        let other_representation =
            compute_string_representation_of_actions(&sub_actions[1..sub_actions.len() - 1]);
        for (i, b) in beginning_inserts.iter().enumerate() {
            let b_rep = create_insert_action(b).to_json();
            for (j, e) in ending_inserts.iter().enumerate() {
                if i != beginning_inserts.len() - 1 || j != ending_inserts.len() - 1 {
                    let e_rep = create_insert_action(e).to_json();
                    collection.push(format!("{}{}{}", b_rep, other_representation, e_rep));
                }
            }
        }
    }
}

fn compute_action_subsequences_including_leading_and_trailing_inserts(
    actions: &[BasicAction],
) -> Vec<String> {
    let mut subsequences = Vec::new();
    for i in 0..actions.len() {
        for j in i..actions.len() {
            let sub_actions = &actions[i..=j];
            if sub_actions.len() < actions.len() {
                subsequences.push(compute_string_representation_of_actions(sub_actions));
            }
            if sub_actions.len() == 1 && is_insert(&sub_actions[0]) {
                append_insert_subsequences(&mut subsequences, &sub_actions[0]);
            } else if sub_actions.len() > 1 {
                _append_insert_subsequences_with_multiple_actions(&mut subsequences, sub_actions);
            }
        }
    }
    subsequences
}

fn find_redundant_commands_from_command(
    sequence: String,
    sequences: &HashMap<String, CommandStatistics>,
) -> Vec<String> {
    let mut redundant = Vec::new();
    let command = sequences
        .get(&sequence)
        .expect("Command not found in sequences");
    for sub_sequence in
        compute_action_subsequences_including_leading_and_trailing_inserts(&command.actions)
    {
        if let Some(existing_command) = sequences.get(&sub_sequence) {
            if existing_command.number_of_times_used == command.number_of_times_used {
                redundant.push(sub_sequence);
            }
        }
    }
    redundant
}

fn filter_out_recommendations_redundant_smaller_commands(
    recommendations: Vec<CommandStatistics>,
) -> Vec<CommandStatistics> {
    // For every command that is a shorter version of another command but is not used any more times: remove it
    let mut pool = pool::ThreadPool::create_with_max_threads();
    let mut action_sequences: HashMap<String, CommandStatistics> = HashMap::new();
    for command in recommendations.into_iter() {
        let representation = compute_string_representation_of_actions(&command.actions);
        action_sequences.insert(representation, command);
    }
    let action_sequences = Arc::new(RwLock::new(action_sequences));

    for sequence in action_sequences.read().unwrap().keys() {
        let action_sequences_clone = Arc::clone(&action_sequences);
        let sequence = sequence.clone();
        pool.execute(move || {
            find_redundant_commands_from_command(sequence, &action_sequences_clone.read().unwrap())
        });
    }
    let results = pool.join();
    let mut action_sequences = action_sequences.write().unwrap();
    for result in results {
        for sub_sequence in result {
            action_sequences.remove(&sub_sequence);
        }
    }
    let result: Vec<CommandStatistics> = action_sequences.values().cloned().collect();
    result
}

pub fn find_best(
    recommendations: Vec<CommandStatistics>,
    max_number_of_recommendations: usize,
) -> Vec<CommandStatistics> {
    if max_number_of_recommendations >= recommendations.len() {
        return recommendations.clone();
    }
    let recommendations = filter_out_recommendations_redundant_smaller_commands(recommendations);
    println!(
        "Narrowed it down to {} recommendations",
        recommendations.len()
    );
    compute_greedy_best_in_parallel(&recommendations, max_number_of_recommendations)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_can_find_greedy_best() {
        let recommendations = vec![
            CommandStatistics {
                number_of_times_used: 50,
                number_of_words_saved: 1000,
                instantiation_set: None,
                actions: vec![create_insert_action("arbitrary")],
                number_of_actions: 1,
                total_number_of_words_dictated: 100,
            },
            CommandStatistics {
                number_of_times_used: 20,
                number_of_words_saved: 40,
                instantiation_set: None,
                actions: vec![create_insert_action("text")],
                number_of_actions: 1,
                total_number_of_words_dictated: 20,
            },
            CommandStatistics {
                number_of_times_used: 5000,
                number_of_words_saved: 20000,
                instantiation_set: None,
                actions: vec![create_insert_action("mod tests {\n]")],
                number_of_actions: 1,
                total_number_of_words_dictated: 400,
            },
            CommandStatistics {
                number_of_times_used: 20,
                number_of_words_saved: 30,
                instantiation_set: None,
                actions: vec![create_insert_action("tarp2")],
                number_of_actions: 1,
                total_number_of_words_dictated: 20,
            },
        ];
        let recommendations_clone = recommendations.clone();
        let best = find_best(recommendations_clone, 2);
        assert_eq!(best.len(), 2);
        assert_eq!(best[0].actions, recommendations[2].actions);
        assert_eq!(best[1].actions, recommendations[0].actions);
    }
}
