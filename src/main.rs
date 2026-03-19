use automaton_simulator::prelude::*;

fn main() -> Result<(), ReadGraphError> {
    let dfa = Dfa::try_read_from_file("dfa.txt")?;
    dbg!(&dfa);
    println!("aa: {}", dfa.run("aa"));
    println!("ab: {}", dfa.run("ab"));
    Ok(())
}
