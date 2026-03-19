use crate::Automaton;
use crate::prelude::{Dfa, LambdaNfa, Nfa};
use crate::raw_automaton::{RawAutomaton, advance_empty_word};
use petgraph::Direction;
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::EdgeRef;
use std::collections::HashSet;

/// Deterministic Finite Automaton with Lambda transitions
#[derive(Debug, Clone)]
pub struct LambdaDfa {
    pub(crate) initial_state: u32,
    pub(crate) final_states: Vec<u32>,
    pub(crate) graph: DiGraph<u32, String>,
    pub(crate) alphabet: Vec<String>,
}

impl Automaton for LambdaDfa {
    /// Runs the Lambda-DFA on the given input string.
    fn run(&self, input: impl AsRef<str>) -> bool {
        let mut current_states = HashSet::new();
        current_states.insert(NodeIndex::new(self.initial_state as usize));
        current_states = advance_empty_word(&self.graph, &current_states);
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
            current_states = advance_empty_word(&self.graph, &next_states);
            if current_states.is_empty() {
                return false;
            }
        }
        current_states.iter().any(|s| {
            self.final_states
                .contains(&u32::try_from(s.index()).unwrap_or(0))
        })
    }

    /// Minimizes the `LambdaDfa` by converting to DFA, minimizing the DFA, and converting back.
    fn minimize(&self) -> Self {
        Self::from(Dfa::from(self.clone()).minimize())
    }
}

impl From<Dfa> for LambdaDfa {
    fn from(dfa: Dfa) -> Self {
        Self {
            initial_state: dfa.initial_state,
            final_states: dfa.final_states,
            graph: dfa.graph,
            alphabet: dfa.alphabet,
        }
    }
}

impl From<Nfa> for LambdaDfa {
    fn from(nfa: Nfa) -> Self {
        Self {
            initial_state: nfa.initial_state,
            final_states: nfa.final_states,
            graph: nfa.graph,
            alphabet: nfa.alphabet,
        }
    }
}

impl From<LambdaNfa> for LambdaDfa {
    fn from(nfa: LambdaNfa) -> Self {
        Self {
            initial_state: nfa.initial_state,
            final_states: nfa.final_states,
            graph: nfa.graph,
            alphabet: nfa.alphabet,
        }
    }
}

impl From<RawAutomaton> for LambdaDfa {
    fn from(raw: RawAutomaton) -> Self {
        assert!(
            raw.edges
                .iter()
                .all(|edge| edge.2.is_empty() || raw.alphabet.contains(&edge.2))
        );

        Self {
            initial_state: raw.initial_state,
            final_states: raw.final_states,
            graph: DiGraph::from_edges(raw.edges),
            alphabet: raw.alphabet,
        }
    }
}

impl LambdaDfa {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lambda_dfa_accepts() {
        let raw = RawAutomaton {
            initial_state: 0,
            final_states: vec![2],
            alphabet: vec!["a".to_string(), "b".to_string(), "c".to_string()],
            edges: vec![
                (0, 0, "a".to_string()),
                (0, 1, String::new()),
                (1, 1, "b".to_string()),
                (1, 2, String::new()),
                (2, 2, "c".to_string()),
            ],
        };
        let nfa = LambdaDfa::from(raw);
        assert!(nfa.run(""));
        assert!(nfa.run("a"));
        assert!(nfa.run("b"));
        assert!(nfa.run("c"));
        assert!(nfa.run("aaabbbccc"));
        assert!(nfa.run("abc"));
    }

    #[test]
    fn test_lambda_dfa_rejects() {
        let raw = RawAutomaton {
            initial_state: 0,
            final_states: vec![2],
            alphabet: vec!["a".to_string(), "b".to_string(), "c".to_string()],
            edges: vec![
                (0, 0, "a".to_string()),
                (0, 1, String::new()),
                (1, 1, "b".to_string()),
                (1, 2, String::new()),
                (2, 2, "c".to_string()),
            ],
        };
        let nfa = LambdaDfa::from(raw);
        assert!(!nfa.run("ba"));
        assert!(!nfa.run("cb"));
        assert!(!nfa.run("ca"));
    }
}
