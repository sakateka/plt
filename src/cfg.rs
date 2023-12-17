use itertools::join;
use std::collections::{BTreeSet, HashMap, HashSet};
use std::fmt;
use std::fs::File;
use std::io::{self, BufRead, BufReader};

#[derive(Debug, Hash, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Nonterminal {
    pub name: String,
    pub sub_index: u32,
}

impl Nonterminal {
    pub fn new(name: String, sub_index: u32) -> Nonterminal {
        Nonterminal { name, sub_index }
    }
    pub fn parse(from: String) -> Nonterminal {
        let mut name = from.trim_matches(|x| x == '<' || x == '>').to_string();
        let mut sub_index = 0;
        let mut index = String::new();
        for ch in name.chars().rev() {
            if !ch.is_numeric() {
                break;
            }
            index.insert(0, ch);
        }
        if let Ok(num) = index.parse::<u32>() {
            let name_chars_count = name.chars().count();
            name.truncate(name_chars_count - index.len());
            sub_index = num;
        }
        Nonterminal::new(name, sub_index)
    }
    pub fn inc_sub_index(&self) -> Nonterminal {
        Nonterminal::new(self.name.to_owned(), self.sub_index + 1)
    }
}
impl fmt::Display for Nonterminal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.sub_index > 0 {
            write!(f, "<{}{}>", self.name, self.sub_index)
        } else {
            let chars = self.name.chars().collect::<Vec<char>>();
            if chars.len() == 1 && chars[0].is_uppercase() && chars[0].is_alphabetic() {
                write!(f, "{}", self.name)
            } else {
                write!(f, "<{}>", self.name)
            }
        }
    }
}

#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct Terminal {
    pub symbol: char,
}

impl Terminal {
    pub fn new(from: char) -> Terminal {
        Terminal { symbol: from }
    }
    pub fn is_a(&self, c: char) -> bool {
        self.symbol == c
    }
}

impl fmt::Display for Terminal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.symbol)
    }
}

#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub enum Symbol {
    N(Nonterminal),
    T(Terminal),
}

impl Symbol {
    pub fn new(c: String) -> Symbol {
        let chars: Vec<char> = c.chars().collect();
        if chars.len() != 1 || (chars[0].is_alphabetic() && chars[0].is_uppercase()) {
            Symbol::N(Nonterminal::parse(c))
        } else {
            Symbol::T(Terminal::new(chars[0]))
        }
    }
    pub fn is_nonterminal(&self) -> bool {
        match self {
            Symbol::T(_) => false,
            Symbol::N(_) => true,
        }
    }
    pub fn is_terminal(&self) -> bool {
        !self.is_nonterminal()
    }
    pub fn is_eq_term(&self, c: char) -> bool {
        match self {
            Symbol::T(ref t) => t.is_a(c),
            Symbol::N(_) => false,
        }
    }

    pub fn is_eq_nonterm(&self, other: &Nonterminal) -> bool {
        match self {
            Symbol::T(_) => false,
            Symbol::N(ref n) => n == other,
        }
    }

    pub fn as_nonterminal(&self) -> Option<&Nonterminal> {
        match self {
            Symbol::T(_) => None,
            Symbol::N(ref x) => Some(x),
        }
    }
    pub fn merge(set: &[Symbol]) -> Symbol {
        let name = set.iter().map(|x| x.to_string()).collect::<String>();
        Symbol::N(Nonterminal::new(name, 0))
    }
}

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Symbol::T(t) => write!(f, "{}", t),
            Symbol::N(n) => write!(f, "{}", n),
        }
    }
}

#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct Production {
    pub left: Nonterminal,
    pub right: Vec<Symbol>,
}

impl AsRef<Production> for Production {
    fn as_ref(&self) -> &Production {
        self
    }
}

impl Production {
    pub fn new(l: Nonterminal, r: Vec<Symbol>) -> Production {
        Production { left: l, right: r }
    }
}

