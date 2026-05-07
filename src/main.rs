use automaton_simulator::pda::{AcceptanceCondition, Pda, PdaTransition};
use petgraph::graph::DiGraph;

// fn main() -> Result<(), Box<dyn std::error::Error>> {
//     let dfa = Dfa::try_read_from_file("dfa2.txt")?;
//     dfa.save_png("dfa.png")?;
//     let dfa_minim = dfa.minimize();
//     dfa_minim.save_png("dfa_minim.png")?;
//     println!("{}", dfa.run("abaa"));
//     println!("{}", dfa.run("abbb"));
//     println!("{}", dfa.run("aaaa"));
//     println!("----------");
//
//     let nfa = Nfa::try_read_from_file("nfa2.txt")?;
//     nfa.save_png("nfa.png")?;
//     println!("{}", nfa.run("abc"));
//     println!("{}", nfa.run("aaabbb"));
//     println!("{}", nfa.run("aaaccc"));
//     println!("{}", nfa.run("aaa"));
//     println!("----------");
//
//     let lnfa = LambdaNfa::try_read_from_file("lambda2.txt")?;
//     lnfa.save_png("lambda_nfa.png")?;
//     println!("{}", lnfa.run("abc"));
//     println!("{}", lnfa.run("cb"));
//     println!("{}", lnfa.run("aaa"));
//     println!("{}", lnfa.run("abb"));
//     println!("----------");
//
//     let transformed = Dfa::from(lnfa);
//     transformed.save_png("transformed.png")?;
//
//     Ok(())
// }

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut graph = DiGraph::new();
    let q0 = graph.add_node(0);
    let q1 = graph.add_node(1);
    let q2 = graph.add_node(2);

    // q0 -> q0
    graph.add_edge(
        q0,
        q0,
        PdaTransition {
            input_symbol: Some('a'),
            pop_symbol: Some('Z'),
            push_symbols: vec!['A', 'Z'],
        },
    );
    graph.add_edge(
        q0,
        q0,
        PdaTransition {
            input_symbol: Some('b'),
            pop_symbol: Some('Z'),
            push_symbols: vec!['B', 'Z'],
        },
    );
    graph.add_edge(
        q0,
        q0,
        PdaTransition {
            input_symbol: Some('a'),
            pop_symbol: Some('A'),
            push_symbols: vec!['A', 'A'],
        },
    );
    graph.add_edge(
        q0,
        q0,
        PdaTransition {
            input_symbol: Some('b'),
            pop_symbol: Some('B'),
            push_symbols: vec!['B', 'B'],
        },
    );
    graph.add_edge(
        q0,
        q0,
        PdaTransition {
            input_symbol: Some('a'),
            pop_symbol: Some('B'),
            push_symbols: vec!['A', 'B'],
        },
    );
    graph.add_edge(
        q0,
        q0,
        PdaTransition {
            input_symbol: Some('b'),
            pop_symbol: Some('A'),
            push_symbols: vec!['B', 'A'],
        },
    );

    // q0 -> q1
    graph.add_edge(
        q0,
        q1,
        PdaTransition {
            input_symbol: Some('a'),
            pop_symbol: Some('A'),
            push_symbols: vec![],
        },
    );
    graph.add_edge(
        q0,
        q1,
        PdaTransition {
            input_symbol: Some('b'),
            pop_symbol: Some('B'),
            push_symbols: vec![],
        },
    );

    // q1 -> q1
    graph.add_edge(
        q1,
        q1,
        PdaTransition {
            input_symbol: Some('a'),
            pop_symbol: Some('A'),
            push_symbols: vec![],
        },
    );
    graph.add_edge(
        q1,
        q1,
        PdaTransition {
            input_symbol: Some('b'),
            pop_symbol: Some('B'),
            push_symbols: vec![],
        },
    );

    // q1 -> q2
    graph.add_edge(
        q1,
        q2,
        PdaTransition {
            input_symbol: None,
            pop_symbol: Some('Z'),
            push_symbols: vec![],
        },
    );

    let pda = Pda {
        initial_state: 0,
        initial_stack_symbol: Some('Z'),
        final_states: vec![2],
        graph,
        acceptance_condition: AcceptanceCondition::Both,
    };

    let test_strings = vec!["ab", "ba", "aabaa", "aa", "abba", ""];
    for s in test_strings {
        println!("\"{}\": {}", s, pda.run(s));
    }

    Ok(())
}
