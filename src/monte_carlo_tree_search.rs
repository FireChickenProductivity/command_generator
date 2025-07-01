use crate::pool::ThreadPool;
use crate::random::RandomNumberGenerator;
use crate::recommendation_generation::CommandStatistics;
use crate::recommendation_scoring::{
    compute_greedy_best, compute_greedy_best_in_parallel, compute_heuristic_recommendation_score,
};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    thread,
};

#[derive(Clone)]
pub struct NodeData {
    pub index: usize,
    pub score: f64,
    pub times_explored: usize,
}

/// Represents a node in the Monte Carlo Tree Search (MCTS) tree.
pub struct ScoredNode {
    children: HashMap<usize, ScoredNode>,
    data: NodeData,
}

impl ScoredNode {
    pub fn new(index: usize) -> Self {
        ScoredNode {
            children: HashMap::new(),
            data: NodeData {
                index,
                score: 0.0,
                times_explored: 0,
            },
        }
    }

    pub fn get_data(&self) -> &NodeData {
        &self.data
    }

    pub fn get_index(&self) -> usize {
        self.data.index
    }

    pub fn get_children(&self) -> impl Iterator<Item = &ScoredNode> {
        self.children.values()
    }

    pub fn has_children(&self) -> bool {
        !self.children.is_empty()
    }

    pub fn get_children_dictionary(&self) -> &HashMap<usize, ScoredNode> {
        &self.children
    }

    pub fn get_score(&self) -> f64 {
        if self.data.times_explored == 0 {
            0.0
        } else {
            self.data.score / self.data.times_explored as f64
        }
    }

    pub fn get_total_score(&self) -> f64 {
        self.data.score
    }

    pub fn get_times_explored(&self) -> usize {
        self.data.times_explored
    }

    pub fn handle_score(&mut self, score: f64) {
        self.data.score += score;
    }

    pub fn handle_exploration(&mut self, times: usize) {
        self.data.times_explored += times;
    }

    pub fn get_child_mut(&mut self, index: usize) -> &mut ScoredNode {
        if !self.children.contains_key(&index) {
            let child = ScoredNode::new(index);
            self.children.insert(index, child);
        }
        self.children.get_mut(&index).unwrap()
    }

    pub fn get_child(&self, index: usize) -> &ScoredNode {
        self.children.get(&index).expect("Child not found")
    }
}

/// Contains data on the current Monte Carlo Tree Search exploration
pub struct MonteCarloExplorationData {
    roots: HashMap<usize, ScoredNode>,
    total_explored: usize,
}

impl MonteCarloExplorationData {
    pub fn new() -> Self {
        MonteCarloExplorationData {
            roots: HashMap::new(),
            total_explored: 0,
        }
    }

    pub fn back_propagate_score(&mut self, path: &[usize], score: f64) {
        if let Some(root) = self.roots.get_mut(&path[0]) {
            root.handle_score(score);
            for &index in &path[1..] {
                let child = root.get_child_mut(index);
                child.handle_score(score);
            }
        } else {
            panic!("Root node not found for path: {:?}", path);
        }
    }

    pub fn get_root(&mut self, index: usize) -> &mut ScoredNode {
        self.roots
            .entry(index)
            .or_insert_with(|| ScoredNode::new(index))
    }

    pub fn progress_has_children(&self, progress: Option<&ScoredNode>) -> bool {
        match progress {
            Some(p) => p.has_children(),
            None => !self.roots.is_empty(),
        }
    }

