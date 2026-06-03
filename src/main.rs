use automaton_simulator::cfg::Cfg;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = Cfg::try_read_from_file("cfg_input_2.txt")?;
    println!("CFG: {:?}", cfg);

    println!("\nEx 1: Generate words of length 4");
    let mut words: Vec<_> = cfg.generate_words(4).into_iter().collect();
    words.sort();
    println!("Words of length 4: {:?}", words);

    println!("\nEx 2: Transform to CNF");
    let cnf = cfg.to_cnf();
    // println!("CNF Productions:");
    // for (head, prods) in &cnf.productions {
    //     println!("  {} -> {:?}", head, prods);
    // }

    println!("\nEx 3: CYK Algorithm");
    let test_words = vec!["aabb", "abab", "ab", "ba", "baba", "aaabbb", "aab"];
    for word in test_words {
        println!("  \"{}\": {}", word, cnf.cyk(word));
    }

    Ok(())
}
