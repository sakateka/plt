use cfg;
use std::collections::HashSet;

pub fn check(text: &str, grammar: &cfg::CFG) -> bool {
    let cnf = grammar.chomsky();
    let text_len = text.chars().count();
    if text_len == 0 {
        // special case for empty string
        for rule in &cnf.productions {
            if rule.left == cnf.start {
                if rule.right.len() == 0 {
                    return true
                }
            }
        }
        return false;
    }
    let mut table = vec![vec![HashSet::new(); text_len]; text_len];

    for (idx, ch) in text.chars().enumerate() {
        for rule in &cnf.productions {
            if rule.right.len() == 1 && rule.right[0].is_eq_term(ch) {
                table[idx][idx].insert(&rule.left);
            }
        }
    }
    for l in 1..text_len {
        // every substring length
        for r in 0..(text_len - l) {
            // every starting location for a substring of length l
            for t in 0..l {
                // every split of the substring at text[r: r+l]
                for rule in &cnf.productions {
                    if rule.right.len() != 2 {
                        continue;
                    }
                    // assert!(rule.right.iter().all(|x| x.is_nonterminal()));
                    let n1_set: Vec<_> = table[r][r + t].iter().cloned().collect();
                    let n2_set: Vec<_> = table[r + t + 1][r + l].iter().cloned().collect();

                    for n1 in &n1_set {
                        for n2 in &n2_set {
                            if rule.right[0].is_eq_nonterm(n1) && rule.right[1].is_eq_nonterm(n2) {
                                table[r][r + l].insert(&rule.left);
                            }
                        }
                    }
                }
            }
        }
    }
    table[0][text_len - 1].contains(&cnf.start)
}
