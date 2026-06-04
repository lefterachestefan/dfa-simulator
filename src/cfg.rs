use std::collections::{HashSet, HashMap, VecDeque};
use std::fs;
use std::io;

#[derive(Debug, Clone)]
pub struct Cfg {
    pub non_terminals: HashSet<String>,
    pub terminals: HashSet<String>,
    pub start_symbol: String,
    pub productions: HashMap<String, Vec<Vec<String>>>,
}

impl Cfg {
    pub fn try_read_from_file(path: &str) -> io::Result<Self> {
        let content = fs::read_to_string(path)?;
        let mut lines = content.lines();

        let non_terminals = lines.next()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Missing non-terminals"))?
            .split_whitespace()
            .map(String::from)
            .collect();

        let terminals = lines.next()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Missing terminals"))?
            .split_whitespace()
            .map(String::from)
            .collect();

        let start_symbol = lines.next()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Missing start symbol"))?
            .trim()
            .to_string();

        let mut productions = HashMap::new();
        for line in lines {
            let line = line.trim();
            if line.is_empty() { continue; }
            let parts: Vec<&str> = line.split("->").collect();
            if parts.len() != 2 { continue; }
            let head = parts[0].trim().to_string();
            let bodies = parts[1].split('|');
            for body in bodies {
                let symbols: Vec<String> = body.split_whitespace()
                    .map(|s| {
                        if s == "lambda" || s == "epsilon" || s == "λ" || s == "ε" {
                            String::new()
                        } else {
                            s.to_string()
                        }
                    })
                    .filter(|s| !s.is_empty())
                    .collect();
                productions.entry(head.clone()).or_insert_with(Vec::new).push(symbols);
            }
        }

        Ok(Cfg {
            non_terminals,
            terminals,
            start_symbol,
            productions,
        })
    }

    /// Exercit'iul 1: Generarea cuvintelor de lungime k
    pub fn generate_words(&self, k: usize) -> HashSet<String> {
        let mut results = HashSet::new();
        let mut queue = VecDeque::new();
        queue.push_back(vec![self.start_symbol.clone()]);

        // Use a limit to prevent infinite loops in case of cycle + lambda
        // although the problem says we can assume no lambda productions that shorten the string
        // but it's safer to have some bound or use BFS properly.
        
        while let Some(current) = queue.pop_front() {
            let terminal_count = current.iter().filter(|s| self.terminals.contains(*s)).count();
            let non_terminal_count = current.len() - terminal_count;

            if non_terminal_count == 0 {
                if terminal_count == k {
                    results.insert(current.concat());
                }
                continue;
            }

            // If terminal_count > k, and assuming no lambda, we can't reach k.
            if terminal_count > k {
                continue;
            }
            
            // To avoid infinite derivation if there are loops, we can limit the length
            // If we assume no lambda, then length never decreases.
            // If length > k and no lambda, we can cut.
            if current.len() > k && self.productions.values().flatten().all(|p| !p.is_empty()) {
                // This is a simplified check.
                continue;
            }
            
            // Another safety break for very long derivations
            if current.len() > k + 20 {
                continue;
            }

            if let Some(pos) = current.iter().position(|s| self.non_terminals.contains(s)) {
                let nt = &current[pos];
                if let Some(prods) = self.productions.get(nt) {
                    for prod in prods {
                        let mut next = current.clone();
                        next.splice(pos..pos+1, prod.iter().cloned());
                        queue.push_back(next);
                    }
                }
            }
        }
        results
    }

