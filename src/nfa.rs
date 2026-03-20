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

/// Nondeterministic Finite Automaton
#[derive(Debug, Clone)]
pub struct Nfa {
    pub(crate) initial_state: u32,
    pub(crate) final_states: Vec<u32>,
    pub(crate) graph: DiGraph<u32, String>,
    pub(crate) alphabet: Vec<String>,
}

impl Automaton for Nfa {
    /// Runs the NFA on the given input string.
    fn run(&self, input: impl AsRef<str>) -> bool {
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

    /// Minimizes the NFA by converting to DFA, minimizing the DFA, and converting back.
    /// Note: This is optimal in terms of equivalent DFA states, but NFA-to-DFA conversion can be exponential.
    fn minimize(&self) -> Self {
        Self::from(Dfa::from(self.clone()).minimize())
    }
}

impl From<Dfa> for Nfa {
    fn from(dfa: Dfa) -> Self {
        Self {
            initial_state: dfa.initial_state,
            final_states: dfa.final_states,
            graph: dfa.graph,
            alphabet: dfa.alphabet,
        }
    }
}

impl From<LambdaNfa> for Nfa {
    fn from(nfa: LambdaNfa) -> Self {
        let mut edges = Vec::new();
        let mut final_states = HashSet::new();

        for node in nfa.graph.node_indices() {
            let mut s = HashSet::new();
            s.insert(node);
            let closure = advance_empty_word(&nfa.graph, &s);

            if closure.iter().any(|&idx| {
                nfa.final_states
                    .contains(&u32::try_from(idx.index()).unwrap_or(0))
            }) {
                final_states.insert(u32::try_from(node.index()).unwrap_or(0));
            }

            for symbol in &nfa.alphabet {
                let mut targets = HashSet::new();
                for &p in &closure {
                    for edge in nfa.graph.edges_directed(p, Direction::Outgoing) {
                        if edge.weight() == symbol {
                            targets.insert(edge.target());
                        }
                    }
                }

                let final_targets = advance_empty_word(&nfa.graph, &targets);
                for target in final_targets {
                    edges.push((
                        u32::try_from(node.index()).unwrap_or(0),
                        u32::try_from(target.index()).unwrap_or(0),
                        symbol.clone(),
                    ));
                }
            }
        }

        Self {
            initial_state: nfa.initial_state,
            final_states: final_states.into_iter().collect(),
            graph: DiGraph::from_edges(edges),
            alphabet: nfa.alphabet,
        }
    }
}

impl From<LambdaDfa> for Nfa {
    fn from(ldfa: LambdaDfa) -> Self {
        Self::from(LambdaNfa::from(ldfa))
    }
}

impl From<RawAutomaton> for Nfa {
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

    #[test]
    fn another() {
        let raw = RawAutomaton {
            initial_state: 0,
            final_states: vec![2, 4],
            alphabet: vec!["a".to_string(), "b".to_string(), "c".to_string()],
            edges: vec![
                (0, 1, "a".into()),
                (0, 3, "a".into()),
                (1, 1, "a".into()),
                (1, 2, "b".into()),
                (2, 2, "b".into()),
                (3, 3, "a".into()),
                (3, 4, "c".into()),
                (4, 4, "c".into()),
            ],
        };

        let nfa = Nfa::from(raw);

        assert!(nfa.run("ab"));
        assert!(nfa.run("abb"));
        assert!(!nfa.run("abc"));
        assert!(nfa.run("aaaabbbbbb"));
        assert!(nfa.run("aaaacccccc"));
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

        let lnfa = LambdaNfa::from(raw);
        let nfa = Nfa::from(lnfa);

        assert!(nfa.run("b"));
        assert!(nfa.run("ab"));
        assert!(nfa.run("aaab"));
        assert!(!nfa.run("a"));
    }
}
