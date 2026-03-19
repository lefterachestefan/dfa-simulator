use automaton_simulator::prelude::*;

fn main() -> Result<(), ReadGraphError> {
    println!("--- DFA ---");
    let dfa = Dfa::try_read_from_file("dfa.txt")?;
    println!("DFA 'aa': {}", dfa.run("aa"));
    println!("DFA 'ab': {}", dfa.run("ab"));

    println!("--- NFA ---");
    let nfa = Nfa::try_read_from_file("nfa.txt")?;
    println!("NFA 'a': {}", nfa.run("a"));
    println!("NFA 'ab': {}", nfa.run("ab"));

    println!("--- Lambda-DFA ---");
    let lambda_deterministic = LambdaDfa::try_read_from_file("lambda_dfa.txt")?;
    println!("Lambda-DFA 'a': {}", lambda_deterministic.run("a"));
    println!("Lambda-DFA 'ab': {}", lambda_deterministic.run("ab"));

    println!("--- Lambda-NFA ---");
    let lambda_nondeterministic = LambdaNfa::try_read_from_file("lambda_nfa.txt")?;
    println!("Lambda-NFA 'a': {}", lambda_nondeterministic.run("a"));
    println!("Lambda-NFA 'ab': {}", lambda_nondeterministic.run("ab"));

    Ok(())
}
