use automaton_simulator::prelude::*;
#[test]
fn test_load_all_files() {
    assert!(Dfa::try_read_from_file("dfa.txt").is_ok());
    assert!(Nfa::try_read_from_file("nfa.txt").is_ok());
    assert!(LambdaDfa::try_read_from_file("lambda_dfa.txt").is_ok());
    assert!(LambdaNfa::try_read_from_file("lambda_nfa.txt").is_ok());
}

#[test]
fn test_visualize() {
    let dfa = Dfa::try_read_from_file("dfa.txt").unwrap();
    let dot = dfa.to_dot();
    assert!(dot.contains("digraph"));
    assert!(dot.contains("rankdir=LR"));

    let result = dfa.save_png("test_output.png");
    assert!(result.is_ok());

    // Clean up
    let _ = std::fs::remove_file("test_output.png");
}
