use crate::raw_automaton::RawAutomaton;
use petgraph::Direction;
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::EdgeRef;
use std::collections::HashSet;

/// Nondeterministic Finite Automaton with Lambda transitions
pub struct LambdaNfa {
    initial_state: u32,
    final_states: Vec<u32>,
    graph: DiGraph<u32, String>,
    alphabet: Vec<String>,
}

impl From<RawAutomaton> for LambdaNfa {
    fn from(raw: RawAutomaton) -> Self {
        for edge in &raw.edges {
            assert!(edge.2.is_empty() || raw.alphabet.contains(&edge.2));
        }
        Self {
            initial_state: raw.initial_state,
            final_states: raw.final_states,
            graph: DiGraph::from_edges(raw.edges),
            alphabet: raw.alphabet,
        }
    }
}

impl LambdaNfa {
    fn epsilon_closure(&self, states: &HashSet<NodeIndex>) -> HashSet<NodeIndex> {
        let mut closure = states.clone();
        let mut stack: Vec<NodeIndex> = states.iter().copied().collect();
        while let Some(current) = stack.pop() {
            for edge in self.graph.edges_directed(current, Direction::Outgoing) {
                if edge.weight().is_empty() && closure.insert(edge.target()) {
                    stack.push(edge.target());
                }
            }
        }
        closure
    }

    pub fn run(&self, input: impl AsRef<str>) -> bool {
        let mut current_states = HashSet::new();
        current_states.insert(NodeIndex::new(self.initial_state as usize));
        current_states = self.epsilon_closure(&current_states);
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
            let mut next_states = HashSet::new();
            for state in &current_states {
                for edge in self.graph.edges_directed(*state, Direction::Outgoing) {
                    if *edge.weight() == word {
                        next_states.insert(edge.target());
                    }
                }
            }
            current_states = self.epsilon_closure(&next_states);
            if current_states.is_empty() {
                return false;
            }
        }
        current_states.iter().any(|s| {
            self.final_states
                .contains(&u32::try_from(s.index()).unwrap_or(0))
        })
    }
}
