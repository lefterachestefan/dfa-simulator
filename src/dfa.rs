use crate::{
    Automaton,
    prelude::*,
    raw_automaton::{RawAutomaton, advance_empty_word},
};
use petgraph::{
    Direction,
    graph::{DiGraph, NodeIndex},
    visit::EdgeRef,
};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet, VecDeque, hash_map::Entry};

/// Deterministic Finite Automaton
#[derive(Debug, Clone)]
pub struct Dfa {
    pub(crate) initial_state: u32,
    pub(crate) final_states: Vec<u32>,
    pub(crate) graph: DiGraph<u32, String>,
    pub(crate) alphabet: Vec<String>,
}

impl Automaton for Dfa {
    /// Runs the DFA on the given input string.
    fn run(&self, input: impl AsRef<str>) -> bool {
        let mut current_state = NodeIndex::new(self.initial_state as usize);
        let mut current_window = input.as_ref();

        while !current_window.is_empty() {
            let mut word_len = current_window.len();
            while word_len > 0
                && !self
                    .alphabet
                    .contains(&current_window[..word_len].to_string())
            {
                word_len -= 1;
            }
            if word_len == 0 {
                return false;
            }
            let (word, rest) = current_window.split_at(word_len);
            current_window = rest;
            let mut next_state = None;
            for edge in self
                .graph
                .edges_directed(current_state, Direction::Outgoing)
            {
                if *edge.weight() == word {
                    next_state = Some(edge.target());
                    break;
                }
            }
            match next_state {
                Some(state) => current_state = state,
                None => return false,
            }
        }

        self.final_states
            .contains(&u32::try_from(current_state.index()).unwrap_or(0))
    }

    /// Minimizes the DFA using Moore's algorithm (O(n^2)).
    fn minimize(&self) -> Self {
        let mut reachable = HashSet::new();
        let mut queue = VecDeque::new();
        let initial_node = NodeIndex::new(self.initial_state as usize);

        reachable.insert(initial_node);
        queue.push_back(initial_node);

        while let Some(node) = queue.pop_front() {
            for edge in self.graph.edges_directed(node, Direction::Outgoing) {
                if reachable.insert(edge.target()) {
                    queue.push_back(edge.target());
                }
            }
        }

        let nodes: Vec<NodeIndex> = reachable.into_iter().collect();
        if nodes.is_empty() {
            return self.clone();
        }

        let mut partition = create_partition(&self.final_states, &nodes);
        let mut num_partitions = 2;

        loop {
            let mut next_partition: HashMap<NodeIndex, usize> = HashMap::new();
            let mut signatures: HashMap<(usize, BTreeMap<String, usize>), usize> = HashMap::new();
            let mut next_num_partitions = 0;

            for &node in &nodes {
                let mut state_transitions = BTreeMap::new();
                for symbol in &self.alphabet {
                    let mut target_part = usize::MAX;
                    for edge in self.graph.edges_directed(node, Direction::Outgoing) {
                        if edge.weight() == symbol {
                            if let Some(&p) = partition.get(&edge.target()) {
                                target_part = p;
                            }
                            break;
                        }
                    }
                    state_transitions.insert(symbol.clone(), target_part);
                }
                let signature = (partition[&node], state_transitions);
                let part_id = if let Some(&id) = signatures.get(&signature) {
                    id
                } else {
                    let id = next_num_partitions;
                    signatures.insert(signature, id);
                    next_num_partitions += 1;
                    id
                };
                next_partition.insert(node, part_id);
            }

            if next_num_partitions == num_partitions {
                break;
            }

            partition = next_partition;
            num_partitions = next_num_partitions;
        }

        let initial_part = partition[&initial_node];
        let mut part_to_new_id = HashMap::new();
        let mut new_final_states = Vec::new();
        let mut next_new_id = 0u32;

        part_to_new_id.insert(initial_part, next_new_id);
        next_new_id += 1;

        for &node in &nodes {
            let part = partition[&node];
            if let Entry::Vacant(e) = part_to_new_id.entry(part) {
                e.insert(next_new_id);
                next_new_id += 1;
            }
            let new_id = part_to_new_id[&part];
            let is_final = self
                .final_states
                .contains(&u32::try_from(node.index()).unwrap_or(0));
            if is_final && !new_final_states.contains(&new_id) {
                new_final_states.push(new_id);
            }
        }

        let mut new_edges = Vec::new();
        let mut seen_edges = HashSet::new();

        for &node in &nodes {
            let from_part = partition[&node];
            let from_id = part_to_new_id[&from_part];
            for symbol in &self.alphabet {
                for edge in self.graph.edges_directed(node, Direction::Outgoing) {
                    if edge.weight() == symbol {
                        if let Some(&to_part) = partition.get(&edge.target()) {
                            let to_id = part_to_new_id[&to_part];
                            if seen_edges.insert((from_id, to_id, symbol.clone())) {
                                new_edges.push((from_id, to_id, symbol.clone()));
                            }
                        }
                        break;
                    }
                }
            }
        }

        Self {
            initial_state: 0,
            final_states: new_final_states,
            graph: DiGraph::from_edges(new_edges),
            alphabet: self.alphabet.clone(),
        }
    }
}

