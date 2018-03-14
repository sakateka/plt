use std::fmt;
use std::io::{self, BufRead, BufReader};
use std::fs::File;
use std::collections::{HashSet, HashMap};
use itertools::join;

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
            return Err(fmt::Error{})
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
    pub fn parse(input_path: &str) -> io::Result<CFG> {
        let file = BufReader::new(File::open(input_path)?);
        CFG::parse_from_reader(file)
    }
    pub fn parse_from_reader<R: ?Sized + BufRead>(r: R) -> io::Result<CFG>
        where R: ::std::marker::Sized {

        let mut cfg = CFG{
            start: Nonterminal::new('?'),
            productions: HashSet::new(),
        };
        for line in r.lines() {
            let mut text = line.unwrap();
            let rule = text.trim();
            if rule.len() == 0 || rule.starts_with('#') {
                continue
            }
            let add_productions = CFG::parse_production(&rule).unwrap();
            if cfg.productions.len() == 0 {
                // first valid rule
                cfg.start = add_productions[0].left.clone();
            }
            cfg.productions.extend(add_productions.iter().cloned());
        }
        if cfg.productions.len() == 0 {
            Err(io::Error::new(io::ErrorKind::Other, "Don't see any rule"))
        } else {
            Ok(cfg)
        }
    }
    pub fn parse_production(line: &str) -> io::Result<Vec<Production>> {
        let mut productions = Vec::new();
        let rule: Vec<&str> = line.split("->").map(|x| x.trim()).collect();
        if rule.len() != 2 || rule[0].len() > 1 {
            return Err(io::Error::new(io::ErrorKind::Other, format!("Bad rule: {}", line)));
        }
        let start = rule[0].chars().next().unwrap();
        if start.is_lowercase() {
            return Err(io::Error::new(io::ErrorKind::Other, format!("Terminal symbol at RHS: {}", line)));
        }
        for lhs in rule[1].split('|').map(|x| x.trim()) {
            productions.push(
                Production::new(
                    Nonterminal::new(start),
                    lhs.chars().map(|x| Symbol::new(x)).collect(),
                )
            );
        }
        Ok(productions)
    }

    pub fn simplify(&self) -> CFG {
        self.remove_epsilon_rules()
            .remove_useless_rules()
            .remove_unreachable_rules()
            .remove_unit_rules()
    }

    pub fn remove_epsilon_rules(&self) -> CFG {
        let mut new_rules: HashSet<Production> = HashSet::new();
        let mut nullable: HashSet<Nonterminal> = HashSet::new();
        let mut changed = true;
        while changed {
            changed = false;
            for rule in &self.productions {
                if rule.right.len() == 0 {
                    nullable.insert(rule.left.clone());
                }
                for s in &rule.right {
                    if let &Symbol::N(ref n) = s {
                        if nullable.get(n).is_some() {
                            if nullable.insert(rule.left.clone()) {
                                changed = true;
                            }
                        }
                    }
                }
            }
        }
        println!("Nullable: -> {}", join(nullable, " "));
        CFG {
            start: self.start.clone(),
            productions: self.productions.clone(),
        }
    }

    pub fn remove_unit_rules(&self) -> CFG {
        CFG {
            start: self.start.clone(),
            productions: self.productions.clone(),
        }
    }

    pub fn remove_useless_rules(&self) -> CFG {
        let mut usefull_nonterminals: HashSet<Nonterminal> = HashSet::new();
        let mut changed = true;
        while changed {
            changed = false;
            for rule in &self.productions {
                if rule.right.len() == 0 {
                    // epsilon rule
                    continue;
                } else {
                    let right_nonterm_set: HashSet<Nonterminal> = rule.right.iter().cloned()
                        .filter(|x| x.is_nonterminal())
                        .map(|x| match x { Symbol::N(n) => n, _ => unreachable!() })
                        .collect();
                    if right_nonterm_set.len() == 0 ||
                        right_nonterm_set.is_subset(&usefull_nonterminals) {
                        // if rule contains only terminals or all Nonterminals can be generated
                        if usefull_nonterminals.insert(rule.left.clone()) {
                            changed = true;
                        }
                    }
                }
            }
        }
        let mut cfg = CFG {
            start: self.start.clone(),
            productions: HashSet::new(),
        };
        for rule in &self.productions {
            let right_nonterm_set: HashSet<Nonterminal> = rule.right.iter().cloned()
                .filter(|x| x.is_nonterminal())
                .map(|x| match x { Symbol::N(n) => n, _ => unreachable!() })
                .collect();
            let some = usefull_nonterminals.get(&rule.left);
            if some.is_some() && right_nonterm_set.is_subset(&usefull_nonterminals) {
                cfg.productions.insert(rule.clone());
            }
        }
        cfg
    }

    pub fn remove_unreachable_rules(&self) -> CFG {
        let mut reachable_symbols: HashSet<Symbol> = HashSet::new();
        reachable_symbols.insert(Symbol::N(self.start.clone()));
        let mut changed = true;
        while changed {
            changed = false;
            for rule in &self.productions {
                if reachable_symbols.get(&Symbol::N(rule.left.clone())).is_some() {
                    for s in &rule.right {
                        if reachable_symbols.insert(s.clone()) {
                            changed = true;
                        }
                    }
                }
            }
        }
        let mut cfg = CFG {
            start: self.start.clone(),
            productions: HashSet::new(),
        };
        for rule in &self.productions {
            let mut right_set: HashSet<Symbol> = rule.right.iter().cloned().collect();
            right_set.insert(Symbol::N(rule.left.clone()));
            if right_set.is_subset(&reachable_symbols) {
                cfg.productions.insert(rule.clone());
            }
        }
        cfg
    }
}

#[cfg(test)]
mod tests {
    use self::super::*;
    use std::io::Cursor;

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
        let cfg = CFG::parse_from_reader(Cursor::new(test_rules)).unwrap();
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
            .remove_useless_rules();
        assert_eq!(format!("{}", cfg.remove_unreachable_rules()), expected);
    }

    #[test]
    fn remove_epsilon() {
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
            .remove_useless_rules()
            .remove_unreachable_rules();
        assert_eq!(format!("{}", cfg.remove_epsilon_rules()), expected);
    }
}
