use cfg;
use std::hash::{Hash, Hasher};
use std::collections::HashSet;

#[derive(Clone)]
pub struct CYKItem<'cyk> {
    var: &'cyk cfg::Nonterminal,
    rule: Option<&'cyk Vec<cfg::Symbol>>,
}
impl<'cyk> Hash for CYKItem<'cyk> {
    fn hash<H: Hasher>(&self, hasher_state: &mut H) {
        self.var.hash(hasher_state);
    }
}
impl<'cyk> PartialEq for CYKItem<'cyk> {
    fn eq(&self, other: &CYKItem) -> bool {
        self.var == other.var
    }
}
impl<'cyk> Eq for CYKItem<'cyk> {}

impl<'cyk> CYKItem<'cyk> {
    pub fn new(rule: &'cyk cfg::Production) -> CYKItem {
        CYKItem{
            var: &rule.left,
            rule: Some(&rule.right),
        }
    }
    fn from_nonterminal(var: &'cyk cfg::Nonterminal) -> CYKItem {
        CYKItem {
            var: var,
            rule: None,
        }
    }
}


type CYKTable<'cyk> = Vec<Vec<HashSet<CYKItem<'cyk>>>>;

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
    fn build_table(&self, text: &str) ->  CYKTable {
        let text_len = text.chars().count();
        let mut table = vec![vec![HashSet::new(); text_len]; text_len];

        for (idx, ch) in text.chars().enumerate() {
            for rule in &self.cfg.productions {
                if rule.right.len() == 1 && rule.right[0].is_eq_term(ch) {
                    table[idx][idx].insert(CYKItem::new(&rule));
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
                        let n1_set: Vec<CYKItem> = table[r][r + t].iter().cloned().collect();
                        let n2_set: Vec<CYKItem> = table[r + t + 1][r + l].iter().cloned().collect();

                        for n1 in &n1_set {
                            for n2 in &n2_set {
                                if rule.right[0].is_eq_nonterm(n1.var)
                                    && rule.right[1].is_eq_nonterm(n2.var)
                                {
                                    table[r][r + l].insert(CYKItem::new(&rule));
                                }
                            }
                        }
                    }
                }
            }
        }
        table
    }

    pub fn accepts(&self, text: &str) -> bool {
        let text_len = text.chars().count();
        if text_len == 0 {
            // special case for empty string
            for rule in &self.cfg.productions {
                if rule.left == self.cfg.start {
                    if rule.right.len() == 0 {
                        return true;
                    }
                }
            }
            return false;
        }
        let table = self.build_table(text);

        let accepted_item = CYKItem::from_nonterminal(&self.cfg.start);
        table[0][text_len - 1].contains(&accepted_item)
    }
}