#[derive(Debug, PartialEq)]
pub struct CFG {
    pub start: Nonterminal,
    pub productions: BTreeSet<Production>,
}
impl fmt::Display for CFG {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut rules: HashMap<Nonterminal, Vec<String>> = HashMap::new();
        for rule in self.productions.iter() {
            let mut chars = match rules.get(&rule.left) {
                Some(s) => s.clone(),
                None => Vec::new(),
            };
            chars.push(join(&rule.right, ""));
            rules.insert(rule.left.clone(), chars);
        }
        if let Some(mut start) = rules.remove(&self.start) {
            start.sort();
            writeln!(f, "{} -> {}", self.start, join(start, " | "))?;
        } else if rules.is_empty() {
            //eprintln!("Empty rule set: {:?}", self);
            return writeln!(f, "{} -> ", self.start);
        }
        for rule in self.productions.iter() {
            if let Some(mut val) = rules.remove(&rule.left) {
                val.sort();
                writeln!(f, "{} -> {}", rule.left, join(val, " | "))?;
            }
        }
        Ok(())
    }
}

impl CFG {
    pub fn new(start: Nonterminal, productions: BTreeSet<Production>) -> CFG {
        CFG { start, productions }
    }

    pub fn load(input_path: &str) -> io::Result<CFG> {
        let file = BufReader::new(File::open(input_path)?);
        CFG::load_from_reader(file)
    }

    #[allow(dead_code)]
    pub fn load_sdt(input_path: &str) -> io::Result<CFG> {
        let file = BufReader::new(File::open(input_path)?);
        CFG::load_sdt_from_reader(file)
    }

    pub fn load_from_reader<R: Sized + BufRead>(r: R) -> io::Result<CFG> {
        CFG::load_cfg_from_reader(r, false)
    }

    #[allow(dead_code)]
    pub fn load_sdt_from_reader<R: Sized + BufRead>(r: R) -> io::Result<CFG> {
        CFG::load_cfg_from_reader(r, true)
    }

    pub fn load_cfg_from_reader<R: Sized + BufRead>(r: R, sdt: bool) -> io::Result<CFG> {
        let mut start: Option<Nonterminal> = None;
        let mut productions = BTreeSet::new();
        for line in r.lines() {
            let text = line?;
            let rule = text.trim();
            if rule.is_empty() || rule.starts_with('#') {
                continue;
            }
            let add_productions = CFG::parse_production(rule, sdt)?;
            if productions.is_empty() {
                // The first valid rule is the start character here
                start = Some(add_productions[0].left.clone());
            }
            productions.extend(add_productions.into_iter());
        }
        if let Some(s) = start {
            Ok(CFG::new(s, productions))
        } else {
            Err(io::Error::new(io::ErrorKind::Other, "Don't see any rule"))
        }
    }

