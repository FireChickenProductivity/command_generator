use crate::pool::ThreadPool;
use crate::random::RandomNumberGenerator;
use crate::recommendation_generation::CommandStatistics;
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
    depth: usize,
}

impl ScoredNode {
    pub fn new(index: usize, depth: usize) -> Self {
        ScoredNode {
            children: HashMap::new(),
            data: NodeData {
                index,
                score: 0.0,
                times_explored: 0,
            },
            depth,
        }
    }

    pub fn get_depth(&self) -> usize {
        self.depth
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
            let child = ScoredNode::new(index, self.depth + 1);
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

    pub fn get_progress_from_choice<'a>(
        &'a mut self,
        choice: usize,
        progress: Option<&'a mut ScoredNode>,
    ) -> &'a mut ScoredNode {
        if let Some(progress) = progress {
            progress.get_child(choice)
        } else {
            self.roots
                .entry(choice)
                .or_insert_with(|| ScoredNode::new(choice, 0))
        }
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

    /// Assumes that progress has a child and picks the best one using UCT
    pub fn compute_best_child(&self, progress: Option<&ScoredNode>, c: f64) -> (usize, f64) {
        if let Some(progress) = progress {
            let children = progress.get_children();
            let best_score = Self::compute_best_score(progress.get_children());
            let times_parent_explored = progress.get_times_explored();
            Self::compute_best_child_from_iterable(children, times_parent_explored, best_score, c)
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

    pub fn handle_expansion(&mut self, path: &[usize]) {
        let mut progress = None;
        for &choice in path {
            progress = Some(self.get_progress_from_choice(choice, progress));
        }
    }

    pub fn handle_exploration(&mut self, path: &[usize], times: usize) {
        self.total_explored += times;
        let mut progress = None;
        for &choice in path {
            progress = Some(self.get_progress_from_choice(choice, progress));
            if let Some(p) = progress {
                p.handle_exploration(times);
            }
        }
    }

    pub fn handle_child_exploration(&mut self, child: &mut ScoredNode) {
        child.handle_exploration(1);
    }

    pub fn create_initial_for_path(&mut self, path: &[usize]) -> &mut ScoredNode {
        let mut progress = None;
        for &choice in path {
            progress = Some(self.get_progress_from_choice(choice, progress));
            if let Some(p) = progress {
                p.handle_exploration(1);
            }
        }
        progress.unwrap()
    }
}
