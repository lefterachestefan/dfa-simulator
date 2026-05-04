use automaton_simulator::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dfa = Dfa::try_read_from_file("dfa2.txt")?;
    dfa.save_png("dfa.png")?;
    let dfa_minim = dfa.minimize();
    dfa_minim.save_png("dfa_minim.png")?;
    println!("{}", dfa.run("abaa"));
    println!("{}", dfa.run("abbb"));
    println!("{}", dfa.run("aaaa"));
    println!("----------");

    let nfa = Nfa::try_read_from_file("nfa2.txt")?;
    nfa.save_png("nfa.png")?;
    println!("{}", nfa.run("abc"));
    println!("{}", nfa.run("aaabbb"));
    println!("{}", nfa.run("aaaccc"));
    println!("{}", nfa.run("aaa"));
    println!("----------");

    let lnfa = LambdaNfa::try_read_from_file("lambda2.txt")?;
    lnfa.save_png("lambda_nfa.png")?;
    println!("{}", lnfa.run("abc"));
    println!("{}", lnfa.run("cb"));
    println!("{}", lnfa.run("aaa"));
    println!("{}", lnfa.run("abb"));
    println!("----------");

    let transformed = Dfa::from(lnfa);
    transformed.save_png("transformed.png")?;

    Ok(())
}