    fn compute_best_score<'a>(children: impl Iterator<Item = &'a ScoredNode>) -> f64 {
        let mut best_score = 0.0;
        children.for_each(|x| {
            if x.get_score() > best_score {
                best_score = x.get_score();
            }
        });
        best_score
    }

    fn compute_best_child_from_iterable<'a>(
        children: impl Iterator<Item = &'a ScoredNode>,
        times_parent_explored: usize,
        best_score: f64,
        c: f64,
    ) -> (usize, f64) {
        let mut best_value = 0.0;
        let mut best_index = 0;
        for child in children {
            let value = child.get_score() / best_score
                + c * ((times_parent_explored as f64).ln() / child.get_times_explored() as f64)
                    .sqrt();
            if value > best_value {
                best_index = child.get_index();
                best_value = value;
            }
        }
        (best_index, best_value)
    }

    fn compute_best_child_from_node(node: &ScoredNode, c: f64) -> (usize, f64) {
        let children = node.get_children();
        let best_score = Self::compute_best_score(children);
        let times_parent_explored = node.get_times_explored();
        Self::compute_best_child_from_iterable(
            node.get_children(),
            times_parent_explored,
            best_score,
            c,
        )
    }

    /// Assumes that progress has a child and picks the best one using UCT
    pub fn compute_best_child(&self, progress: Option<&ScoredNode>, c: f64) -> (usize, f64) {
        if let Some(progress) = progress {
            Self::compute_best_child_from_node(progress, c)
        } else {
            let children = self.roots.values();
            let best_score = Self::compute_best_score(self.roots.values());
            let times_parent_explored = self.total_explored;
            Self::compute_best_child_from_iterable(children, times_parent_explored, best_score, c)
        }
    }

    pub fn compute_next_index_after_exploration(&self, progress: &Option<ScoredNode>) -> usize {
        match progress {
            Some(progress) => {
                progress
                    .get_children()
                    .max_by_key(|x| x.get_index())
                    .unwrap()
                    .get_index()
                    + 1
            }
            None => self.roots.keys().max().map_or(0, |&x| x + 1),
        }
    }

    pub fn handle_exploration(&mut self, path: &[usize], times: usize) {
        self.increment_total_explored(times);
        let mut progress = self.get_root(path[0]);
        progress.handle_exploration(times);
        for &choice in &path[1..] {
            progress = progress.get_child_mut(choice);
            progress.handle_exploration(times);
        }
    }

    pub fn handle_expansion(&mut self, path: &[usize]) {
        let mut progress = self.get_root(path[0]);
        for &choice in &path[1..] {
            progress = progress.get_child_mut(choice);
        }
    }

    pub fn create_initial_for_path(&mut self, path: &[usize]) -> &mut ScoredNode {
        // In the original python program, I counted this as an exploration for some reason. My decision to not do that may cause a bug.
        let mut progress = self.get_root(path[0]);
        for &choice in &path[1..] {
            progress = progress.get_child_mut(choice);
        }
        progress
    }

    pub fn increment_total_explored(&mut self, times: usize) {
        self.total_explored += times;
    }

    pub fn get_roots_mut(&mut self) -> &mut HashMap<usize, ScoredNode> {
        &mut self.roots
    }

    pub fn get_roots(&self) -> &HashMap<usize, ScoredNode> {
        &self.roots
    }
}

struct SearchConstants {
    pub c: f64,
    pub rollouts_per_exploration: usize,
    pub rollouts_per_child_expansion: usize,
    pub maximum_depth: usize,
    pub recommendation_limit: usize,
}

struct Roller<'a> {
    best_recommendation: Vec<CommandStatistics>,
    best_score: f64,
    best_recommendation_indexes: Vec<usize>,
    recommendations: &'a Vec<CommandStatistics>,
    generator: RandomNumberGenerator,
}

impl<'a> Roller<'a> {
    fn new(recommendations: &'a Vec<CommandStatistics>, seed: u64) -> Self {
        Roller {
            best_recommendation: Vec::new(),
            best_score: 0.0,
            best_recommendation_indexes: Vec::new(),
            recommendations,
            generator: RandomNumberGenerator::new(seed),
        }
    }
    fn simulate_play_out(
        &mut self,
        starting_path: &[usize],
        use_greedy: bool,
        constants: &SearchConstants,
    ) -> f64 {
        let mut path = starting_path.to_vec();
        let num_remaining = constants.recommendation_limit - starting_path.len();
        let mut next_possible_index = *path.last().unwrap_or(&0);
        let mut last_potential_index = self.recommendations.len() - num_remaining;
        let num_random = if use_greedy {
            std::cmp::max(num_remaining - 1, 0)
        } else {
            num_remaining
        };

        for _ in 0..num_random {
            let choice = self
                .generator
                .next_in_range(next_possible_index, last_potential_index);
            next_possible_index = choice + 1;
            last_potential_index += 1;
            path.push(choice);
        }

        let (potential_recommendations, score, path) = if use_greedy {
            let (potential_recommendations, score, mut path) = compute_greedy_best(
                &self.recommendations,
                constants.recommendation_limit,
                &path,
                (next_possible_index, last_potential_index + 1),
            );
            path.sort();
            (potential_recommendations, score, path)
        } else {
            let potential_recommendations: Vec<CommandStatistics> = path
                .iter()
                .map(|&i| self.recommendations[i].clone())
                .collect();
            let score = compute_heuristic_recommendation_score(&potential_recommendations);
            (potential_recommendations, score, path)
        };

        if score > self.best_score {
            self.best_score = score;
            self.best_recommendation = potential_recommendations;
            self.best_recommendation_indexes = path;
        }

        score
    }

