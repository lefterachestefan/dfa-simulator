use dfa_simulator::prelude::*;

#[test]
fn test_load_all_files() {
    assert!(Dfa::try_read_from_file("dfa.txt").is_ok());
    assert!(Nfa::try_read_from_file("nfa.txt").is_ok());
    assert!(LambdaDfa::try_read_from_file("lambda_dfa.txt").is_ok());
    assert!(LambdaNfa::try_read_from_file("lambda_nfa.txt").is_ok());
}