fn create_partition(final_states: &[u32], nodes: &Vec<NodeIndex>) -> HashMap<NodeIndex, usize> {
    let mut partition: HashMap<NodeIndex, usize> = HashMap::new();

    for &node in nodes {
        let is_final = final_states.contains(&u32::try_from(node.index()).unwrap_or(0));
        partition.insert(node, usize::from(is_final));
    }

    partition
}

impl From<Nfa> for Dfa {
    fn from(nfa: Nfa) -> Self {
        Self::from(LambdaNfa::from(nfa))
    }
}

impl From<LambdaDfa> for Dfa {
    fn from(ldfa: LambdaDfa) -> Self {
        Self::from(LambdaNfa::from(ldfa))
    }
}

impl From<RawAutomaton> for Dfa {
    fn from(raw: RawAutomaton) -> Self {
        for edge in &raw.edges {
            assert!(!edge.2.is_empty() && raw.alphabet.contains(&edge.2));
        }

        Self {
            initial_state: raw.initial_state,
            final_states: raw.final_states,
            graph: DiGraph::from_edges(raw.edges),
            alphabet: raw.alphabet,
        }
    }
}

impl From<LambdaNfa> for Dfa {
    fn from(nfa: LambdaNfa) -> Self {
        let mut dfa_states = HashMap::new();
        let mut queue = VecDeque::new();
        let mut dfa_edges: Vec<(u32, u32, String)> = Vec::new();
        let mut dfa_final_states = Vec::new();
        let initial_nfa_states = {
            let mut s = HashSet::new();
            s.insert(NodeIndex::new(nfa.initial_state as usize));
            advance_empty_word(&nfa.graph, &s)
        };
        let initial_nfa_states_sorted: BTreeSet<NodeIndex> =
            initial_nfa_states.into_iter().collect();
        let mut next_dfa_state = 0u32;

        dfa_states.insert(initial_nfa_states_sorted.clone(), next_dfa_state);
        queue.push_back(initial_nfa_states_sorted);
        next_dfa_state += 1;

        while let Some(current_nfa_states) = queue.pop_front() {
            let current_dfa_id = dfa_states.get(&current_nfa_states).copied().unwrap_or(0);

            if current_nfa_states.iter().any(|&idx| {
                nfa.final_states
                    .contains(&u32::try_from(idx.index()).unwrap_or(0))
            }) {
                dfa_final_states.push(current_dfa_id);
            }

            for symbol in &nfa.alphabet {
                let mut next_nfa_states = HashSet::new();

                for &nfa_state in &current_nfa_states {
                    for edge in nfa.graph.edges_directed(nfa_state, Direction::Outgoing) {
                        if edge.weight() == symbol {
                            next_nfa_states.insert(edge.target());
                        }
                    }
                }

                if next_nfa_states.is_empty() {
                    continue;
                }

                let next_nfa_states_closure = advance_empty_word(&nfa.graph, &next_nfa_states);
                let next_nfa_states_sorted: BTreeSet<NodeIndex> =
                    next_nfa_states_closure.into_iter().collect();
                let next_dfa_id = if let Some(&id) = dfa_states.get(&next_nfa_states_sorted) {
                    id
                } else {
                    let id = next_dfa_state;
                    dfa_states.insert(next_nfa_states_sorted.clone(), id);
                    queue.push_back(next_nfa_states_sorted);
                    next_dfa_state += 1;
                    id
                };

                dfa_edges.push((current_dfa_id, next_dfa_id, symbol.clone()));
            }
        }

        let mut graph = DiGraph::new();

        for i in 0..next_dfa_state {
            graph.add_node(i);
        }

        for (from, to, symbol) in dfa_edges {
            graph.add_edge(
                NodeIndex::new(from as usize),
                NodeIndex::new(to as usize),
                symbol,
            );
        }

        Self {
            initial_state: 0,
            final_states: dfa_final_states,
            graph,
            alphabet: nfa.alphabet,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dfa_accepts() {
        let raw = RawAutomaton {
            initial_state: 0,
            final_states: vec![1],
            alphabet: vec!["a".to_string(), "b".to_string()],
            edges: vec![(0, 1, "a".to_string()), (1, 0, "b".to_string())],
        };
        let dfa = Dfa::from(raw);
        assert!(dfa.run("a"));
        assert!(dfa.run("aba"));
        assert!(dfa.run("ababa"));
    }

    #[test]
    fn test_dfa_rejects() {
        let raw = RawAutomaton {
            initial_state: 0,
            final_states: vec![1],
            alphabet: vec!["a".to_string(), "b".to_string()],
            edges: vec![(0, 1, "a".to_string()), (1, 0, "b".to_string())],
        };
        let dfa = Dfa::from(raw);
        assert!(!dfa.run(""));
        assert!(!dfa.run("b"));
        assert!(!dfa.run("ab"));
        assert!(!dfa.run("aa"));
    }

    #[test]
    fn test_dfa_multi_char_symbol() {
        let raw = RawAutomaton {
            initial_state: 0,
            final_states: vec![1],
            alphabet: vec!["ab".to_string(), "c".to_string()],
            edges: vec![(0, 1, "ab".to_string()), (1, 0, "c".to_string())],
        };
        let dfa = Dfa::from(raw);
        assert!(dfa.run("ab"));
        assert!(dfa.run("abcab"));
        assert!(!dfa.run("a"));
        assert!(!dfa.run("b"));
        assert!(!dfa.run("abc"));
    }

    #[test]
    fn test_from_lambda_nfa() {
        let raw = RawAutomaton {
            initial_state: 0,
            final_states: vec![2],
            alphabet: vec!["a".to_string(), "b".to_string()],
            edges: vec![
                (0, 0, "a".to_string()),
                (0, 1, String::new()),
                (1, 2, "b".to_string()),
            ],
        };
        let nfa = LambdaNfa::from(raw);
        let dfa = Dfa::from(nfa);
        assert!(dfa.run("b"));
        assert!(dfa.run("ab"));
        assert!(dfa.run("aaab"));
        assert!(!dfa.run("a"));
        assert!(!dfa.run(""));
    }

    #[test]
    fn test_from_nfa() {
        let raw = RawAutomaton {
            initial_state: 0,
            final_states: vec![2],
            alphabet: vec!["a".to_string(), "b".to_string()],
            edges: vec![
                (0, 0, "a".to_string()),
                (0, 1, "a".to_string()),
                (1, 2, "b".to_string()),
            ],
        };
        let nfa = Nfa::from(raw);
        let dfa = Dfa::from(nfa);
        assert!(dfa.run("ab"));
        assert!(dfa.run("aab"));
        assert!(!dfa.run("a"));
        assert!(!dfa.run("b"));
    }

    #[test]
    fn test_from_lambda_dfa() {
        let raw = RawAutomaton {
            initial_state: 0,
            final_states: vec![2],
            alphabet: vec!["a".to_string(), "b".to_string()],
            edges: vec![(0, 1, "a".to_string()), (1, 2, String::new())],
        };
        let ldfa = LambdaDfa::from(raw);
        let dfa = Dfa::from(ldfa);
        assert!(dfa.run("a"));
        assert!(!dfa.run(""));
        assert!(!dfa.run("b"));
    }

    #[test]
    fn test_dfa_minimize() {
        // DFA for even number of 'a's, but with redundant states
        let raw = RawAutomaton {
            initial_state: 0,
            final_states: vec![0, 2],
            alphabet: vec!["a".to_string()],
            edges: vec![
                (0, 1, "a".to_string()),
                (1, 2, "a".to_string()),
                (2, 3, "a".to_string()),
                (3, 0, "a".to_string()),
            ],
        };
        let dfa = Dfa::from(raw);
        let minimized = dfa.minimize();
        // 0 and 2 are equivalent, 1 and 3 are equivalent.
        // Minimized DFA should have 2 states.
        assert!(minimized.graph.node_count() <= 2);
        assert!(minimized.run(""));
        assert!(!minimized.run("a"));
        assert!(minimized.run("aa"));
        assert!(!minimized.run("aaa"));
        assert!(minimized.run("aaaa"));
    }
}