    pub fn parse_production(line: &str, _sdt: bool) -> io::Result<Vec<Production>> {
        let mut productions = Vec::new();
        let rule: Vec<&str> = line.split(" -> ").map(|x| x.trim()).collect();
        if rule.len() != 2 {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Bad rule: {}", line),
            ));
        }

        if rule[0].chars().count() == 0 {
            return Err(io::Error::new(io::ErrorKind::Other, "Missing left Symbol"));
        }
        let left = Symbol::new(rule[0].to_string());
        if left.is_terminal() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Terminal symbol at LHS: {}", line),
            ));
        }
        let left = left.as_nonterminal().unwrap();
        for rhs in rule[1].split('|').map(|x| x.trim()) {
            let symbols = CFG::parse_rhs(rhs)?;
            let prod = Production::new(left.clone(), symbols);
            productions.push(prod);
        }
        Ok(productions)
    }

    pub fn parse_rhs(rhs: &str) -> io::Result<Vec<Symbol>> {
        let mut name = String::new();
        let mut symbols = Vec::new();
        let mut read_long_name = false;
        for ch in rhs.chars() {
            if ch == '>' {
                if !read_long_name {
                    return Err(io::Error::new(
                        io::ErrorKind::Other,
                        "Unexpected symbol '>'",
                    ));
                }
                read_long_name = false;
            }
            if ch == '<' {
                if read_long_name {
                    return Err(io::Error::new(
                        io::ErrorKind::Other,
                        "Unexpected symbol '<'",
                    ));
                }
                read_long_name = true;
            }
            name.push(ch);
            if !read_long_name {
                symbols.push(Symbol::new(name.clone()));
                name.truncate(0);
            }
        }
        if read_long_name {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Unterminated Nonterminal symbol name, expect '>'",
            ));
        }
        Ok(symbols)
    }

    pub fn get_terminals(&self) -> HashSet<Terminal> {
        let mut term = HashSet::new();
        for rule in &self.productions {
            term.extend(
                rule.right
                    .iter()
                    .filter(|&x| !x.is_nonterminal())
                    .cloned()
                    .map(|x| match x {
                        Symbol::T(n) => n,
                        _ => unreachable!(),
                    })
                    .collect::<HashSet<Terminal>>(),
            );
        }
        term
    }

    pub fn get_variables(&self) -> BTreeSet<Nonterminal> {
        let mut vars = BTreeSet::new();
        for rule in &self.productions {
            vars.extend(
                rule.right
                    .iter()
                    .filter(|&x| x.is_nonterminal())
                    .cloned()
                    .map(|x| match x {
                        Symbol::N(n) => n,
                        _ => unreachable!(),
                    })
                    .collect::<HashSet<Nonterminal>>(),
            );
            vars.insert(rule.left.clone());
        }
        vars
    }

    pub fn get_nullable(&self) -> HashSet<Nonterminal> {
        let mut nullable: HashSet<Nonterminal> = HashSet::new();
        let mut changed = true;
        while changed {
            changed = false;
            for rule in &self.productions {
                // rule N -> epsilon or
                // if the rule contains only Nonterminal-s and they all lead to epsilon
                if (rule.right.is_empty()
                    || rule.right.iter().fold(true, |acc, x| {
                        if !acc {
                            acc
                        } else {
                            x.is_nonterminal() && nullable.contains(x.as_nonterminal().unwrap())
                        }
                    }))
                    && nullable.insert(rule.left.clone())
                {
                    changed = true;
                }
            }
        }
        nullable
    }

    pub fn simplify(&self) -> CFG {
        self.remove_epsilon_rules()
            .remove_unit_rules()
            .remove_useless_rules()
            .remove_unreachable_rules()
    }

    pub fn remove_epsilon_rules(&self) -> CFG {
        let nullable = self.get_nullable();

        let mut new_rules = BTreeSet::new();
        self.productions.iter().for_each(|rule| {
            if !rule.right.is_empty() {
                new_rules.insert(rule.clone());
            }
        });
        for rule in &self.productions {
            if rule
                .right
                .iter()
                .any(|x| x.is_nonterminal() && nullable.contains(x.as_nonterminal().unwrap()))
            {
                new_rules.insert(Production::new(rule.left.clone(), rule.right.clone()));
                let mut source = new_rules.clone();
                let mut source2 = BTreeSet::new();
                let mut changed = true;
                while changed {
                    changed = false;
                    for r in &source {
                        for (idx, sym) in r.right.iter().enumerate() {
                            if sym.is_nonterminal()
                                && nullable.contains(sym.as_nonterminal().unwrap())
                            {
                                let mut new = r.clone();
                                new.right.remove(idx);
                                // skip new epsilon rule and skip new unit rule
                                if !(new.right.is_empty()
                                    || new.right.len() == 1
                                        && new.right[0].is_nonterminal()
                                        && new.right[0].as_nonterminal().unwrap() == &new.left
                                    || !new_rules.insert(new.clone()))
                                {
                                    changed = true;
                                    source2.insert(new);
                                }
                            }
                        }
                    }
                    source = source2.clone();
                }
            }
        }
        let mut start = self.start.clone();
        // if ε in L(G) add 'S -> ε'
        if nullable.contains(&self.start) {
            // if S in right hand side of any rule
            // instead 'S -> ε' add 'S1 -> S | ε'
            let cfg = self.remove_start_from_rhs();
            if start != cfg.start {
                new_rules.insert(Production::new(cfg.start.clone(), vec![Symbol::N(start)]));
                start = cfg.start
            }
            new_rules.insert(Production::new(start.clone(), Vec::new()));
        }
        CFG::new(start, new_rules)
    }

    pub fn remove_unit_rules(&self) -> CFG {
        let mut unit_sets = self
            .get_variables()
            .iter()
            .cloned()
            .map(|x| (x.clone(), vec![x].into_iter().collect()))
            .collect::<HashMap<Nonterminal, HashSet<Nonterminal>>>();

        for nonterm in &self.get_variables() {
            let set = unit_sets.get_mut(nonterm).unwrap();
            let mut changed = true;
            while changed {
                changed = false;
                for rule in &self.productions {
                    if rule.right.len() == 1
                        && rule.right[0].is_nonterminal()
                        && set.contains(&rule.left)
                    {
                        // add rule.right<Nonterminal> into unit_sets[rule.left]{} set
                        let right = rule.right[0].as_nonterminal().unwrap();
                        if set.insert(right.clone()) {
                            changed = true
                        }
                    }
                }
            }
            set.remove(nonterm);
        }
        let rules = self
            .productions
            .iter()
            .filter(|x| !(x.right.len() == 1 && x.right[0].is_nonterminal()))
            .cloned()
            .collect::<BTreeSet<Production>>();
        let mut new_rules = rules.clone();
        for (k, v) in &unit_sets {
            for rule in &rules {
                if v.contains(&rule.left) {
                    new_rules.insert(Production::new(k.to_owned(), rule.right.to_owned()));
                }
            }
        }
        let mut changed = true;
        while changed {
            changed = false;
        }
        CFG::new(self.start.clone(), new_rules)
    }

    pub fn remove_useless_rules(&self) -> CFG {
        let mut usefull_nonterminals = BTreeSet::new();
        let mut changed = true;
        while changed {
            changed = false;
            for rule in &self.productions {
                let right_nonterm_set: BTreeSet<Nonterminal> = rule
                    .right
                    .iter()
                    .filter(|&x| x.is_nonterminal())
                    .cloned()
                    .map(|x| match x {
                        Symbol::N(n) => n,
                        _ => unreachable!(),
                    })
                    .collect();
                if right_nonterm_set.is_empty()
                    || right_nonterm_set.is_subset(&usefull_nonterminals)
                {
                    // if rule contains only terminals or all Nonterminals can be generated
                    if usefull_nonterminals.insert(rule.left.clone()) {
                        changed = true;
                    }
                }
            }
        }
        let mut productions = BTreeSet::new();
        for rule in &self.productions {
            let right_nonterm_set: BTreeSet<Nonterminal> = rule
                .right
                .iter()
                .filter(|&x| x.is_nonterminal())
                .cloned()
                .map(|x| match x {
                    Symbol::N(n) => n,
                    _ => unreachable!(),
                })
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
        let mut productions = BTreeSet::new();
        for rule in &self.productions {
            let mut right_set: HashSet<Symbol> = rule.right.iter().cloned().collect();
            right_set.insert(Symbol::N(rule.left.clone()));
            if right_set.is_subset(&reachable_symbols) {
                productions.insert(rule.clone());
            }
        }
        CFG::new(self.start.clone(), productions)
    }

    pub fn remove_start_from_rhs(&self) -> CFG {
        let mut start = self.start.clone();
        let mut productions = self.productions.clone();

        let start_in_rhs = self.productions.iter().any(|rule| {
            rule.right
                .iter()
                .any(|x| x.as_nonterminal() == Some(&self.start))
        });
        if start_in_rhs {
            let vars = self.get_variables();
            start = start.inc_sub_index();
            while vars.contains(&start) {
                start = start.inc_sub_index();
            }
            productions.insert(Production::new(
                start.clone(),
                vec![Symbol::N(self.start.clone())],
            ));
        }
        CFG::new(start, productions)
    }

    /*
    pub fn add_new_start(&self) -> CFG {
        let new_start = self.start.inc_sub_index();
        let mut new_rule = Production::new(new_start.clone(), vec![Symbol::N(self.start.clone())]);
        let mut productions = self.productions.clone();
        while !productions.insert(new_rule.clone()) {
            new_rule.left = new_rule.left.inc_sub_index();
        }

        CFG::new(new_rule.left.clone(), productions)
    }
    */

    pub fn is_normal_form(&self) -> Option<String> {
        if self != &self.remove_start_from_rhs() {
            Some(format!(
                "The 'Start ({})' character is present in the right part of the rules",
                self.start
            ))
        } else if self != &self.remove_start_from_rhs().remove_epsilon_rules() {
            Some("Epsilon rules are not excluded from grammar".into())
        } else if self
            != &self
                .remove_start_from_rhs()
                .remove_epsilon_rules()
                .remove_unit_rules()
        {
            Some("There are Unit rules in the grammar".into())
        } else if self
            != &self
                .remove_start_from_rhs()
                .remove_epsilon_rules()
                .remove_unit_rules()
                .remove_useless_rules()
        {
            Some("There are non-generating characters in the grammar".into())
        } else if self
            != &self
                .remove_start_from_rhs()
                .remove_epsilon_rules()
                .remove_unit_rules()
                .remove_useless_rules()
                .remove_unreachable_rules()
        {
            Some("There are unreachable characters in the grammar".into())
        } else {
            None
        }
    }

    pub fn chomsky(&self) -> CFG {
        let cfg = self
            .remove_start_from_rhs()
            .remove_epsilon_rules()
            .remove_unit_rules()
            .remove_useless_rules()
            .remove_unreachable_rules();

        // Eliminate all rules having more than two symbols on the right-hand side.
        let mut new_productions = BTreeSet::new();
        for rule in cfg.productions {
            if rule.right.len() <= 2 {
                new_productions.insert(rule.clone());
                continue;
            }
            let mut split = rule.right.split_at(1);
            let mut left = Symbol::merge(split.1);
            new_productions.insert(Production::new(
                rule.left.clone(),
                vec![split.0[0].clone(), left.clone()],
            ));
            loop {
                if split.1.len() == 2 {
                    new_productions.insert(Production::new(
                        left.as_nonterminal().unwrap().to_owned(),
                        split.1.to_vec(),
                    ));
                    break;
                }
                split = split.1.split_at(1);
                let mut new_rule =
                    Production::new(left.as_nonterminal().unwrap().to_owned(), split.0.to_vec());
                left = Symbol::merge(split.1);
                new_rule.right.push(left.clone());
                new_productions.insert(new_rule);
            }
        }

        // Eliminate all rules of the form A →  u₁u₂,
        // where u₁ and u₂ are not both variables.
        let mut productions = BTreeSet::new();
        for rule in new_productions {
            if rule.right.iter().all(|x| x.is_nonterminal())
                || rule.right.len() == 1 && rule.right[0].is_terminal()
            {
                productions.insert(rule);
            } else {
                let mut new_rule = rule.clone();
                for (idx, sym) in rule.right.into_iter().enumerate() {
                    if sym.is_terminal() {
                        let left = Nonterminal::new(format!("{}", sym), 0);
                        productions.insert(Production::new(left.clone(), vec![sym]));
                        new_rule.right[idx] = Symbol::N(left);
                    }
                }
                productions.insert(new_rule);
            }
        }
        CFG::new(cfg.start, productions)
    }

    pub fn greibach(&self) -> CFG {
        let cfg = self.chomsky();
        let cfg = cfg.eliminate_left_recursion();
        CFG::new(cfg.start.clone(), cfg.productions.clone())
    }

    pub fn eliminate_left_recursion(&self) -> CFG {
        CFG::new(self.start.clone(), self.productions.clone())
    }
}

