use crate::pool::ThreadPool;
use crate::random::RandomNumberGenerator;
use crate::recommendation_generation::CommandStatistics;
use crate::recommendation_scoring::{compute_greedy_best, compute_heuristic_recommendation_score};
use std::collections::HashMap;

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

    pub fn get_child(&mut self, index: usize) -> &mut ScoredNode {
        if !self.children.contains_key(&index) {
            let child = ScoredNode::new(index);
            self.children.insert(index, child);
        }
        self.children.get_mut(&index).unwrap()
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
                let child = root.get_child(index);
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
            progress = progress.get_child(choice);
            progress.handle_exploration(times);
        }
    }

    pub fn handle_expansion(&mut self, path: &[usize]) {
        let mut progress = self.get_root(path[0]);
        for &choice in &path[1..] {
            progress = progress.get_child(choice);
        }
    }

    pub fn handle_child_exploration(&mut self, child: &mut ScoredNode) {
        child.handle_exploration(1);
    }

    pub fn create_initial_for_path(&mut self, path: &[usize]) -> &mut ScoredNode {
        // In the original python program, I counted this as an exploration for some reason. My decision to not do that may cause a bug.
        let mut progress = self.get_root(path[0]);
        for &choice in &path[1..] {
            progress = progress.get_child(choice);
        }
        progress
    }

    pub fn increment_total_explored(&mut self, times: usize) {
        self.total_explored += times;
    }

    pub fn get_roots(&mut self) -> &mut HashMap<usize, ScoredNode> {
        &mut self.roots
    }
}

// class MonteCarloTreeSearcher:
//     def __init__(
//         self,
//         scoring_function,
//         recommendation_limit: int,
//         recommendations: list[PotentialCommandInformation],
//         start,
//         maximum_depth: int,
//         greedy_function,
//         c: float,
//         greedy_depth: int=1,
//         rollouts_per_exploration: int=10,
//         rollouts_per_child_expansion: int=1,
//     ):
//         """recommendations should be sorted in ascending order of value"""
//         self.scoring_function = scoring_function
//         self.best_recommendation: list[PotentialCommandInformation]
//         self.best_score: int = 0
//         self.best_recommendation_indexes: list[int]
//         self.recommendation_limit = recommendation_limit
//         self.exploration_data = MonteCarloExplorationData()
//         self.recommendations = recommendations
//         self.start = start
//         self.initial_progress = self.exploration_data.create_initial_for_path(self.start)
//         self.maximum_depth = min(len(start) + maximum_depth, recommendation_limit)
//         self.rollouts_per_exploration = rollouts_per_exploration
//         self.rollouts_per_child_expansion = rollouts_per_child_expansion
//         self.greedy_function = greedy_function
//         self.greedy_depth = greedy_depth
//         self.c = c

//     def get_best_score(self):
//         return self.best_score

//     def get_best_recommendation_indexes(self):
//         return self.best_recommendation_indexes

//     def simulate_play_out(
//             self,
//             starting_path: list[int],
//             use_greedy: bool=False,
//             greedy_breadth: int=None,
//         ):
//         path = starting_path[:]
//         num_remaining = self.recommendation_limit - len(starting_path)
//         next_possible_index = path[-1]
//         last_potential_index = len(self.recommendations) - num_remaining
//         if use_greedy:
//             num_random = max(num_remaining - self.greedy_depth, 0)
//         else:
//             num_random = num_remaining
//         for _ in range(num_random):
//             choice = random.randint(next_possible_index, last_potential_index)
//             next_possible_index = choice + 1
//             last_potential_index += 1
//             path.append(choice)
//         if use_greedy:
//             if greedy_breadth:
//                 last_potential_index = min(next_possible_index + greedy_breadth - 1, last_potential_index)
//             potential_recommendations, score, path = self.greedy_function(self.recommendation_limit, self.recommendations, self.scoring_function, start=path, index_range=(next_possible_index, last_potential_index + 1))
//             path = sorted(path)
//         else:
//             potential_recommendations = [self.recommendations[i] for i in path]
//             score = self.scoring_function(potential_recommendations)
//         if score > self.best_score:
//             self.best_score = score
//             self.best_recommendation = potential_recommendations
//             self.best_recommendation_indexes = path
//         self.exploration_data.back_propagate_score(starting_path, score)

//     def select_next_starting_path(self):
//         #Recursively pick best node until reaching leaf
//         path = self.start[:]
//         progress = self.initial_progress
//         best_child, _ = self.exploration_data.compute_best_child(progress, self.c)
//         while best_child is not None and len(path) < self.maximum_depth - 1:
//             path.append(best_child)
//             progress = self.exploration_data.get_progress_from_choice(best_child, progress)
//             best_child, _ = self.exploration_data.compute_best_child(progress, self.c)
//         if len(path) < self.maximum_depth - 1 and best_child is None:
//             self.explore_every_child(path)
//             best_child, _ = self.exploration_data.compute_best_child(progress, self.c)
//             path.append(best_child)
//         return path

