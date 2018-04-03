use cfg;
use std::collections::HashSet;
use std::fmt;

#[derive(Debug, Hash, PartialEq, Clone)]
struct State<'er> {
    rule: &'er cfg::Production,
    dot: usize,
    origin: usize,
}
impl<'er> Eq for State<'er> {}
impl<'er> fmt::Display for State<'er> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "[{}] {} -> {}.{}",
            self.origin,
            self.rule.left,
            self.rule.right[..self.dot]
                .iter()
                .fold(String::new(), |acc, x| format!("{}{}", acc, x)),
            self.rule.right[self.dot..]
                .iter()
                .fold(String::new(), |acc, x| format!("{}{}", acc, x)),
        )
    }
}

impl<'er> State<'er> {
    fn new(rule: &'er cfg::Production, origin: usize) -> State<'er> {
        State {
            rule: rule,
            dot: 0,
            origin: origin,
        }
    }
    fn finished(&self) -> bool {
        self.dot >= self.rule.right.len()
    }

    fn symbol(&self) -> Option<&'er cfg::Symbol> {
        if self.finished() {
            None
        } else {
            Some(&self.rule.right[self.dot])
        }
    }

    fn nonterminal_here(&self) -> bool {
        match self.symbol() {
            None => false,
            Some(ref sym) => sym.is_nonterminal(),
        }
    }

    fn shift(&self) -> State<'er> {
        State {
            dot: self.dot + 1,
            rule: self.rule,
            origin: self.origin,
        }
    }
}

pub struct EarleyParser<'er> {
    cfg: &'er cfg::CFG,
}

impl<'er> EarleyParser<'er> {
    pub fn new(grammar: &'er cfg::CFG) -> EarleyParser<'er> {
        EarleyParser { cfg: grammar }
    }

    fn init_states(&self, len: usize) -> Vec<HashSet<State>> {
        (0..(len + 2))
            .map(|x| {
                if x == 0 {
                    self.cfg.productions
                        .iter()
                        .filter(|x| x.left == self.cfg.start)
                        .map(|x| State {
                            rule: x,
                            dot: 0,
                            origin: 0,
                        })
                        .collect::<HashSet<_>>()
                } else {
                    HashSet::new()
                }
            })
            .collect::<Vec<_>>()
    }

    pub fn parse(&self, text: String) -> bool {
        let mut chart = self.init_states(text.chars().count());
        for (idx, letter) in text.chars().chain("\0".chars()).enumerate() {
            let mut changed = true;
            while changed {
                changed = false;
                let links: Vec<_> = chart[idx].iter().cloned().collect();
                for state in &links {
                    if state.finished() {
                        self.completer(state, idx, &mut chart);
                    } else {
                        if state.nonterminal_here() {
                            self.predictor(state, idx, &mut chart[idx]);
                        } else {
                            self.scaner(state, letter, &mut chart[idx + 1]);
                        }
                    }
                }
                if links.len() != chart[idx].len() {
                    changed = true;
                }
            }
        }
        let mut ret = false;
        for (idx, state) in chart.iter().take(text.len() + 1).enumerate() {
            let accepts = state
                .iter()
                .any(|s| s.rule.left == self.cfg.start && s.finished());
            let (parsed, unparsed) = text.split_at(idx);
            print!("({}) {}.{} ", idx, parsed, unparsed);
            if accepts {
                if !ret {
                    ret = true;
                }
                println!("ACCEPTS");
            } else {
                println!("");
            }
            for i in state {
                println!("\t{}", i);
            }
        }
        ret
    }
    fn completer(&self, state: &State<'er>, idx: usize, chart: &mut Vec<HashSet<State<'er>>>) {
        let links: Vec<_> = chart[state.origin].iter().cloned().collect();
        for r in links {
            if let Some(sym) = r.symbol() {
                if sym.is_eq_nonterm(&state.rule.left) {
                    chart[idx].insert(r.shift());
                }
            }
        }
    }
    fn predictor(&self, state: &State, origin: usize, states: &mut HashSet<State<'er>>) {
        self.cfg
            .productions
            .iter()
            .filter(|r| match state.symbol().unwrap() {
                &cfg::Symbol::N(ref n) => n == &r.left,
                &cfg::Symbol::T(_) => false,
            })
            .for_each(|r| {
                states.insert(State::new(r, origin));
            });
    }
    fn scaner(&self, state: &State<'er>, letter: char, states: &mut HashSet<State<'er>>) {
        if let Some(sym) = state.symbol() {
            if sym.is_eq_term(letter) {
                states.insert(state.shift());
            }
        }
    }
}