#[cfg(test)]
mod tests {
    use self::super::*;
    use std::io::Cursor;

    #[test]
    fn load_cfg() {
        let productions = vec![
            Production::new(
                Nonterminal::new("S".to_string(), 2),
                vec![
                    Symbol::N(Nonterminal::new("S".to_string(), 1)),
                    Symbol::N(Nonterminal::new("Some".to_string(), 0)),
                    Symbol::T(Terminal { symbol: 'a' }),
                ],
            ),
            Production::new(
                Nonterminal::new("S".to_string(), 2),
                vec![
                    Symbol::N(Nonterminal::new("s".to_string(), 0)),
                    Symbol::N(Nonterminal::new("S".to_string(), 0)),
                    Symbol::T(Terminal { symbol: 'a' }),
                ],
            ),
        ];
        let expected = CFG {
            start: productions[0].left.clone(),
            productions: productions.into_iter().collect(),
        };
        let test_definition = "<S2> -> <S1><Some>a | <s>Sa\n";
        let cfg = CFG::load_from_reader(Cursor::new(test_definition)).unwrap();
        assert_eq!(cfg.start, expected.start);
        assert_eq!(cfg.productions, expected.productions);
        assert_eq!(format!("{}", cfg), test_definition);
        let text = Cursor::new("<a> -> ||||");
        assert!(CFG::load_from_reader(text).is_ok());
    }

