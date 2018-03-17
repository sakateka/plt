use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Hash, PartialEq)]
pub struct State {
    pub name: String,
    pub is_start: bool,
    pub is_finish: bool,
}
impl Eq for State {}

impl State {
    pub fn new(name: String, start: bool, finish: bool) -> State {
        State {
            name: name,
            is_start: start,
            is_finish: finish,
        }
    }

    pub fn is_error(&self) -> bool {
        self.name == "-"
    }
}

#[derive(Debug)]
pub struct DFA {
    start: State,
    states: HashSet<State>,
    jump: HashMap<(State, String), State>,
}

impl DFA {
    pub fn new(start: State, jump: HashMap<(State, String), State>) -> DFA {
        let mut dfa = DFA {
            start: start,
            states: HashSet::new(),
            jump: jump,
        };
        dfa.compute_states();
        dfa
    }

    pub fn parse(input_path: &str) -> io::Result<DFA> {
        let file = BufReader::new(File::open(input_path)?);
        DFA::parse_from_reader(file)
    }

    pub fn parse_from_reader<R: ?Sized + BufRead>(r: R) -> io::Result<DFA>
    where
        R: ::std::marker::Sized,
    {
        let mut start: Option<State> = None;
        let mut row_header: State;
        let mut start_marker;
        let mut finish_marker;
        let mut alpha = Vec::new();
        let mut jump = HashMap::new();

        for line in r.lines() {
            let mut text = line?;
            let row = text.trim();
            if row.is_empty() || row.starts_with('#') {
                continue;
            }
            if alpha.is_empty() {
                alpha = text.split('|')
                    .skip(1)
                    .map(|x| x.trim().to_string())
                    .collect();
                continue;
            }
            for (i, s) in text.split('|').map(|x| x.trim().to_string()).enumerate() {
                if i == 0 {
                    start_marker = s.starts_with('>');
                    finish_marker = s.starts_with(if start_marker { ">*" } else { "*" });
                    row_header = State::new(s.to_string(), start_marker, finish_marker);
                    if start_marker {
                        if start.is_some() {
                            return Err(io::Error::new(
                                io::ErrorKind::Other,
                                "Duplicated start State",
                            ));
                        } else {
                            start = Some(row_header.clone());
                        }
                    }
                    continue;
                }
            }
        }
        if let Some(s) = start {
            Ok(DFA::new(s, jump))
        } else {
            Err(io::Error::new(
                io::ErrorKind::Other,
                "Don't see start state",
            ))
        }
    }

    pub fn compute_states(&mut self) {
        println!("Compute state");
    }

    pub fn check_string(&self, string: String) {
        println!("Check {}", string);
    }
    pub fn check(&self, input: &str) -> io::Result<()> {
        let file = BufReader::new(File::open(input)?);
        for line in file.lines() {
            self.check_string(line?);
        }
        Ok(())
    }
}
