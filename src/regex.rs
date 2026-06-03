use crate::lambda_nfa::LambdaNfa;
use petgraph::{graph::DiGraph, visit::EdgeRef};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, PartialEq, Eq)]
enum Token {
    Char(char),
    Union,
    Concat,
    Star,
    Plus,
    LParen,
    RParen,
}

/// Helper for regex to `LambdaNfa` conversion
pub struct RegexConverter;

impl RegexConverter {
    const fn priority(token: &Token) -> i32 {
        match token {
            Token::LParen => 4,
            Token::Star | Token::Plus => 3,
            Token::Concat => 2,
            Token::Union => 1,
            _ => 0,
        }
    }

    fn preprocess(regex: &str) -> String {
        let mut result = String::new();
        let chars: Vec<char> = regex.chars().filter(|c| !c.is_whitespace()).collect();
        for i in 0..chars.len() {
            let c1 = chars[i];
            result.push(c1);
            if i + 1 < chars.len() {
                let c2 = chars[i + 1];
                let is_operand = |c: char| c.is_alphanumeric() || c == 'λ' || c == 'ε';
                let is_unary = |c: char| c == '*' || c == '+';

                if (is_operand(c1) || is_unary(c1) || c1 == ')') && (is_operand(c2) || c2 == '(') {
                    result.push('.');
                }
            }
        }
        result
    }

    fn tokenize(regex: &str) -> Vec<Token> {
        regex
            .chars()
            .map(|c| match c {
                '|' => Token::Union,
                '.' => Token::Concat,
                '*' => Token::Star,
                '+' => Token::Plus,
                '(' => Token::LParen,
                ')' => Token::RParen,
                _ => Token::Char(c),
            })
            .collect()
    }

    fn to_postfix(tokens: Vec<Token>) -> Vec<Token> {
        let mut output = Vec::new();
        let mut stack = Vec::new();

        for token in tokens {
            match token {
                Token::Char(_) => output.push(token),
                Token::LParen => stack.push(token),
                Token::RParen => {
                    while let Some(top) = stack.pop() {
                        if top == Token::LParen {
                            break;
                        }
                        output.push(top);
                    }
                }
                _ => {
                    while let Some(top) = stack.last() {
                        if *top == Token::LParen || Self::priority(&token) > Self::priority(top) {
                            break;
                        }
                        output.push(stack.pop().expect("stack not empty"));
                    }
                    stack.push(token);
                }
            }
        }
        while let Some(top) = stack.pop() {
            output.push(top);
        }
        output
    }

    #[must_use]
    pub fn to_lambda_nfa(regex: &str) -> LambdaNfa {
        let preprocessed = Self::preprocess(regex);
        let tokens = Self::tokenize(&preprocessed);
        let postfix = Self::to_postfix(tokens);

        let mut graph = DiGraph::new();
        let mut stack = Vec::new();
        let mut alphabet = HashSet::new();

        for token in postfix {
            match token {
                Token::Char(c) => {
                    let s = graph.add_node(0);
                    let e = graph.add_node(0);
                    let label = if c == 'λ' || c == 'ε' {
                        String::new()
                    } else {
                        alphabet.insert(c.to_string());
                        c.to_string()
                    };
                    graph.add_edge(s, e, label);
                    stack.push((s, e));
                }
                Token::Union => {
                    let (s2, e2) = stack.pop().expect("stack not empty");
                    let (s1, e1) = stack.pop().expect("stack not empty");
                    let s = graph.add_node(0);
                    let e = graph.add_node(0);
                    graph.add_edge(s, s1, String::new());
                    graph.add_edge(s, s2, String::new());
                    graph.add_edge(e1, e, String::new());
                    graph.add_edge(e2, e, String::new());
                    stack.push((s, e));
                }
                Token::Concat => {
                    let (s2, e2) = stack.pop().expect("stack not empty");
                    let (s1, e1) = stack.pop().expect("stack not empty");
                    graph.add_edge(e1, s2, String::new());
                    stack.push((s1, e2));
                }
                Token::Star => {
                    let (s1, e1) = stack.pop().expect("stack not empty");
                    let s = graph.add_node(0);
                    let e = graph.add_node(0);
                    graph.add_edge(s, s1, String::new());
                    graph.add_edge(s, e, String::new());
                    graph.add_edge(e1, s1, String::new());
                    graph.add_edge(e1, e, String::new());
                    stack.push((s, e));
                }
                Token::Plus => {
                    let (s1, e1) = stack.pop().expect("stack not empty");
                    let s = graph.add_node(0);
                    let e = graph.add_node(0);
                    graph.add_edge(s, s1, String::new());
                    graph.add_edge(e1, s1, String::new());
                    graph.add_edge(e1, e, String::new());
                    stack.push((s, e));
                }
                _ => {}
            }
        }

        let (start, end) = stack.pop().expect("Invalid regex");

        let mut new_graph = DiGraph::new();
        let mut node_map = HashMap::new();
        for node in graph.node_indices() {
            node_map.insert(node, new_graph.add_node(node.index() as u32));
        }
        for edge in graph.edge_references() {
            new_graph.add_edge(
                node_map[&edge.source()],
                node_map[&edge.target()],
                edge.weight().clone(),
            );
        }

        LambdaNfa {
            initial_state: node_map[&start].index() as u32,
            final_states: vec![node_map[&end].index() as u32],
            graph: new_graph,
            alphabet: alphabet.into_iter().collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Automaton;

    #[test]
    fn test_regex_to_nfa() {
        let nfa = RegexConverter::to_lambda_nfa("(a|b)*c");
        assert!(nfa.run("c"));
        assert!(nfa.run("ac"));
        assert!(nfa.run("bc"));
        assert!(nfa.run("aaabbbc"));
        assert!(!nfa.run(""));
        assert!(!nfa.run("a"));
        assert!(!nfa.run("b"));
        assert!(!nfa.run("ca"));
    }
}