    #[test]
    fn load_mailformed_cfg() {
        let text = Cursor::new("S -> <");
        assert!(CFG::load_from_reader(text).is_err(), "Eat unexpected '<'");
        let text = Cursor::new("S -> <<a>");
        assert!(CFG::load_from_reader(text).is_err(), "Eat unexpected '<'");
        let text = Cursor::new("S -> >");
        assert!(CFG::load_from_reader(text).is_err(), "Eat unexpected '>'");
        let text = Cursor::new("S -> <a>>");
        assert!(CFG::load_from_reader(text).is_err(), "Eat unexpected '>'");
        let text = Cursor::new(" -> <a>");
        assert!(CFG::load_from_reader(text).is_err(), "Missing left Symbol");
        let text = Cursor::new("a -> ");
        assert!(CFG::load_from_reader(text).is_err(), "Terminal at LHS");
    }

    #[test]
    fn remove_epsilon() {
        let test_rules = r#"
            S -> AaB | aB | cC
            A -> AB | a | b | B
            B -> Ba |
            C -> AB | c
        "#;
        let expected = format!(
            "{}\n",
            join(
                vec![
                    "S -> Aa | AaB | a | aB | c | cC",
                    "A -> AB | B | a | b",
                    "B -> Ba | a",
                    "C -> A | AB | B | c",
                ],
                "\n"
            )
        );
        let cfg = CFG::load_from_reader(Cursor::new(test_rules)).unwrap();
        assert_eq!(format!("{}", cfg.remove_epsilon_rules()), expected);
    }

