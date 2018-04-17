use cfg;
use std::collections::{HashMap, HashSet};
use std::ops::{Index, IndexMut};

struct CYKTable<'cyk>(
    Vec<Vec<HashSet<&'cyk cfg::Nonterminal>>>,
    HashMap<&'cyk cfg::Nonterminal, HashSet<&'cyk cfg::Production>>,
);
impl<'cyk> Index<usize> for CYKTable<'cyk> {
    type Output = Vec<HashSet<&'cyk cfg::Nonterminal>>;
    fn index<'a>(&'a self, index: usize) -> &'a Vec<HashSet<&'cyk cfg::Nonterminal>> {
        &self.0[index]
    }
}

impl<'cyk> IndexMut<usize> for CYKTable<'cyk> {
    fn index_mut<'a>(&'a mut self, index: usize) -> &'a mut Vec<HashSet<&'cyk cfg::Nonterminal>> {
        &mut self.0[index]
    }
}
impl<'cyk> CYKTable<'cyk> {
    fn new(len: usize) -> CYKTable<'cyk> {
        CYKTable(vec![vec![HashSet::new(); len]; len], HashMap::new())
    }
}

pub type CYKParsePath<'cyk> = Vec<&'cyk cfg::Production>;

#[derive(Debug)]
pub struct CYKParser {
    cfg: cfg::CFG,
}

impl CYKParser {
    pub fn new(grammar: &cfg::CFG) -> CYKParser {
        CYKParser {
            cfg: grammar.chomsky(),
        }
    }
    fn build_recognizer_table(&self, text: &str) -> CYKTable {
        let text_len = text.chars().count();
        let mut table = CYKTable::new(text_len);

        for rule in &self.cfg.productions {
            table
                .1
                .entry(&rule.left)
                .or_insert(HashSet::new())
                .insert(&rule);
            for (idx, ch) in text.chars().enumerate() {
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
                    for rule in &self.cfg.productions {
                        if rule.right.len() != 2 {
                            continue;
                        }
                        // assert!(rule.right.iter().all(|x| x.is_nonterminal()));
                        let n1_set: Vec<_> = table[r][r + t].iter().cloned().collect();
                        let n2_set: Vec<_> = table[r + t + 1][r + l].iter().cloned().collect();

                        for n1 in &n1_set {
                            for n2 in &n2_set {
                                if rule.right[0].is_eq_nonterm(n1)
                                    && rule.right[1].is_eq_nonterm(n2)
                                {
                                    table[r][r + l].insert(&rule.left);
                                }
                            }
                        }
                    }
                }
            }
        }
        table
    }

    fn accepts_by_epsilon(&self) -> Option<&cfg::Production> {
        // special case for empty string
        for rule in &self.cfg.productions {
            if rule.left == self.cfg.start {
                if rule.right.len() == 0 {
                    return Some(rule);
                }
            }
        }
        None
    }

    pub fn accepts(&self, text: &str) -> bool {
        let text_len = text.chars().count();
        if text_len == 0 {
            return self.accepts_by_epsilon().is_some();
        }
        let table = self.build_recognizer_table(text);
        table[0][text_len - 1].contains(&self.cfg.start)
    }

    fn build_parser_table(&self, text: &str) -> CYKTable {
        let text_len = text.chars().count();
        let mut table = CYKTable::new(text_len);

        for rule in &self.cfg.productions {
            table
                .1
                .entry(&rule.left)
                .or_insert(HashSet::new())
                .insert(&rule);
            for (i, ch) in text.chars().enumerate() {
                if rule.right.len() == 1 && rule.right[0].is_eq_term(ch) {
                    table[i][0].insert(&rule.left);
                }
            }
        }
        let mut changed = true;
        while changed {
            changed = false;
            for i in 0..text_len {
                for j in 1..(text_len - i) {
                    for k in 0..j {
                        for rule in &self.cfg.productions {
                            if rule.right.len() != 2 {
                                continue;
                            }
                            // assert!(rule.right.iter().all(|x| x.is_nonterminal()));
                            let n1_set: Vec<_> = table[i][k].iter().cloned().collect();
                            let n2_set: Vec<_> =
                                table[i + k + 1][j - k - 1].iter().cloned().collect();

                            for n1 in &n1_set {
                                for n2 in &n2_set {
                                    if rule.right[0].is_eq_nonterm(n1)
                                        && rule.right[1].is_eq_nonterm(n2)
                                    {
                                        if table[i][j].insert(&rule.left) {
                                            changed = true;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        table
    }

    pub fn parse(&self, text: &str) -> Option<CYKParsePath> {
        let text_len = text.chars().count();
        if text_len == 0 {
            if let Some(rule) = self.accepts_by_epsilon() {
                return Some(vec![rule]);
            } else {
                return None;
            }
        }
        let mut path = None;
        let chars: Vec<_> = text.chars().collect();
        let table = self.build_parser_table(text);

        /*
        use itertools::join;
        table.0.iter().for_each(|x| {
            println!("");
            x.iter().for_each(|y| print!("{{{:12}}}", join(y, ",")));
        });
        table.1.iter().for_each(|x| {
            print!("\n{}: ", x.0);
            x.1.iter().for_each(|y| print!("{} | ", join(&y.right, "")));
        });
        */

        let last_index = text_len - 1;
        if table[0][last_index].contains(&self.cfg.start) {
            let mut path_in = Vec::new();
            self.build_path(&chars, 0, last_index, &self.cfg.start, &table, &mut path_in);
            path = Some(path_in);
        }
        path
    }

    fn build_path<'cyk>(
        &self,
        chars: &Vec<char>,
        i: usize,
        j: usize,
        nonterm: &cfg::Nonterminal,
        table: &CYKTable<'cyk>,
        path: &mut CYKParsePath<'cyk>,
    ) {
        if j == 0 {
            if let Some(rule) = self.rule_with_terminal(chars[i], nonterm, table) {
                path.push(rule)
            }
        } else if j > 0 {
            if let Some((rule, k)) = self.rule_with_nonterminals(i, j, nonterm, table) {
                path.push(rule);
                let n1 = rule.right[0].as_nonterminal().unwrap();
                let n2 = rule.right[1].as_nonterminal().unwrap();
                self.build_path(chars, i, k, n1, table, path);
                self.build_path(chars, i + k + 1, j - k - 1, n2, table, path);
            }
        }
    }

    fn rule_with_terminal<'cyk>(
        &self,
        term: char,
        nonterm: &cfg::Nonterminal,
        table: &CYKTable<'cyk>,
    ) -> Option<&'cyk cfg::Production> {
        if let Some(rules) = table.1.get(nonterm) {
            for rule in rules {
                if rule.right[0].is_eq_term(term) {
                    return Some(rule);
                }
            }
        }
        None
    }

    fn rule_with_nonterminals<'cyk>(
        &self,
        i: usize,
        j: usize,
        nonterm: &cfg::Nonterminal,
        table: &CYKTable<'cyk>,
    ) -> Option<(&'cyk cfg::Production, usize)> {
        if let Some(rules) = table.1.get(nonterm) {
            for k in 0..j {
                let set_a = &table[i][k];
                let set_b = &table[i + k + 1][j - k - 1];

                for rule in rules {
                    if rule.right.len() == 1 {
                        continue;
                    }
                    let n1 = rule.right[0].as_nonterminal().unwrap();
                    let n2 = rule.right[1].as_nonterminal().unwrap();
                    if set_a.contains(n1) && set_b.contains(n2) {
                        return Some((rule, k as usize));
                    }
                }
            }
        }
        None
    }
}