    /// Exercit'iul 2: Transformarea î'n Forma Normală' Chomsky
    pub fn to_cnf(&self) -> Cfg {
        let mut cnf = self.clone();

        // Pasul 1: START
        let mut s_in_rhs = false;
        for prods in cnf.productions.values() {
            for prod in prods {
                if prod.contains(&cnf.start_symbol) {
                    s_in_rhs = true;
                    break;
                }
            }
            if s_in_rhs { break; }
        }
        if s_in_rhs {
            let old_start = cnf.start_symbol.clone();
            cnf.start_symbol = "S0".to_string();
            cnf.non_terminals.insert(cnf.start_symbol.clone());
            cnf.productions.insert(cnf.start_symbol.clone(), vec![vec![old_start]]);
        }

        // Pasul 2: TERM (Replace terminals in mixed rules)
        let mut terminal_to_nt = HashMap::new();
        let mut new_productions = HashMap::new();
        
        for (head, prods) in &cnf.productions {
            let mut updated_prods = Vec::new();
            for prod in prods {
                if prod.len() > 1 || (prod.len() == 1 && cnf.non_terminals.contains(&prod[0])) {
                    let mut new_prod = Vec::new();
                    for sym in prod {
                        if cnf.terminals.contains(sym) {
                            let nt = terminal_to_nt.entry(sym.clone()).or_insert_with(|| {
                                format!("X_{}", sym)
                            }).clone();
                            new_prod.push(nt);
                        } else {
                            new_prod.push(sym.clone());
                        }
                    }
                    updated_prods.push(new_prod);
                } else {
                    updated_prods.push(prod.clone());
                }
            }
            new_productions.insert(head.clone(), updated_prods);
        }
        
        for (term, nt) in terminal_to_nt {
            cnf.non_terminals.insert(nt.clone());
            new_productions.insert(nt, vec![vec![term]]);
        }
        cnf.productions = new_productions;

        // Pasul 3: BIN (Binarizare)
        let mut binarized_prods = HashMap::new();
        let mut bin_counter = 0;
        for (head, prods) in &cnf.productions {
            let mut updated_prods = Vec::new();
            for prod in prods {
                if prod.len() > 2 {
                    let mut current_head = head.clone();
                    for i in 0..prod.len() - 2 {
                        let new_nt = format!("C_{}", bin_counter);
                        bin_counter += 1;
                        cnf.non_terminals.insert(new_nt.clone());
                        
                        let entry = binarized_prods.entry(current_head.clone()).or_insert_with(Vec::new);
                        entry.push(vec![prod[i].clone(), new_nt.clone()]);
                        
                        current_head = new_nt;
                    }
                    let entry = binarized_prods.entry(current_head).or_insert_with(Vec::new);
                    entry.push(vec![prod[prod.len()-2].clone(), prod[prod.len()-1].clone()]);
                } else {
                    updated_prods.push(prod.clone());
                }
            }
            if !updated_prods.is_empty() {
                binarized_prods.entry(head.clone()).or_insert_with(Vec::new).extend(updated_prods);
            }
        }
        cnf.productions = binarized_prods;

        // Pasul 4: DEL (λ-eliminare)
        let mut nullable = HashSet::new();
        let mut changed = true;
        while changed {
            changed = false;
            for (head, prods) in &cnf.productions {
                if nullable.contains(head) { continue; }
                for prod in prods {
                    if prod.is_empty() || prod.iter().all(|s| nullable.contains(s)) {
                        nullable.insert(head.clone());
                        changed = true;
                        break;
                    }
                }
            }
        }
        
        let mut no_lambda_prods = HashMap::new();
        for (head, prods) in &cnf.productions {
            let mut updated_prods = HashSet::new();
            for prod in prods {
                if prod.is_empty() { continue; }
                
                let mut current_gen = vec![Vec::new()];
                for sym in prod {
                    let mut next_gen = Vec::new();
                    for p in current_gen {
                        // Option 1: Keep sym
                        let mut p1 = p.clone();
                        p1.push(sym.clone());
                        next_gen.push(p1);
                        
                        // Option 2: Remove sym (if nullable)
                        if nullable.contains(sym) {
                            next_gen.push(p);
                        }
                    }
                    current_gen = next_gen;
                }
                for p in current_gen {
                    if !p.is_empty() {
                        updated_prods.insert(p);
                    }
                }
            }
            no_lambda_prods.insert(head.clone(), updated_prods.into_iter().collect());
        }
        // If start symbol is nullable, we might need to add S -> λ back if we want to preserve λ in language
        // but CNF usually doesn't include λ except possibly at start.
        // The PDF doesn't specify this, so I'll leave it as is.
        cnf.productions = no_lambda_prods;

        // Pasul 5: UNIT (Eliminarea unitarelor)
        let mut unit_closures = HashMap::new();
        for nt in &cnf.non_terminals {
            let mut closure = HashSet::new();
            let mut stack = vec![nt.clone()];
            while let Some(current) = stack.pop() {
                if let Some(prods) = cnf.productions.get(&current) {
                    for prod in prods {
                        if prod.len() == 1 && cnf.non_terminals.contains(&prod[0]) {
                            if closure.insert(prod[0].clone()) {
                                stack.push(prod[0].clone());
                            }
                        }
                    }
                }
            }
            unit_closures.insert(nt.clone(), closure);
        }
        
        let mut final_prods = HashMap::new();
        for (head, prods) in &cnf.productions {
            let mut updated_prods = HashSet::new();
            // Non-unit productions
            for prod in prods {
                if prod.len() != 1 || !cnf.non_terminals.contains(&prod[0]) {
                    updated_prods.insert(prod.clone());
                }
            }
            // Productions from unit closure
            if let Some(closure) = unit_closures.get(head) {
                for unit_nt in closure {
                    if let Some(unit_prods) = cnf.productions.get(unit_nt) {
                        for prod in unit_prods {
                            if prod.len() != 1 || !cnf.non_terminals.contains(&prod[0]) {
                                updated_prods.insert(prod.clone());
                            }
                        }
                    }
                }
            }
            final_prods.insert(head.clone(), updated_prods.into_iter().collect());
        }
        cnf.productions = final_prods;

        cnf
    }

