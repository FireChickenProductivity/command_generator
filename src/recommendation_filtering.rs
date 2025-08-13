// Defines code for filtering recommendations matching specified criteria

use crate::action_records::BasicAction;
use crate::recommendation_generation::{ActionSet, CommandStatistics};

/// Filters out recommendations that the lambda function returns true on
pub fn filter_out_recommendations<F>(
    recommendations: Vec<CommandStatistics>,
    filter_fn: F,
) -> Vec<CommandStatistics>
where
    F: Fn(&CommandStatistics) -> bool,
{
    recommendations
        .into_iter()
        .filter(|rec| !filter_fn(rec))
        .collect()
}

/// Filters out recommendations that have any action match the filter function
pub fn filter_out_recommendations_with_action_matching_filter<F>(
    recommendations: Vec<CommandStatistics>,
    filter_fn: F,
) -> Vec<CommandStatistics>
where
    F: Fn(&BasicAction) -> bool,
{
    filter_out_recommendations(recommendations, |rec| {
        rec.actions.iter().any(|action| filter_fn(action))
    })
}

/// Filters out recommendations that contain any of the rejected actions.
pub fn filter_out_recommendations_containing_actions(
    recommendations: Vec<CommandStatistics>,
    rejected_actions: &ActionSet,
) -> Vec<CommandStatistics> {
    filter_out_recommendations_with_action_matching_filter(recommendations, |action| {
        rejected_actions.contains_action(action)
    })
}

/// Filters out recommendations containing the rejected action
pub fn filter_out_recommendations_containing_action(
    recommendations: Vec<CommandStatistics>,
    rejected_action: &BasicAction,
) -> Vec<CommandStatistics> {
    filter_out_recommendations_with_action_matching_filter(recommendations, |action| {
        action.to_json() == rejected_action.to_json()
    })
}
