use std::fmt;
use std::io::{self, BufRead, BufReader};
use std::fs::File;
use std::collections::{HashSet, HashMap};
use itertools::join;

const ALPHA: &'static str = "ABCDEFGHIJKLMNOPQRSTUVWXYZΓΔΘΛΞΣΦΨΩ";

#[derive(Debug, Hash, PartialEq, Clone)]
pub struct Nonterminal {
    pub symbol: char,
}

impl Eq for Nonterminal {}

impl Nonterminal {
    pub fn new(from: char) -> Nonterminal {
        Nonterminal {
            symbol: from,
        }
    }
}
impl fmt::Display for Nonterminal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.symbol)
    }
}


#[derive(Debug, Hash, PartialEq, Clone)]
pub struct Terminal {
    pub symbol: char,
}

impl Terminal {
    pub fn new(from: char) -> Terminal {
        Terminal {
            symbol: from,
        }
    }
}


#[derive(Debug, Hash, PartialEq, Clone)]
pub enum Symbol {
    N(Nonterminal),
    T(Terminal),
}
impl Eq for Symbol {}

impl Symbol {
    pub fn new(c: char) -> Symbol {
        if c.is_lowercase() {
            Symbol::T(Terminal::new(c))
        } else {
            Symbol::N(Nonterminal::new(c))
        }
    }
    pub fn get_symbol(&self) -> char {
        match self {
            &Symbol::T(ref x) => x.symbol,
            &Symbol::N(ref x) => x.symbol,
        }
    }
    pub fn is_nonterminal(&self) -> bool {
        match self {
            &Symbol::T(_) => false,
            &Symbol::N(_) => true,
        }
    }
    pub fn as_nonterminal(&self) -> Option<&Nonterminal> {
        match self {
            &Symbol::T(_) => None,
            &Symbol::N(ref x) => Some(&x),
        }
    }
}

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.get_symbol())
    }
}

#[derive(Debug, Hash, PartialEq, Clone)]
pub struct Production {
    pub left: Nonterminal,
    pub right: Vec<Symbol>,
}

impl Eq for Production {}

impl Production {
    pub fn new(l: Nonterminal, r: Vec<Symbol>) -> Production {
        Production {
            left: l,
            right: r,
        }
    }
}

#[derive(Debug)]
pub struct CFG {
    pub start: Nonterminal,
    pub productions: HashSet<Production>,
    pub variables: HashSet<Nonterminal>,
}
impl fmt::Display for CFG {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut rules: HashMap<Nonterminal, Vec<String>> = HashMap::new();
        let mut prods: Vec<&Production> = self.productions.iter().collect();
        prods.sort_by(|a, b| a.left.symbol.cmp(&b.left.symbol));
        for rule in &prods {
            let mut chars = match rules.get(&rule.left) {
                Some(s) => s.clone(),
                None => Vec::new(),
            };
            chars.push(join(&rule.right, ""));
            rules.insert(rule.left.clone(), chars);
        }
        if let Some(mut start) = rules.remove(&self.start) {
            start.sort();
            if let Err(e) = write!(f, "{} -> {}\n", self.start.symbol, join(start, " | ")) {
                return Err(e)
            }
        } else {
            if rules.len() > 0 {
                eprintln!("Don't see Start Symbol");
                return Err(fmt::Error{})
            }
        }
        for rule in &prods {
            if let Some(mut val) = rules.remove(&rule.left) {
                val.sort();
                if let Err(e) = write!(f, "{} -> {}\n", rule.left.symbol, join(val, " | ")) {
                    return Err(e)
                }
            }
        }
        Ok(())
    }
}

impl CFG {
    pub fn new(start: Nonterminal, prods: HashSet<Production>) -> CFG {
        let mut cfg = CFG{
            start: start,
            productions: prods,
            variables: HashSet::new(),
        };
        cfg.update_variables();
        cfg
    }

    pub fn parse(input_path: &str) -> io::Result<CFG> {
        let file = BufReader::new(File::open(input_path)?);
        CFG::parse_from_reader(file)
    }
    pub fn parse_from_reader<R: ?Sized + BufRead>(r: R) -> io::Result<CFG>
        where R: ::std::marker::Sized {

        let mut start: Option<Nonterminal> = None;
        let mut productions = HashSet::new();
        for line in r.lines() {
            let mut text = line?;
            let rule = text.trim();
            if rule.is_empty() || rule.starts_with('#') {
                continue
            }
            let add_productions = CFG::parse_production(&rule)?;
            if productions.is_empty() {
                // The first valid rule is the start character here
                start = Some(add_productions[0].left.clone());
            }
            productions.extend(add_productions.iter().cloned());
        }
        if let Some(s) = start {
            Ok(CFG::new(s, productions))
        } else {
            Err(io::Error::new(io::ErrorKind::Other, "Don't see any rule"))
        }
    }

