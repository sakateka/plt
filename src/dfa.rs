use std::fmt;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read};
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone)]
pub struct State {
    pub name: String,
    pub is_start: bool,
    pub is_accept: bool,
    row: usize,
}
impl PartialEq for State {
    fn eq(&self, other: &State) -> bool {
        self.name == other.name
    }
}
impl Eq for State {}

impl Hash for State {
    fn hash<H: Hasher>(&self, hasher_state: &mut H) {
        self.name.hash(hasher_state);
    }
}

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "'{}' at row:{}", self.name, self.row)
    }
}

impl State {
    pub fn new(text: &str, col: usize, row: usize) -> io::Result<State> {
        let mut name = text.trim();
        if name.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to build state, col {} row {}", col, row),
            ));
        }
        let mut is_start = false;
        let mut is_accept = false;
        if name.starts_with('^') {
            if name.len() < 2 {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    format!("Malformed State name: {}, col {} row {}", name, col, row),
                ));
            } else {
                is_start = true;
                name = &name[1..];
            }
        }
        if name.starts_with('*') {
            if name.len() < 2 {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    format!("Malformed State name: {}, col {} row {}", name, col, row),
                ));
            } else {
                is_accept = true;
                name = &name[1..];
            }
        }
        Ok(State {
            name: name.to_owned(),
            is_start: is_start,
            is_accept: is_accept,
            row: row,
        })
    }
    pub fn is_error(&self) -> bool {
        self.name == "-"
    }
}

#[derive(Debug)]
pub struct DerivationPath<'a> (pub &'a Vec<&'a State>);

impl<'a> fmt::Display for DerivationPath<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for s in self.0 {
            if s.is_error() {
                write!(f, "->{}", "(ERROR)".to_string()).unwrap();
            } else {
                write!(f, "->{}", s.name).unwrap();
            }
        }
        Ok(())
    }
}


#[derive(Debug)]
pub struct DFA {
    start: State,
    finish: HashSet<State>,
    jump: HashMap<(State, char), State>,
}

impl DFA {
    pub fn new(jump: HashMap<(State, char), State>) -> io::Result<DFA> {
        if jump.is_empty() {
            return Err(io::Error::new(io::ErrorKind::Other, "Empty jump table"));
        }

        let mut headers: HashSet<State> = HashSet::new();
        let mut jumps: HashSet<State> = HashSet::new();

        let mut finish: HashSet<State> = HashSet::new();
        let start: HashSet<(State, usize)> = jump.iter()
            .filter(|x| {
                let s = &(x.0).0;
                headers.insert(s.clone());
                jumps.insert(x.1.clone());
                if s.is_accept {
                    finish.insert(s.clone());
                }
                s.is_start
            })
            .map(|x| {
                let s = &(x.0).0;
                (s.clone(), s.row)
            })
            .collect();

        if start.len() > 1 {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("A duplicate starting condition: {:?}", start),
            ));
        }

        let unreachable: HashSet<&State> =
            headers.difference(&jumps).filter(|x| !x.is_start).collect();
        if !unreachable.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Unreachable states: {:?}", unreachable),
            ));
        }
        let unknown: HashSet<&State> = jumps
            .difference(&headers)
            .filter(|x| !x.is_error())
            .collect();
        if !unknown.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Unknown states: {:?}", unknown),
            ));
        }

        Ok(DFA {
            start: start.iter().next().unwrap().to_owned().0,
            finish: finish,
            jump: jump,
        })
    }

    pub fn load(input_path: &str, debug: bool) -> io::Result<DFA> {
        let file = BufReader::new(File::open(input_path)?);
        DFA::load_from_reader(file, debug)
    }

    pub fn load_from_reader<R: ?Sized + BufRead>(r: R, debug: bool) -> io::Result<DFA>
    where
        R: ::std::marker::Sized,
    {
        let mut lines = r.lines()
            .map(|l| match l {
                Ok(t) => t.trim().to_owned(),
                Err(e) => panic!(e),
            })
            .filter(|l| {
                if l.is_empty() || l.starts_with("#") {
                    if debug {
                        eprintln!("Skip: {}", l);
                    }
                    false
                } else {
                    true
                }
            });

        let alpha: Vec<char>;
        if let Some(text) = lines.next() {
            if debug {
                eprintln!("Parse alphabet: {}", text);
            }

            alpha = text.split('|')
                .skip(1)
                .map(|x| x.trim())
                .filter(|x| {
                    if x.len() != 1 {
                        eprintln!("Skip alphabet element: {}", x);
                    }
                    x.len() == 1
                })
                .map(|x| x.chars().next().unwrap())
                .collect();
            if debug {
                eprintln!("Got alphabet: {:?}", alpha);
            }
        } else {
            return Err(io::Error::new(io::ErrorKind::Other, "There is no alphabet"));
        }
        if alpha.len() != alpha.iter().cloned().collect::<HashSet<char>>().len() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "A duplicate in the alphabet",
            ));
        }

        let mut jump = HashMap::new();
        let mut headers_set = HashSet::new();
        for (idx, text) in lines.enumerate() {
            if debug {
                eprintln!("Parse state row: {}", text);
            }

            let mut row = text.split('|');
            let header = State::new(row.next().unwrap(), 0, idx)?;
            if let Some(dup) = headers_set.take(&header) {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    format!(
                        "Malformed jump table, duplicate state row header 1:{:?}, 2:{:?}",
                        dup,
                        header,
                    ),
                ));
            } else {
                headers_set.insert(header.clone());
            }
            let mut states: Vec<State> = Vec::new();
            for (col, item) in row.enumerate() {
                let state = State::new(item, col, idx)?;
                states.push(state);
            }
            if states.len() != alpha.len() {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    format!(
                        "Malformed jump table, row {} '{}' states count not match alphabet lenght",
                        idx,
                        text
                    ),
                ));
            }
            jump.extend(
                alpha
                    .iter()
                    .cloned()
                    .map(|x| (header.clone(), x))
                    .zip(states.into_iter()),
            );
        }
        let dfa = DFA::new(jump);
        if debug {
            eprintln!("{:?}", dfa);
        }
        dfa
    }

    pub fn check_string(&self, string: String, show_path: bool) -> bool {
        let mut state = &self.start;
        let mut errorneous_sym = None;
        let mut index = 0;
        let mut path: Vec<&State> = Vec::new();

        path.push(state);
        for (idx, sym) in string.chars().enumerate() {
            index = idx;
            let key = (state.clone(), sym);
            if let Some(x) = self.jump.get(&key) {
                state = &x;
                if show_path {
                    path.push(state);
                }
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
        } else if !self.finish.contains(state) {
            msg = format!("EOL but DFA state {} is not accepting", state);
        }
        if show_path {
            println!("{}", DerivationPath(&path));
        }
        println!("{} - {}", string, msg);
        msg == "OK"
    }

    pub fn check(&self, input: Box<Read>, show_path: bool) -> io::Result<()> {
        let buf = BufReader::new(input);
        for line in buf.lines() {
            self.check_string(line?, show_path);
        }
        Ok(())
    }
}
