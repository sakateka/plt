use cfg;
use std::collections::{HashMap, HashSet};
use std::fmt;

pub struct Generator {
    left: bool,
    rules: HashMap<cfg::Symbol, Vec<Vec<cfg::Symbol>>>,
    queue: HashSet<Vec<cfg::Symbol>>,
    visited: HashSet<Vec<cfg::Symbol>>,
    min_len: usize,
    max_len: usize,
}

#[derive(Debug)]
pub struct GeneratedItem<'a>(pub &'a Vec<cfg::Symbol>);

impl<'a> fmt::Display for GeneratedItem<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            self.0
                .iter()
                .fold(String::new(), |acc, ref arg| acc + arg.to_string().as_ref())
        )
    }
}

#[derive(Debug)]
pub struct GeneratedSet(pub HashSet<Vec<cfg::Symbol>>);

impl fmt::Display for GeneratedSet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for item in &self.0 {
            writeln!(f, "{}", GeneratedItem(item)).unwrap();
        }
        Ok(())
    }
}

impl Generator {
    pub fn new(grammar: cfg::CFG, lmin: u32, lmax: u32, left: bool) -> Generator {
        let mut rules: HashMap<cfg::Symbol, Vec<Vec<cfg::Symbol>>> = HashMap::new();
        for rule in grammar.productions {
            let mut symbols = match rules.get(&cfg::Symbol::N(rule.left.clone())) {
                Some(s) => s.clone(),
                None => Vec::new(),
            };
            symbols.push(rule.right.clone());
            rules.insert(cfg::Symbol::N(rule.left.clone()), symbols);
        }
        let mut queue = HashSet::new();
        if let Some(cases) = rules.get(&cfg::Symbol::N(grammar.start)) {
            for case in cases {
                queue.insert(case.clone());
            }
        }
        Generator {
            left,
            rules,
            queue,
            visited: HashSet::new(),
            min_len: lmin as usize,
            max_len: lmax as usize,
        }
    }
}

impl Iterator for Generator {
    type Item = Vec<cfg::Symbol>;

    fn next(&mut self) -> Option<Vec<cfg::Symbol>> {
        loop {
            let next_item = match self.queue.iter().next() {
                Some(item) => item.to_vec(),
                None => return None,
            };
            self.queue.remove(&next_item);
            if next_item.is_empty() {
                return Some(next_item);
            }
            if next_item.len() > self.max_len {
                // too long a sequence, drop it
                continue;
            }
            if next_item.iter().all(|x| x.is_terminal()) {
                // only terminals
                if next_item.len() >= self.min_len {
                    return Some(next_item);
                } else {
                    // too short a sequence, drop
                    continue;
                }
            }
            let idx = if self.left {
                next_item.iter().position(|x| x.is_nonterminal()).unwrap()
            } else {
                next_item.iter().rposition(|x| x.is_nonterminal()).unwrap()
            };
            if let Some(rules) = self.rules.get(&next_item[idx]) {
                for seq in rules {
                    let mut new_seq = next_item[..idx].to_vec();
                    new_seq.extend(seq.clone());
                    if next_item.len() > idx + 1 {
                        new_seq.extend(next_item[idx + 1..].iter().cloned());
                    }
                    if new_seq.len() <= self.max_len && !self.visited.contains(&new_seq) {
                        self.visited.insert(new_seq.clone());
                        self.queue.insert(new_seq);
                    }
                }
            } else {
                unreachable!() // unreachable Nonterminal symbol ???
            }
        }
    }
}