    #[test]
    fn remove_units() {
        let test_rules = "
            Я -> AaB | aB | cC
            A -> AB | a | b | B
            B -> Ba |
            C -> AB | c
        ";
        let expected = format!(
            "{}\n",
            join(
                vec![
                    "Я -> Aa | AaB | a | aB | c | cC",
                    "A -> AB | Ba | a | b",
                    "B -> Ba | a",
                    "C -> AB | Ba | a | b | c",
                ],
                "\n"
            )
        );

        let cfg = CFG::load_from_reader(Cursor::new(test_rules))
            .unwrap()
            .remove_epsilon_rules();
        assert_eq!(format!("{}", cfg.remove_unit_rules()), expected);

        let test_rules = "
            E -> T | E+T
            F -> I | (E)
            I -> a | b | Ia | Ib | I0 | I1
            T -> F | T*F
        ";
        let expected = format!(
            "{}\n",
            join(
                vec![
                    "E -> (E) | E+T | I0 | I1 | Ia | Ib | T*F | a | b",
                    "F -> (E) | I0 | I1 | Ia | Ib | a | b",
                    "I -> I0 | I1 | Ia | Ib | a | b",
                    "T -> (E) | I0 | I1 | Ia | Ib | T*F | a | b",
                ],
                "\n"
            )
        );
        let cfg = CFG::load_from_reader(Cursor::new(test_rules)).unwrap();
        assert_eq!(format!("{}", cfg.remove_unit_rules()), expected);
    }

