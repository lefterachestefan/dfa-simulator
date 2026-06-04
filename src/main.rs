use automaton_simulator::prelude::*;
use automaton_simulator::cfg::Cfg;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. CFG Example
    if let Ok(cfg) = Cfg::try_read_from_file("cfg_input_2.txt") {
        println!("--- CFG Examples ---");
        println!("CFG: {:?}", cfg);

        println!("\nEx 1: Generate words of length 4");
        let mut words: Vec<_> = cfg.generate_words(4).into_iter().collect();
        words.sort();
        println!("Words of length 4: {:?}", words);

        println!("\nEx 2: Transform to CNF");
        let cnf = cfg.to_cnf();

        println!("\nEx 3: CYK Algorithm");
        let test_words = vec!["aabb", "abab", "ab", "ba", "baba", "aaabbb", "aab"];
        for word in test_words {
            println!("  \"{}\": {}", word, cnf.cyk(word));
        }
    }

    println!("\n--- Genetic Agent to DFA Translation ---");
    // Path to a generated agent from genetic-algo3
    let agent_path = "../genetic-algo3/algo/pol1";
    
    if let Some(agent) = Agent::load_from_file(agent_path) {
        println!("Loaded agent from {}", agent_path);
        
        let alphabet = vec!["0".to_string(), "1".to_string()];
        let dfa = agent.to_dfa(&alphabet, 20);
        
        println!("Generated DFA with {} states", dfa.graph.node_count());
        println!("Final states: {:?}", dfa.final_states);
        
        // Save to DOT/PNG for visualization
        dfa.save_png("genetic_dfa.png")?;
        println!("Saved DFA visualization to genetic_dfa.png");

        // Test the DFA
        let test_words = vec!["0", "1", "00", "01", "10", "11", "101", "110"];
        for word in test_words {
            println!("  \"{}\": {}", word, dfa.run(word));
        }
    } else {
        println!("Could not load agent from {}. (This is expected if genetic-algo3 hasn't been run yet)", agent_path);
    }

    Ok(())
}
