use std::io::{self, BufRead, BufReader};
use std::fs::File;
use std::collections::HashSet;

#[derive(Debug, Hash, PartialEq)]
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

#[derive(Debug, Hash, PartialEq)]
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


#[derive(Debug, Hash, PartialEq)]
pub enum Symbol {
    N(Nonterminal),
    T(Terminal),
}

#[derive(Debug, Hash, PartialEq)]
pub struct Production {
    pub left: Nonterminal,
    pub right: Vec<Symbol>,
}

impl Eq for Production {}

pub struct CFG {
    pub start: Nonterminal,
    pub productions: HashSet<Production>,
    pub categories: HashSet<Nonterminal>,
}

impl CFG {
    pub fn parse(inputPath: &str) -> io::Result<CFG> {
        let mut cfg: CFG;
        let mut file = BufReader::new(
            File::open(inputPath).expect(
                format!("opening file {}", inputPath).as_ref()
            )
        );
        let mut it = file.lines();
        match it.next() {
            Some(s) => {
                println!("{:?}", s);
            }
            None => {
                return Err(io::Error::new(io::ErrorKind::Other, "Don't see any rule",));
            }
        }
        for line in it {
            println!("{:?}", line);
        }
        Ok(CFG{
            start: Nonterminal::new('a'),
            productions: HashSet::new(),
            categories: HashSet::new(),
        })
    }
}
