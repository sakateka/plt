use crate::cfg;
use std::collections::HashSet;
use std::fmt;

#[derive(Debug, Hash, PartialEq, Clone)]
pub struct State<'er> {
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

pub struct Column<'er> {
    states: HashSet<State<'er>>,
    token: char,
    #[allow(unused)]
    index: usize,
}
impl<'er> Column<'er> {
    pub fn new(token: char, index: usize) -> Column<'er> {
        Column {
            states: HashSet::new(),
            token: token,
            index: index,
        }
    }
    pub fn from(states: HashSet<State<'er>>, token: char, index: usize) -> Column<'er> {
        Column {
            states: states,
            token: token,
            index: index,
        }
    }
    pub fn len(&self) -> usize {
        self.states.len()
    }
    pub fn insert(&mut self, state: State<'er>) -> bool {
        self.states.insert(state)
    }
}

impl<'er> EarleyParser<'er> {
    pub fn new(grammar: &'er cfg::CFG) -> EarleyParser<'er> {
        EarleyParser { cfg: grammar }
    }

    fn init_states(&self, text: &str) -> Vec<Column<'er>> {
        // "\0" - gamma
        "\0".chars()
            .chain(text.chars())
            .enumerate()
            .map(|x| {
                if x.0 == 0 {
                    Column::from(
                        self.cfg
                            .productions
                            .iter()
                            .filter(|x| x.left == self.cfg.start)
                            .map(|x| State {
                                rule: x,
                                dot: 0,
                                origin: 0,
                            })
                            .collect(),
                        x.1,
                        x.0,
                    )
                } else {
                    Column::new(x.1, x.0)
                }
            })
            .collect::<Vec<_>>()
    }

    pub fn parse(&self, text: &str) -> Vec<Column> {
        let mut chart = self.init_states(text);
        let chart_len = chart.len();
        for idx in 0..chart_len {
            let mut changed = true;
            while changed {
                changed = false;
                let links: Vec<_> = chart[idx].states.iter().cloned().collect();
                for state in &links {
                    if state.finished() {
                        self.completer(state, idx, &mut chart);
                    } else {
                        if state.nonterminal_here() {
                            self.predictor(state, idx, &mut chart[idx]);
                        } else if idx < chart_len - 1 {
                            self.scaner(state, &mut chart[idx + 1]);
                        }
                    }
                }
                if links.len() != chart[idx].len() {
                    changed = true;
                }
            }
        }
        chart
    }
    fn completer(&self, state: &State<'er>, idx: usize, chart: &mut Vec<Column<'er>>) {
        let links: Vec<_> = chart[state.origin].states.iter().cloned().collect();
        for r in links {
            if let Some(sym) = r.symbol() {
                if sym.is_eq_nonterm(&state.rule.left) {
                    chart[idx].insert(r.shift());
                }
            }
        }
    }
    fn predictor(&self, state: &State, origin: usize, states: &mut Column<'er>) {
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
    fn scaner(&self, state: &State<'er>, states: &mut Column<'er>) {
        if let Some(sym) = state.symbol() {
            if sym.is_eq_term(states.token) {
                states.insert(state.shift());
            }
        }
    }
    pub fn print(&self, chart: &Vec<Column<'er>>) -> bool {
        let mut ret = false;
        let mut parsed = String::new();
        println!("CFG.Start: {}", self.cfg.start);
        for (idx, column) in chart.iter().enumerate() {
            parsed.push(column.token);
            let accepts = column
                .states
                .iter()
                .any(|s| s.rule.left == self.cfg.start && s.finished() && s.origin == 0);
            print!("({}) {} ", idx, parsed);
            if accepts {
                if idx != chart.len() - 1 {
                    print!("PARTIAL ");
                } else {
                    ret = true
                }
                println!("ACCEPTS");
            } else {
                println!("");
            }
            for i in &column.states {
                println!("\t{}", i);
            }
        }
        ret
    }

    /*
    pub fn derivation_path(&self, chart: &Vec<Column<'er>>) -> Option<Vec<&'er cfg::Production>> {
        let mut chart_idx = chart.len();
        if chart_idx == 0 {
            eprintln!("Parsing failed");
            return None;
        }

        let mut path: Vec<State> = Vec::new();
        let completed = chart.iter().last().and_then(|last| {
            last.states
                .iter()
                .filter(|x| x.finished())
                .max_by(|a, b| a.dot.cmp(&b.dot))
        });
        if let Some(item) = completed {
            path.push(item.clone());
            let mut step = chart.len();
            let mut item = item;
            for sym in item.rule.right.iter().rev() {
                if sym.is_nonterminal() {
                    let completed = chart[chart_idx]
                        .states
                        .iter()
                        .filter(|x| Some(&x.rule.left) == sym.as_nonterminal()  && x.finished())
                        .max_by(|a, b| a.dot.cmp(&b.dot)).unwrap();

                    if completed.rule.right.len() == 0 {
                        path.push(completed.clone());
                        continue
                    }
                } else {
                    chart_idx -= 1;
                }
            }
        }
        eprintln!("Parsing failed");
        None
    }
    */
}
