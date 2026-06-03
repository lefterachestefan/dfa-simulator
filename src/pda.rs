use petgraph::{
    Direction,
    graph::{DiGraph, NodeIndex},
    visit::EdgeRef,
};
use std::collections::HashSet;

/// Acceptance condition for PDA
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AcceptanceCondition {
    /// Accept if the automaton is in a final state after consuming the input.
    FinalState,
    /// Accept if the stack is empty after consuming the input.
    EmptyStack,
    /// Accept if both conditions are met.
    Both,
}

/// Transition in a PDA
#[derive(Debug, Clone)]
pub struct PdaTransition {
    /// The input symbol to consume. `None` represents a lambda transition.
    pub input_symbol: Option<char>,
    /// The symbol to pop from the stack. `None` represents a lambda transition (no pop).
    pub pop_symbol: Option<char>,
    /// The symbols to push onto the stack (in order, first symbol becomes new top).
    pub push_symbols: Vec<char>,
}

/// Push-down Automaton
#[derive(Debug, Clone)]
pub struct Pda {
    pub initial_state: u32,
    pub initial_stack_symbol: Option<char>,
    pub final_states: Vec<u32>,
    pub graph: DiGraph<u32, PdaTransition>,
    pub acceptance_condition: AcceptanceCondition,
}

const fn is_lambda(symbol: Option<char>) -> bool {
    matches!(symbol, None | Some('λ') | Some('ε'))
}

impl Pda {
    /// Runs the PDA on the given input string.
    #[must_use]
    pub fn run(&self, input: &str) -> bool {
        let mut initial_stack = Vec::new();
        if !is_lambda(self.initial_stack_symbol) {
            if let Some(s) = self.initial_stack_symbol {
                initial_stack.push(s);
            }
        }

        let mut current_configs = HashSet::new();
        current_configs.insert((NodeIndex::new(self.initial_state as usize), initial_stack));

        current_configs = self.epsilon_closure(current_configs);

        for c in input.chars() {
            let mut next_configs = HashSet::new();
            for (state, stack) in current_configs {
                for edge in self.graph.edges_directed(state, Direction::Outgoing) {
                    let trans = edge.weight();
                    if trans.input_symbol == Some(c) {
                        if is_lambda(trans.pop_symbol) {
                            let mut next_stack = stack.clone();
                            for &s in trans.push_symbols.iter().rev() {
                                next_stack.push(s);
                            }
                            next_configs.insert((edge.target(), next_stack));
                        } else if let Some(top) = stack.last() {
                            if Some(*top) == trans.pop_symbol {
                                let mut next_stack = stack.clone();
                                next_stack.pop();
                                for &s in trans.push_symbols.iter().rev() {
                                    next_stack.push(s);
                                }
                                next_configs.insert((edge.target(), next_stack));
                            }
                        }
                    }
                }
            }
            current_configs = self.epsilon_closure(next_configs);
            if current_configs.is_empty() {
                return false;
            }
        }

        current_configs.iter().any(|(state, stack)| {
            let is_final = self.final_states.contains(&(state.index() as u32));
            let is_empty = stack.is_empty();
            match self.acceptance_condition {
                AcceptanceCondition::FinalState => is_final,
                AcceptanceCondition::EmptyStack => is_empty,
                AcceptanceCondition::Both => is_final && is_empty,
            }
        })
    }

