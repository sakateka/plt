use std::io::{self, BufRead, BufReader};
use std::fs::File;
use std::collections::HashSet;

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

impl Symbol {
    pub fn new(c: char) -> Symbol {
        if c.is_lowercase() {
            Symbol::T(Terminal::new(c))
        } else {
            Symbol::N(Nonterminal::new(c))
        }
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

impl CFG {
    pub fn parse(input_path: &str) -> io::Result<CFG> {
        let file = BufReader::new(
            File::open(input_path).expect(&format!("opening file {}", input_path))
        );
        let mut cfg: CFG;
        let mut it = file.lines();
        if let Some(first_line) = it.next() {
            let text = first_line.unwrap();
            let first_productions = CFG::parse_production(text.as_str())?;
            cfg = CFG{
                start: first_productions[0].left.clone(),
                productions: first_productions.iter().cloned().collect(),
            };
            
        } else {
            return Err(io::Error::new(io::ErrorKind::Other, "Don't see any rule",));
        }
        for line in it {
            let add_productions = CFG::parse_production(&line.unwrap()).unwrap();
            cfg.productions.extend(add_productions.iter().cloned());
        }
        Ok(cfg)
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
        self.remove_useless_rules()
            .remove_epsilon_rules()
            .remove_unit_rules()
    }

    pub fn remove_useless_rules(&self) -> CFG {
        let mut usefull_nonterminals: HashSet<Nonterminal> = HashSet::new();
        let mut changed = true;
        while changed {
            changed = false;
            for rule in &self.productions {
                if rule.right.len() == 0 {
                    // epsilon rule
                    if usefull_nonterminals.insert(rule.left.clone()) {
                        changed = true;
                    }
                    continue;
                }
                for sym in &rule.right {
                    match sym {
                        &Symbol::T(_) => {
                            if usefull_nonterminals.insert(rule.left.clone()) {
                                changed = true;
                            }
                        },
                        &Symbol::N(ref n) => {
                            if let Some(_) = usefull_nonterminals.get(n) {
                                if usefull_nonterminals.insert(rule.left.clone()) {
                                    changed = true;
                                }
                            }
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
            if let Some(_) = usefull_nonterminals.get(&rule.left) {
                cfg.productions.insert(rule.clone());
            }
        }
        cfg
    }

    pub fn remove_epsilon_rules(&self) -> CFG {
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
}
