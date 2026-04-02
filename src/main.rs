use automaton_simulator::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dfa = Dfa::try_read_from_file("dfa.txt")?;
    dfa.save_png("dfa.png")?;

    let nfa = Nfa::try_read_from_file("nfa.txt")?;
    nfa.save_png("nfa.png")?;

    let lnfa = LambdaNfa::try_read_from_file("lambda_nfa.txt")?;
    lnfa.save_png("lambda_nfa.png")?;

    let dfa = Dfa::try_read_from_file("dfa.txt")?;
    let minimized = dfa.minimize();
    minimized.save_png("minimized_dfa.png")?;

    Ok(())
}
