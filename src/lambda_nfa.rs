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
use std::collections::HashSet;

/// Nondeterministic Finite Automaton with Lambda transitions
#[derive(Debug, Clone)]
pub struct LambdaNfa {
    pub(crate) initial_state: u32,
    pub(crate) final_states: Vec<u32>,
    pub(crate) graph: DiGraph<u32, String>,
    pub(crate) alphabet: Vec<String>,
}

impl Automaton for LambdaNfa {
    /// Runs the Lambda-NFA on the given input string.
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
    /// Minimizes the `LambdaNfa` by converting to DFA, minimizing the DFA, and converting back.
    fn minimize(&self) -> Self {
        Self::from(Dfa::from(self.clone()).minimize())
    }
}

impl From<Dfa> for LambdaNfa {
    fn from(dfa: Dfa) -> Self {
        Self {
            initial_state: dfa.initial_state,
            final_states: dfa.final_states,
            graph: dfa.graph,
            alphabet: dfa.alphabet,
        }
    }
}

impl From<Nfa> for LambdaNfa {
    fn from(nfa: Nfa) -> Self {
        Self {
            initial_state: nfa.initial_state,
            final_states: nfa.final_states,
            graph: nfa.graph,
            alphabet: nfa.alphabet,
        }
    }
}

impl From<LambdaDfa> for LambdaNfa {
    fn from(ldfa: LambdaDfa) -> Self {
        Self {
            initial_state: ldfa.initial_state,
            final_states: ldfa.final_states,
            graph: ldfa.graph,
            alphabet: ldfa.alphabet,
        }
    }
}

impl From<RawAutomaton> for LambdaNfa {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lambda_nfa_accepts() {
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
        let nfa = LambdaNfa::from(raw);
        assert!(nfa.run(""));
        assert!(nfa.run("a"));
        assert!(nfa.run("b"));
        assert!(nfa.run("c"));
        assert!(nfa.run("aaabbbccc"));
        assert!(nfa.run("abc"));
    }

    #[test]
    fn test_lambda_nfa_rejects() {
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
        let nfa = LambdaNfa::from(raw);
        assert!(!nfa.run("ba"));
        assert!(!nfa.run("cb"));
        assert!(!nfa.run("ca"));
    }

    #[test]
    fn another2() {
        let raw = RawAutomaton {
            initial_state: 0,
            final_states: vec![5, 6],
            alphabet: vec!["a".to_string(), "b".to_string(), "c".to_string()],
            edges: vec![
                (0, 1, String::new()),
                (0, 2, String::new()),
                (1, 3, "a".into()),
                (2, 4, "a".into()),
                (3, 3, "a".into()),
                (3, 5, "a".into()),
                (4, 4, "a".into()),
                (4, 6, "c".into()),
                (5, 5, "b".into()),
                (6, 6, "c".into()),
            ],
        };
        let nfa = LambdaNfa::from(raw);
        assert!(!nfa.run("ab"));
        assert!(!nfa.run("abb"));
        assert!(!nfa.run("abc"));
        assert!(nfa.run("aaaabbbbbb"));
        assert!(nfa.run("aaaacccccc"));
    }
}
