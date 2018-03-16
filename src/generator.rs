use std::collections::{HashSet, HashMap};
use cfg;

pub struct Generator {
    rules: HashMap<cfg::Nonterminal, Vec<Vec<cfg::Symbol>>>,
    queue: HashSet<Vec<cfg::Symbol>>,
    from: u32,
    upto: u32,
}


impl Generator {
    pub fn new(grammar: cfg::CFG, lmin: u32, lmax: u32) -> Generator {
        let mut rules: HashMap<cfg::Nonterminal, Vec<Vec<cfg::Symbol>>> = HashMap::new();
        for rule in grammar.simplify().productions {
            let mut symbols = match rules.get(&rule.left) {
                Some(s) => s.clone(),
                None => Vec::new(),
            };
            symbols.push(rule.right.clone());
            rules.insert(rule.left.clone(), symbols);
        }
        Generator {
            rules: rules,
            queue: HashSet::new(),
            from: lmin,
            upto: lmax,
        }
    }
}

impl Iterator for Generator {
    type Item = Vec<cfg::Symbol>;
    fn next(&mut self) -> Option<Vec<cfg::Symbol>> {

        None
    }
}