    pub fn parse_production(line: &str) -> io::Result<Vec<Production>> {
        let mut productions = Vec::new();
        let rule: Vec<&str> = line.split("->").map(|x| x.trim()).collect();
        if rule.len() != 2 || rule[0].len() > 1 {
            return Err(io::Error::new(io::ErrorKind::Other, format!("Bad rule: {}", line)));
        }

        let left_letter =  rule[0].chars().next().unwrap();
        if left_letter.is_lowercase() {
            return Err(io::Error::new(io::ErrorKind::Other, format!("Terminal symbol at RHS: {}", line)));
        }
        for rhs in rule[1].split('|').map(|x| x.trim()) {
            productions.push(
                Production::new(
                    Nonterminal::new(left_letter),
                    rhs.chars().map(|x| Symbol::new(x)).collect(),
                )
            )
        }
        Ok(productions)
    }

    pub fn update_variables(&mut self) {
        self.variables.clear();
        for rule in &self.productions {
            self.variables.extend(rule.right.iter().cloned()
                .filter(|x| x.is_nonterminal())
                .map(|x| match x { Symbol::N(n) => n, _ => unreachable!() })
                .collect::<HashSet<Nonterminal>>()
            );
        }
    }

    pub fn get_new_start(&self) -> Nonterminal {
        let mut free_variables = ALPHA.chars()
            .map(|x| Nonterminal::new(x))
            .filter(|x| !self.variables.contains(x))
            .collect::<Vec<Nonterminal>>();
        free_variables.pop()
            .expect("Exceeded the maximum number of non-terminal characters")
    }

    pub fn simplify(&self) -> CFG {
        self.remove_epsilon_rules()
            .remove_unit_rules()
            .remove_useless_rules()
            .remove_unreachable_rules()
    }

    pub fn remove_epsilon_rules(&self) -> CFG {
        let mut new_rules: HashSet<Production> = HashSet::new();
        let mut nullable: HashSet<Nonterminal> = HashSet::new();
        let mut changed = true;
        while changed {
            changed = false;
            for rule in &self.productions {
                if rule.right.is_empty() {
                    if nullable.insert(rule.left.clone()) {
                        changed = true;
                    }
                }
                // if the rule contains only Nonterminal-s and they all lead to epsilon
                if rule.right.iter()
                    .fold(true, |acc, x| if !acc { acc } else {
                        x.is_nonterminal() 
                        && nullable.contains(x.as_nonterminal().unwrap())
                    }) {
                    if nullable.insert(rule.left.clone()) {
                        changed = true;
                    }
                }
            }
        }
        let mut start_hash_epsilon = false;
        for rule in &self.productions {
            if !rule.right.is_empty() {
                new_rules.insert(rule.clone());
            } else {
                start_hash_epsilon = rule.left == self.start;
            }
        }
        for null in &nullable {
            for rule in &self.productions {
                for (idx, sym) in rule.right.iter().enumerate() {
                    if let &Symbol::N(ref n) = sym {
                        if n == null {
                            let mut new = Production::new(
                                rule.left.clone(),
                                rule.right.clone()
                            );
                            new.right.remove(idx);
                            if !new.right.is_empty() // skip new epsilon rule
                                // skip new unit rule
                                && !(new.right.len() == 1 && new.right[0].is_nonterminal()
                                     && new.right[0].as_nonterminal().unwrap() == &new.left){
                                new_rules.insert(new);
                            }
                        }
                    }
                }
            }
        }
        let mut start = self.start.clone();
        if nullable.contains(&self.start) && start_hash_epsilon {
            // add 'S1 -> λ | S'
            let new_start = self.get_new_start();
            new_rules.insert(Production::new(new_start.clone(), Vec::new()));
            new_rules.insert(Production::new(new_start.clone(), vec![Symbol::N(start)]));
            start = new_start;
        }
        CFG::new(start, new_rules)
    }

    pub fn remove_unit_rules(&self) -> CFG {
        CFG::new(self.start.clone(), self.productions.clone())
    }

