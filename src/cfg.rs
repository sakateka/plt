#[derive(Debug, Hash)]
pub struct Nonterminal {
    pub symbol: String,
}

#[derive(Debug, Hash)]
pub struct Terminal {
    pub symbol: String,
}

#[derive(Debug, Hash)]
pub enum Symbol {
    N(Nonterminal),
    T(Terminal),
}

#[derive(Debug, Hash)]
pub struct Production {
    pub left: Nonterminal,
    pub right: Vec<Symbol>,
}

#[derive(Debug, Hash)]
pub struct CFG {
    pub start: Nonterminal,
    pub productions: Vec<Production>,
    pub categories: Vec<Nonterminal>,
}