    fn epsilon_closure(
        &self,
        mut configs: HashSet<(NodeIndex, Vec<char>)>,
    ) -> HashSet<(NodeIndex, Vec<char>)> {
        let mut stack: Vec<(NodeIndex, Vec<char>)> = configs.iter().cloned().collect();
        while let Some((state, current_stack)) = stack.pop() {
            for edge in self.graph.edges_directed(state, Direction::Outgoing) {
                let trans = edge.weight();
                if is_lambda(trans.input_symbol) {
                    if is_lambda(trans.pop_symbol) {
                        let mut next_stack = current_stack.clone();
                        for &s in trans.push_symbols.iter().rev() {
                            next_stack.push(s);
                        }
                        let next_config = (edge.target(), next_stack);
                        if configs.insert(next_config.clone()) {
                            stack.push(next_config);
                        }
                    } else if let Some(top) = current_stack.last() {
                        if Some(*top) == trans.pop_symbol {
                            let mut next_stack = current_stack.clone();
                            next_stack.pop();
                            for &s in trans.push_symbols.iter().rev() {
                                next_stack.push(s);
                            }
                            let next_config = (edge.target(), next_stack);
                            if configs.insert(next_config.clone()) {
                                stack.push(next_config);
                            }
                        }
                    }
                }
            }
        }
        configs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_an_bn() {
        let mut graph = DiGraph::new();
        let q0 = graph.add_node(0);
        let q1 = graph.add_node(1);
        let q2 = graph.add_node(2);
        let q3 = graph.add_node(3);

        graph.add_edge(
            q0,
            q1,
            PdaTransition {
                input_symbol: Some('a'),
                pop_symbol: Some('Z'),
                push_symbols: vec!['A', 'Z'],
            },
        );
        graph.add_edge(
            q1,
            q1,
            PdaTransition {
                input_symbol: Some('a'),
                pop_symbol: Some('A'),
                push_symbols: vec!['A', 'A'],
            },
        );
        graph.add_edge(
            q1,
            q2,
            PdaTransition {
                input_symbol: Some('b'),
                pop_symbol: Some('A'),
                push_symbols: vec![],
            },
        );
        graph.add_edge(
            q2,
            q2,
            PdaTransition {
                input_symbol: Some('b'),
                pop_symbol: Some('A'),
                push_symbols: vec![],
            },
        );
        graph.add_edge(
            q2,
            q3,
            PdaTransition {
                input_symbol: None,
                pop_symbol: Some('Z'),
                push_symbols: vec!['Z'],
            },
        );
        graph.add_edge(
            q0,
            q3,
            PdaTransition {
                input_symbol: None,
                pop_symbol: Some('Z'),
                push_symbols: vec!['Z'],
            },
        );

        let pda = Pda {
            initial_state: 0,
            initial_stack_symbol: Some('Z'),
            final_states: vec![3],
            graph,
            acceptance_condition: AcceptanceCondition::FinalState,
        };

        assert!(pda.run(""));
        assert!(pda.run("ab"));
        assert!(pda.run("aabb"));
        assert!(pda.run("aaabbb"));
        assert!(!pda.run("a"));
        assert!(!pda.run("b"));
        assert!(!pda.run("ba"));
        assert!(!pda.run("abb"));
        assert!(!pda.run("aab"));
    }

    #[test]
    fn test_lambda_pop() {
        let mut graph = DiGraph::new();
        let q0 = graph.add_node(0);
        let q1 = graph.add_node(1);

        graph.add_edge(
            q0,
            q1,
            PdaTransition {
                input_symbol: Some('a'),
                pop_symbol: None, // Lambda pop
                push_symbols: vec!['A'],
            },
        );

        let pda = Pda {
            initial_state: 0,
            initial_stack_symbol: None, // Start empty
            final_states: vec![1],
            graph,
            acceptance_condition: AcceptanceCondition::FinalState,
        };

        assert!(pda.run("a"));
        assert!(!pda.run(""));
    }

    #[test]
    fn test_lambda_literals() {
        let mut graph = DiGraph::new();
        let q0 = graph.add_node(0);
        let q1 = graph.add_node(1);

        graph.add_edge(
            q0,
            q1,
            PdaTransition {
                input_symbol: Some('λ'),
                pop_symbol: Some('ε'),
                push_symbols: vec![],
            },
        );

        let pda = Pda {
            initial_state: 0,
            initial_stack_symbol: Some('Z'),
            final_states: vec![1],
            graph,
            acceptance_condition: AcceptanceCondition::FinalState,
        };

        assert!(pda.run(""));
    }

    #[test]
    fn test_palindrome_even() {
        let mut graph = DiGraph::new();
        let q0 = graph.add_node(0);
        let q1 = graph.add_node(1);
        let q2 = graph.add_node(2);

        // Push 'A' for 'a', 'B' for 'b'
        for (c, s) in [('a', 'A'), ('b', 'B')] {
            graph.add_edge(
                q0,
                q0,
                PdaTransition {
                    input_symbol: Some(c),
                    pop_symbol: None,
                    push_symbols: vec![s],
                },
            );
        }

        // Guess the middle (non-determinism)
        graph.add_edge(
            q0,
            q1,
            PdaTransition {
                input_symbol: None,
                pop_symbol: None,
                push_symbols: vec![],
            },
        );

        // Pop 'A' for 'a', 'B' for 'b'
        for (c, s) in [('a', 'A'), ('b', 'B')] {
            graph.add_edge(
                q1,
                q1,
                PdaTransition {
                    input_symbol: Some(c),
                    pop_symbol: Some(s),
                    push_symbols: vec![],
                },
            );
        }

        // Accept if stack is empty
        graph.add_edge(
            q1,
            q2,
            PdaTransition {
                input_symbol: None,
                pop_symbol: None,
                push_symbols: vec![],
            },
        );

        let pda = Pda {
            initial_state: 0,
            initial_stack_symbol: None,
            final_states: vec![2],
            graph,
            acceptance_condition: AcceptanceCondition::Both,
        };

        assert!(pda.run(""));
        assert!(pda.run("aa"));
        assert!(pda.run("abba"));
        assert!(pda.run("baab"));
        assert!(pda.run("aaaa"));
        assert!(!pda.run("a"));
        assert!(!pda.run("ab"));
        assert!(!pda.run("aba"));
    }
}