    fn get_best_score(&self) -> f64 {
        self.best_score
    }

    fn get_best_recommendation_indexes(&self) -> &Vec<usize> {
        &self.best_recommendation_indexes
    }
}

// Instead of using initial_progress, exploration data will only keep track of what happens after the start
pub struct MonteCarloTreeSearcher<'a> {
    roller: Roller<'a>,
    start: Vec<usize>,
    constants: SearchConstants,
    exploration_data: MonteCarloExplorationData,
    random_number_generator: RandomNumberGenerator,
}

fn explore_every_child(
    path: &mut Vec<usize>,
    data: &mut MonteCarloExplorationData,
    roller: &mut Roller,
    constants: &SearchConstants,
    start_length: usize,
) -> usize {
    let start = if path.is_empty() {
        0
    } else {
        *path.last().unwrap() + 1
    };
    let ending = roller.recommendations.len() - constants.recommendation_limit + path.len();
    let starting_path_index = if start_length < path.len() {
        // I only need to handle counting exploration when we are past the root
        data.handle_exploration(&path[start_length..], ending - start);
        path.len()
    } else {
        0
    };

    for i in start..ending {
        path.push(i);
        let progress = data.get_root(path[starting_path_index]);
        for _ in 0..constants.rollouts_per_child_expansion {
            let score = roller.simulate_play_out(path, false, constants);
            progress.handle_score(score);
            for j in starting_path_index + 1..path.len() {
                let progress = progress.get_child_mut(path[j]);
                progress.handle_score(score);
            }
        }
        progress.handle_exploration(constants.rollouts_per_child_expansion);
        path.pop();
    }
    let progress = &data.create_initial_for_path(path);
    MonteCarloExplorationData::compute_best_child_from_node(progress, constants.c).0
}

fn select_starting_path(
    data: &mut MonteCarloExplorationData,
    roller: &mut Roller,
    start: &Vec<usize>,
    constants: &SearchConstants,
) -> Vec<usize> {
    let mut path = start.clone();
    let starting_index = {
        let roots = data.get_roots();
        if roots.is_empty() {
            // This works because the starting path goes from 0 to 1 minus the length of the path. This invariant is maintained outside of the data structure.
            path.len()
        } else {
            let (index, _) = data.compute_best_child(None, constants.c);
            index
        }
    };
    let has_children = {
        let roots = data.get_roots();
        let mut progress = roots.get(&starting_index).unwrap();
        path.push(starting_index);
        while path.len() < constants.maximum_depth - 1 && progress.has_children() {
            let (best_child, _) =
                MonteCarloExplorationData::compute_best_child_from_node(progress, constants.c);
            progress = progress.get_child(best_child);
            path.push(best_child);
        }
        progress.has_children()
    };
    if path.len() < constants.maximum_depth - 1 && !has_children {
        let best_child = explore_every_child(&mut path, data, roller, constants, start.len());
        path.push(best_child);
    }
    path
}

impl<'a> MonteCarloTreeSearcher<'a> {
    pub fn new(
        recommendation_limit: usize,
        recommendations: &'a Vec<CommandStatistics>,
        start: Vec<usize>,
        seed: u64,
    ) -> Self {
        let max_depth = recommendation_limit - start.len() - 1;
        let max_remaining_depth = std::cmp::min(start.len() + max_depth, recommendation_limit);
        MonteCarloTreeSearcher {
            roller: Roller::new(recommendations, seed),
            exploration_data: MonteCarloExplorationData::new(),
            start,
            constants: SearchConstants {
                c: 0.000001,
                rollouts_per_exploration: 10,
                rollouts_per_child_expansion: 1,
                maximum_depth: max_remaining_depth,
                recommendation_limit,
            },
            random_number_generator: RandomNumberGenerator::new(seed),
        }
    }

    pub fn get_best_score(&self) -> f64 {
        self.roller.get_best_score()
    }

    pub fn get_best_recommendation_indexes(&self) -> &Vec<usize> {
        self.roller.get_best_recommendation_indexes()
    }

