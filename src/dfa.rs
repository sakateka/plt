use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Hash, PartialEq)]
pub struct State {
    pub name: String,
}
impl Eq for State {}

impl State {
    pub fn new(name: &str) -> State {
        State {
            name: name.to_owned(),
        }
    }
    pub fn is_error(&self) -> bool {
        self.name == "-"
    }
}

#[derive(Debug)]
pub struct DFA {
    start: State,
    finish: HashSet<State>,
    jump: HashMap<(State, char), State>,
}

impl DFA {
    pub fn new(start: State, finish: HashSet<State>, jump: HashMap<(State, char), State>) -> DFA {
        DFA {
            start: start,
            finish: finish,
            jump: jump,
        }
    }

    pub fn parse(input_path: &str, debug: bool) -> io::Result<DFA> {
        let file = BufReader::new(File::open(input_path)?);
        DFA::parse_from_reader(file, debug)
    }

    pub fn parse_from_reader<R: ?Sized + BufRead>(r: R, debug: bool) -> io::Result<DFA>
    where
        R: ::std::marker::Sized,
    {
        let mut start = None;
        let mut row_header;
        let mut alpha = Vec::new();
        let mut finish = HashSet::new();
        let mut jump = HashMap::new();

        for line in r.lines() {
            row_header = None;
            let mut text = line?;
            let row = text.trim();
            if row.is_empty() || row.starts_with('#') {
                if debug {
                    eprintln!("Skip: {}", text);
                }
                continue;
            }
            if alpha.is_empty() {
                if debug {
                    eprintln!("Parse alphabet: {}", text);
                }
                alpha = text.split('|')
                    .skip(1)
                    .map(|x| x.trim())
                    .filter(|x| {
                        if x.len() != 1 {
                            eprintln!("Skip alphabet element: {:?}", x);
                        }
                        x.len() == 1
                    })
                    .map(|x| x.chars().next().unwrap())
                    .collect();
                if debug {
                    eprintln!("Got alphabet: {:?}", alpha);
                }

                continue;
            }
            if debug {
                eprintln!("Parse state row: {}", text);
            }
            for (i, s) in text.split('|').map(|x| x.trim()).enumerate() {
                if i == 0 {
                    let mut is_start = false;
                    let mut is_finish = false;
                    let mut name = s;
                    if name.starts_with('>') {
                        if start.is_some() {
                            return Err(io::Error::new(
                                io::ErrorKind::Other,
                                "Duplicated start State",
                            ));
                        }
                        if name.len() < 2 {
                            return Err(io::Error::new(
                                io::ErrorKind::Other,
                                format!("Malformed State name: {}", s),
                            ));
                        } else {
                            is_start = true;
                            name = &s[1..];
                        }
                    }
                    if name.starts_with('*') {
                        if name.len() < 2 {
                            return Err(io::Error::new(
                                io::ErrorKind::Other,
                                format!("Malformed State name: {}", s),
                            ));
                        } else {
                            is_finish = true;
                            name = &name[1..];
                        }
                    }
                    row_header = Some(State::new(name));
                    if is_start {
                        start = row_header.clone();
                    }
                    if is_finish {
                        finish.insert(row_header.clone().unwrap());
                    }
                } else {
                    let state = State::new(s);
                    if i - 1 >= alpha.len() {
                        return Err(io::Error::new(
                            io::ErrorKind::Other,
                            format!("Malformed jump table, state {} has no input: {}", s, text),
                        ));
                    }
                    if let Some(ref cur_state) = row_header {
                        jump.insert((cur_state.clone(), alpha[i - 1].clone()), state);
                    } else {
                        return Err(io::Error::new(
                            io::ErrorKind::Other,
                            format!("Malformed jump table, no current state: {}", text),
                        ));
                    }
                }
            }
        }
        if let Some(s) = start {
            Ok(DFA::new(s, finish, jump))
        } else {
            Err(io::Error::new(
                io::ErrorKind::Other,
                "Don't see start state",
            ))
        }
    }

    pub fn check_string(&self, string: String) -> bool {
        let mut state = self.start.clone();
        let mut errorneous_sym = None;
        let mut index = 0;
        for (idx, sym) in string.chars().enumerate() {
            index = idx;
            let key = (state.clone(), sym);
            if let Some(x) = self.jump.get(&key) {
                state = x.clone();
                if state.is_error() {
                    break;
                }
            } else {
                errorneous_sym = Some(sym);
                break;
            }
        }
        let mut msg = "OK".to_string();
        if let Some(sym) = errorneous_sym {
            msg = format!(
                "Symbol '{}' at idx '{}' not in the alphabet of the DFA",
                sym,
                index
            );
        } else if state.is_error() {
            msg = format!(
                "DFA in the error state at idx {}, unaccepted part: {}",
                index,
                &string[index..]
            );
        } else if !self.finish.contains(&state) {
            msg = format!("EOL but DFA state '{:?}' is not accepting", state);
        }
        println!("{} - {}", string, msg);
        msg == "OK"
    }

    pub fn check(&self, input: &str) -> io::Result<()> {
        let file = BufReader::new(File::open(input)?);
        for line in file.lines() {
            self.check_string(line?);
        }
        Ok(())
    }
}
