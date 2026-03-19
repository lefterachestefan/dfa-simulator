use crate::raw_automaton::RawAutomaton;
use petgraph::Direction;
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::EdgeRef;
use std::collections::HashSet;

/// Nondeterministic Finite Automaton
#[derive(Debug, Clone)]
pub struct Nfa {
    initial_state: u32,
    final_states: Vec<u32>,
    graph: DiGraph<u32, String>,
    alphabet: Vec<String>,
}

impl From<RawAutomaton> for Nfa {
    fn from(raw: RawAutomaton) -> Self {
        for edge in &raw.edges {
            assert!(!edge.2.is_empty(), "NFA cannot have lambda transitions");
            assert!(raw.alphabet.contains(&edge.2));
        }
        Self {
            initial_state: raw.initial_state,
            final_states: raw.final_states,
            graph: DiGraph::from_edges(raw.edges),
            alphabet: raw.alphabet,
        }
    }
}

impl Nfa {
    /// Runs the NFA on the given input string.
    pub fn run(&self, input: impl AsRef<str>) -> bool {
        let mut current_states = HashSet::new();
        current_states.insert(NodeIndex::new(self.initial_state as usize));
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
            current_states = next_states;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nfa_accepts() {
        let raw = RawAutomaton {
            initial_state: 0,
            final_states: vec![2],
            alphabet: vec!["a".to_string(), "b".to_string()],
            edges: vec![
                (0, 0, "a".to_string()),
                (0, 0, "b".to_string()),
                (0, 1, "a".to_string()),
                (1, 2, "b".to_string()),
            ],
        };
        let nfa = Nfa::from(raw);
        assert!(nfa.run("ab"));
        assert!(nfa.run("aab"));
        assert!(nfa.run("bab"));
        assert!(nfa.run("aaab"));
    }

    #[test]
    fn test_nfa_rejects() {
        let raw = RawAutomaton {
            initial_state: 0,
            final_states: vec![2],
            alphabet: vec!["a".to_string(), "b".to_string()],
            edges: vec![
                (0, 0, "a".to_string()),
                (0, 0, "b".to_string()),
                (0, 1, "a".to_string()),
                (1, 2, "b".to_string()),
            ],
        };
        let nfa = Nfa::from(raw);
        assert!(!nfa.run(""));
        assert!(!nfa.run("a"));
        assert!(!nfa.run("b"));
        assert!(!nfa.run("aba"));
    }
}