    fn explore_solution(&mut self) {
        let path = select_starting_path(
            &mut self.exploration_data,
            &mut self.roller,
            &self.start,
            &self.constants,
        );
        assert!(path.len() <= self.constants.recommendation_limit);
        let path_after_start = &path[self.start.len()..];
        self.exploration_data.handle_expansion(path_after_start);
        for _ in 0..self.constants.rollouts_per_exploration {
            let score = self.roller.simulate_play_out(&path, true, &self.constants);
            self.exploration_data
                .back_propagate_score(&path_after_start, score);
        }
        self.exploration_data
            .handle_exploration(&path_after_start, self.constants.rollouts_per_exploration);
    }

    pub fn explore_solutions(&mut self, num_trials: usize) {
        for _ in 0..num_trials {
            self.explore_solution();
        }
    }

    pub fn get_root_values(&self) -> HashMap<usize, (f64, usize)> {
        let roots = self.exploration_data.get_roots();
        let mut values = HashMap::new();
        for (key, root) in roots {
            values.insert(
                key.clone(),
                (root.get_total_score(), root.get_times_explored()),
            );
        }
        values
    }
}

fn perform_double_greedy(
    indexes: Vec<usize>,
    search_start_index: usize,
    recommendations: &Vec<CommandStatistics>,
    recommendation_limit: usize,
) -> (f64, Vec<usize>) {
    let mut best_score = -1.0;
    let mut best_index = 0;
    let mut best_indexes = indexes.clone();

    for i in search_start_index..recommendations.len() {
        best_indexes.push(i);
        let (_, score) =
            compute_greedy_best_in_parallel(recommendations, recommendation_limit, &best_indexes);
        if score > best_score {
            best_score = score;
            best_index = i;
        }
        best_indexes.pop();
    }

    println!("best score from double greedy: {}", best_score);
    best_indexes.push(best_index);
    (best_score, best_indexes)
}

fn perform_worker_monte_carlo_tree_search<'a>(
    recommendations: &'a Vec<CommandStatistics>,
    start: &Vec<usize>,
    recommendation_limit: usize,
    seed: u64,
    number_of_trials: usize,
) -> MonteCarloTreeSearcher<'a> {
    let mut searcher =
        MonteCarloTreeSearcher::new(recommendation_limit, recommendations, start.clone(), seed);
    searcher.explore_solutions(number_of_trials);
    searcher
}

fn compute_best_index_from_aggregation(aggregation: &HashMap<usize, (f64, usize)>) -> usize {
    let mut best_score = 0.0;
    let mut best_index = 0;

    for (&key, &(score, num_explored)) in aggregation.iter() {
        let average_score = score / num_explored as f64;
        if average_score > best_score {
            best_index = key;
            best_score = average_score;
        }
    }
    best_index
}

// def perform_possibly_parallel_monte_carlo_tree_search(scoring_function, recommendation_limit, recommendations, indexes, greedy_function, number_of_trials: int, aggregate_tree=True, cores_override: int=None, c: float=1):
//     try:
//         if cores_override:
//             num_workers = min(cores_override, multiprocessing.cpu_count())
//         else:
//             num_workers = multiprocessing.cpu_count()
//     except:
//         num_workers = 1
//     trials_per_worker = number_of_trials if num_workers == 1 else round(1.7*number_of_trials/num_workers)
//     search_arguments = (max(trials_per_worker, 10), scoring_function, recommendation_limit, recommendations, indexes, recommendation_limit - len(indexes) - 1, greedy_function, c)
//     if num_workers == 1:
//         return perform_worker_monte_carlo_tree_search(*search_arguments)
//     else:
//         results = []
//         with multiprocessing.Pool(processes=num_workers) as p:
//             for _ in range(num_workers):
//                 result = p.apply_async(perform_worker_monte_carlo_tree_search, search_arguments, {"aggregate_tree": aggregate_tree})
//                 results.append(result)
//             best_score = 0
//             best_recommendation_indexes: list[int]
//             if aggregate_tree:
//                 value_aggregation = None
//             for result in results:
//                 if aggregate_tree:
//                     values, score, indexes = result.get()
//                     if value_aggregation is None:
//                         value_aggregation = values
//                     else:
//                         for key in values:
//                             total_score, num_explorations = values[key]
//                             value_aggregation[key][0] += total_score
//                             value_aggregation[key][1] += num_explorations
//                 else:
//                     score, indexes = result.get()
//                 if score > best_score:
//                     best_score = score
//                     best_recommendation_indexes = indexes
//         print(f"Best score {best_score} using {num_workers} workers.")
//         if aggregate_tree:
//             return compute_best_index_from_aggregation(value_aggregation), best_score, best_recommendation_indexes
//         return best_score, best_recommendation_indexes