    pub fn remove_useless_rules(&self) -> CFG {
        let mut usefull_nonterminals: HashSet<Nonterminal> = HashSet::new();
        let mut changed = true;
        while changed {
            changed = false;
            for rule in &self.productions {
                if rule.right.is_empty() {
                    // epsilon rule
                    continue;
                } else {
                    let right_nonterm_set: HashSet<Nonterminal> = rule.right.iter().cloned()
                        .filter(|x| x.is_nonterminal())
                        .map(|x| match x { Symbol::N(n) => n, _ => unreachable!() })
                        .collect();
                    if right_nonterm_set.is_empty() ||
                        right_nonterm_set.is_subset(&usefull_nonterminals) {
                        // if rule contains only terminals or all Nonterminals can be generated
                        if usefull_nonterminals.insert(rule.left.clone()) {
                            changed = true;
                        }
                    }
                }
            }
        }
        let mut productions = HashSet::new();
        for rule in &self.productions {
            let right_nonterm_set: HashSet<Nonterminal> = rule.right.iter().cloned()
                .filter(|x| x.is_nonterminal())
                .map(|x| match x { Symbol::N(n) => n, _ => unreachable!() })
                .collect();
            let here = usefull_nonterminals.contains(&rule.left);
            if here && right_nonterm_set.is_subset(&usefull_nonterminals) {
                productions.insert(rule.clone());
            }
        }
        CFG::new(self.start.clone(), productions)
    }

    pub fn remove_unreachable_rules(&self) -> CFG {
        let mut reachable_symbols: HashSet<Symbol> = HashSet::new();
        reachable_symbols.insert(Symbol::N(self.start.clone()));
        let mut changed = true;
        while changed {
            changed = false;
            for rule in &self.productions {
                if reachable_symbols.contains(&Symbol::N(rule.left.clone())) {
                    for s in &rule.right {
                        if reachable_symbols.insert(s.clone()) {
                            changed = true;
                        }
                    }
                }
            }
        }
        let mut productions = HashSet::new();
        for rule in &self.productions {
            let mut right_set: HashSet<Symbol> = rule.right.iter().cloned().collect();
            right_set.insert(Symbol::N(rule.left.clone()));
            if right_set.is_subset(&reachable_symbols) {
                productions.insert(rule.clone());
            }
        }
        CFG::new(self.start.clone(), productions)
    }
}

#[cfg(test)]
mod tests {
    use self::super::*;
    use std::io::Cursor;

    #[test]
    fn remove_epsilon() {
        let test_rules = r#"
            S -> AaB | aB | cC
            A -> AB | a | b | B
            B -> Ba |
            C -> AB | c
        "#;
        let expected = format!("{}\n", join(vec![
            "S -> Aa | AaB | a | aB | c | cC",
            "A -> AB | B | a | b",
            "B -> Ba | a",
            "C -> A | AB | B | c",
        ], "\n"));
        let cfg = CFG::parse_from_reader(Cursor::new(test_rules)).unwrap();
        assert_eq!(format!("{}", cfg.remove_epsilon_rules()), expected);
    }

    #[test]
    fn remove_useless() {
        let test_rules = r#"
            S -> aAB | E
            A -> aA | bB
            B -> ACb| b
            C -> A | bA | cC | aE
            D -> a | c | Fb
            E -> cE | aE | Eb | ED | FG
            F -> BC | EC | AC
            G -> Ga | Gb
        "#;
        let expected = format!("{}\n", join(vec![
            "S -> aAB",
            "A -> aA | bB",
            "B -> ACb | b",
            "C -> A | bA | cC",
            "D -> Fb | a | c",
            "F -> AC | BC"
        ], "\n"));
        let cfg = CFG::parse_from_reader(Cursor::new(test_rules)).unwrap()
            .remove_epsilon_rules();
        assert_eq!(format!("{}", cfg.remove_useless_rules()), expected);
    }

    #[test]
    fn remove_unreachable() {
        let test_rules = r#"
            S -> aAB | E
            A -> aA | bB
            B -> ACb| b
            C -> A | bA | cC | aE
            D -> a | c | Fb
            E -> cE | aE | Eb | ED | FG
            F -> BC | EC | AC
            G -> Ga | Gb
        "#;
        let expected = format!("{}\n", join(vec![
            "S -> aAB",
            "A -> aA | bB",
            "B -> ACb | b",
            "C -> A | bA | cC",
        ], "\n"));
        let cfg = CFG::parse_from_reader(Cursor::new(test_rules)).unwrap()
            .remove_epsilon_rules()
            .remove_useless_rules();
        assert_eq!(format!("{}", cfg.remove_unreachable_rules()), expected);
    }

}