    #[test]
    fn remove_useless() {
        let test_rules = "
            S -> aAB | E
            A -> aA | bB
            B -> ACb| b
            C -> A | bA | cC | aE
            D -> a | c | Fb
            E -> cE | aE | Eb | ED | FG
            F -> BC | EC | AC
            G -> Ga | Gb
        ";
        let expected = format!(
            "{}\n",
            join(
                vec![
                    "S -> aAB",
                    "A -> aA | bB",
                    "B -> ACb | b",
                    "C -> A | bA | cC",
                    "D -> Fb | a | c",
                    "F -> AC | BC",
                ],
                "\n"
            )
        );
        let cfg = CFG::load_from_reader(Cursor::new(test_rules)).unwrap();
        assert_eq!(format!("{}", cfg.remove_useless_rules()), expected);
    }

    #[test]
    fn remove_unreachable() {
        let test_rules = "
            S -> aAB | E
            A -> aA | bB
            B -> ACb| b
            C -> A | bA | cC | aE
            D -> a | c | Fb
            E -> cE | aE | Eb | ED | FG
            F -> BC | EC | AC
            G -> Ga | Gb
        ";
        let expected = format!(
            "{}\n",
            join(
                vec![
                    "S -> aAB",
                    "A -> aA | bB",
                    "B -> ACb | b",
                    "C -> A | bA | cC",
                ],
                "\n"
            )
        );
        let cfg = CFG::load_from_reader(Cursor::new(test_rules))
            .unwrap()
            .remove_useless_rules();
        assert_eq!(format!("{}", cfg.remove_unreachable_rules()), expected);
    }

    #[test]
    fn simplify() {
        let test_rules = "
            S ->  | S(S)S
        ";
        let expected = format!(
            "{}\n",
            join(
                vec![
                    "<S1> ->  | () | ()S | (S) | (S)S | S() | S()S | S(S) | S(S)S",
                    "S -> () | ()S | (S) | (S)S | S() | S()S | S(S) | S(S)S",
                ],
                "\n"
            )
        );
        let cfg = CFG::load_from_reader(Cursor::new(test_rules)).unwrap();
        assert_eq!(format!("{}", cfg.simplify()), expected);
    }

    #[test]
    fn chomsky() {
        let test_rules = "
            A ->  BAB | B |
            B -> 00 |
        ";
        let expected = format!(
            "{}\n",
            join(
                vec![
                    "<A1> ->  | <0><0> | AB | B<AB> | BA | BB",
                    "<0> -> 0",
                    "A -> <0><0> | AB | B<AB> | BA | BB",
                    "<AB> -> AB",
                    "B -> <0><0>",
                ],
                "\n"
            )
        );
        let cfg = CFG::load_from_reader(Cursor::new(test_rules)).unwrap();
        assert_eq!(format!("{}", cfg.chomsky()), expected);
    }
}
