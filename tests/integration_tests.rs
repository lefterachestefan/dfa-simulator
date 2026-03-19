use dfa_simulator::dfa::Dfa;
use dfa_simulator::nfa::Nfa;
use dfa_simulator::lambda_dfa::LambdaDfa;
use dfa_simulator::lambda_nfa::LambdaNfa;
use dfa_simulator::raw_automaton::AutomatonFromFile;

#[test]
fn test_load_all_files() {
    assert!(Dfa::try_read_from_file("dfa.txt").is_ok());
    assert!(Nfa::try_read_from_file("nfa.txt").is_ok());
    assert!(LambdaDfa::try_read_from_file("lambda_dfa.txt").is_ok());
    assert!(LambdaNfa::try_read_from_file("lambda_nfa.txt").is_ok());
}