    /// Exercit'iul 3: Algoritmul CYK
    pub fn cyk(&self, word: &str) -> bool {
        let n = word.len();
        if n == 0 {
            // Check if S can produce λ. This is not directly handled by basic CYK.
            // But we can check if S was nullable before λ-elimination.
            // Or just check if there's an empty production for start symbol in original.
            // For now, let's assume word is non-empty or handle it specially.
            return false; 
        }

        // T[i][j] is a set of non-terminals that can generate word[i..i+j]
        // Using 1-based indexing for length j as in PDF
        let mut table = vec![vec![HashSet::new(); n + 1]; n];

        // Step 1: Base case (j = 1)
        for i in 0..n {
            let sym = word.chars().nth(i).unwrap().to_string();
            for (head, prods) in &self.productions {
                for prod in prods {
                    if prod.len() == 1 && prod[0] == sym {
                        table[i][1].insert(head.clone());
                    }
                }
            }
        }

        // Step 2: Recursive step
        for j in 2..=n { // length
            for i in 0..=n - j { // start position
                let mut added_heads = HashSet::new();
                for k in 1..j { // split point
                    // T[i, j] = {A | A -> BC, B in T[i, k], C in T[i+k, j-k]}
                    let set_b = &table[i][k];
                    let set_c = &table[i + k][j - k];
                    
                    if set_b.is_empty() || set_c.is_empty() { continue; }
                    
                    for (head, prods) in &self.productions {
                        for prod in prods {
                            if prod.len() == 2 {
                                if set_b.contains(&prod[0]) && set_c.contains(&prod[1]) {
                                    added_heads.insert(head.clone());
                                }
                            }
                        }
                    }
                }
                table[i][j].extend(added_heads);
            }
        }

        table[0][n].contains(&self.start_symbol)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_words() {
        let mut non_terminals = HashSet::new();
        non_terminals.insert("S".to_string());
        let mut terminals = HashSet::new();
        terminals.insert("a".to_string());
        terminals.insert("b".to_string());
        let mut productions = HashMap::new();
        productions.insert("S".to_string(), vec![
            vec!["a".to_string(), "S".to_string(), "b".to_string()],
            vec![],
        ]);
        let cfg = Cfg {
            non_terminals,
            terminals,
            start_symbol: "S".to_string(),
            productions,
        };

        let words = cfg.generate_words(2);
        assert_eq!(words, ["ab".to_string()].into_iter().collect::<HashSet<_>>());
        
        let words4 = cfg.generate_words(4);
        assert_eq!(words4, ["aabb".to_string()].into_iter().collect::<HashSet<_>>());
    }

    #[test]
    fn test_cnf_cyk() {
        let mut non_terminals = HashSet::new();
        non_terminals.insert("S".to_string());
        let mut terminals = HashSet::new();
        terminals.insert("a".to_string());
        terminals.insert("b".to_string());
        let mut productions = HashMap::new();
        productions.insert("S".to_string(), vec![
            vec!["a".to_string(), "S".to_string(), "b".to_string()],
            vec!["a".to_string(), "b".to_string()],
        ]);
        let cfg = Cfg {
            non_terminals,
            terminals,
            start_symbol: "S".to_string(),
            productions,
        };

        let cnf = cfg.to_cnf();
        assert!(cnf.cyk("ab"));
        assert!(cnf.cyk("aabb"));
        assert!(cnf.cyk("aaabbb"));
        assert!(!cnf.cyk("a"));
        assert!(!cnf.cyk("b"));
        assert!(!cnf.cyk("aba"));
    }
}