//     def explore_every_child(self, starting_path: list[int]):
//         if len(starting_path) < self.maximum_depth:
//             start = starting_path[-1] + 1 if starting_path else 0
//             ending = len(self.recommendations) - self.recommendation_limit + (len(starting_path))
//             self.exploration_data.handle_exploration(starting_path, ending - start)
//             progress = self.exploration_data.create_initial_for_path(starting_path)
//             for i in range(start, ending):
//                 starting_path.append(i)
//                 self.expand(starting_path)
//                 for _ in range(self.rollouts_per_child_expansion): self.simulate_play_out(starting_path, use_greedy=False)
//                 child = self.exploration_data.get_progress_from_choice(i, progress)
//                 self.exploration_data.handle_child_exploration(child)
//                 starting_path.pop()

//     def expand(self, path):
//         self.exploration_data.handle_expansion(path)

//     def explore_solution(self):
//         #Need to pick a good node to explore
//         #Need to do a play out
//         #Back propagate
//         starting_path = self.select_next_starting_path()
//         assert len(starting_path) <= self.recommendation_limit, (starting_path, self.recommendation_limit)
//         self.expand(starting_path)
//         for _ in range(self.rollouts_per_exploration): self.simulate_play_out(starting_path, use_greedy=True)
//         self.exploration_data.handle_exploration(starting_path, self.rollouts_per_exploration)

//     def explore_solutions(self, num_trials: int):
//         for _ in range(num_trials):
//             self.explore_solution()

//     def get_root_values(self):
//         roots = self.exploration_data.get_roots(self.initial_progress)
//         values = {}
//         for key in roots:
//             values[key] = [roots[key].get_total_score(), roots[key].get_times_explored()]
//         return values

// Instead of using initial_progress, exploration data will only keep track of what happens after the start
pub struct MonteCarloTreeSearcher {
    best_recommendation: Vec<CommandStatistics>,
    best_score: f64,
    best_recommendation_indexes: Vec<usize>,
    recommendation_limit: usize,
    exploration_data: MonteCarloExplorationData,
    recommendations: Vec<CommandStatistics>,
    start: Vec<usize>,
    maximum_depth: usize,
    rollouts_per_exploration: usize,
    rollouts_per_child_expansion: usize,
    c: f64,
    random_number_generator: RandomNumberGenerator,
}

impl MonteCarloTreeSearcher {
    pub fn new(
        recommendation_limit: usize,
        recommendations: Vec<CommandStatistics>,
        start: Vec<usize>,
        max_depth: usize,
        c: f64,
        rollouts_per_exploration: usize,
        rollouts_per_child_expansion: usize,
        seed: u64,
    ) -> Self {
        let maximum_depth = std::cmp::min(start.len() + max_depth, recommendation_limit);
        MonteCarloTreeSearcher {
            best_recommendation: Vec::new(),
            best_score: 0.0,
            best_recommendation_indexes: Vec::new(),
            recommendation_limit,
            exploration_data: MonteCarloExplorationData::new(),
            recommendations,
            start,
            maximum_depth,
            rollouts_per_exploration,
            rollouts_per_child_expansion,
            c,
            random_number_generator: RandomNumberGenerator::new(seed),
        }
    }

    pub fn get_best_score(&self) -> f64 {
        self.best_score
    }

    pub fn get_best_recommendation_indexes(&self) -> &Vec<usize> {
        &self.best_recommendation_indexes
    }

    fn simulate_play_out(&mut self, starting_path: &[usize], use_greedy: bool) {
        let mut path = starting_path.to_vec();
        let num_remaining = self.recommendation_limit - starting_path.len();
        let mut next_possible_index = *path.last().unwrap_or(&0);
        let mut last_potential_index = self.recommendations.len() - num_remaining;
        let num_random = if use_greedy {
            std::cmp::max(num_remaining - 1, 0)
        } else {
            num_remaining
        };

        for _ in 0..num_random {
            let choice = self
                .random_number_generator
                .next_in_range(next_possible_index, last_potential_index);
            next_possible_index = choice + 1;
            last_potential_index += 1;
            path.push(choice);
        }

        let (potential_recommendations, score, path) = if use_greedy {
            let (potential_recommendations, score, mut path) = compute_greedy_best(
                &self.recommendations,
                self.recommendation_limit,
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

        self.exploration_data
            .back_propagate_score(starting_path, score);
    }

    fn select_starting_path(&mut self) -> Vec<usize> {
        let mut path = self.start.clone();
        let starting_index = {
            let roots = self.exploration_data.get_roots();
            if roots.is_empty() {
                path.len()
            } else {
                let (index, _) = self.exploration_data.compute_best_child(None, self.c);
                index
            }
        };
        let progress = {
            let roots = self.exploration_data.get_roots();
            let mut progress = roots.get_mut(&starting_index).unwrap();
            path.push(starting_index);
            while path.len() < self.maximum_depth - 1 && progress.has_children() {
                let (best_child, _) =
                    MonteCarloExplorationData::compute_best_child_from_node(progress, self.c);
                progress = progress.get_child(best_child);
                path.push(best_child);
            }
            progress
        };
        if path.len() < self.maximum_depth - 1 && !progress.has_children() {
            self.explore_every_child(&mut path);
            let (best_child, _) =
                MonteCarloExplorationData::compute_best_child_from_node(progress, self.c);
            path.push(best_child);
        }
        path
    }

    fn explore_every_child(&mut self, starting_path: &mut Vec<usize>) {}
}
