use serde_yaml;

use std::error::Error;
use std::fmt;
use std::fs::File;
use std::io::{self, BufRead, BufReader};

#[derive(Debug, PartialEq, Hash, Clone, Copy)]
pub enum PDAState {
    State(u32),
    Stuck,
}

impl PDAState {
    pub fn new(id: u32) -> PDAState {
        PDAState::State(id)
    }
}

impl Eq for PDAState {}

impl fmt::Display for PDAState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                &PDAState::State(num) => format!("{}", num),
                &PDAState::Stuck => format!("STUCK"),
            }
        )
    }
}

impl<'de> ::serde::Deserialize<'de> for PDAState {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        struct Visitor;

        impl<'de> ::serde::de::Visitor<'de> for Visitor {
            type Value = PDAState;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("positive integer")
            }

            fn visit_u64<E>(self, value: u64) -> Result<PDAState, E>
            where
                E: ::serde::de::Error,
            {
                if value <= ::std::u32::MAX as u64 {
                    Ok(PDAState::State(value as u32))
                } else {
                    Err(E::custom(format!("too big number {}", value)))
                }
            }
        }

        // Deserialize the PDAState from a u64.
        deserializer.deserialize_u64(Visitor)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct PDAConfiguration {
    pub state: PDAState,
    pub stack: Vec<char>,
}
impl fmt::Display for PDAConfiguration {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "state: {}, stack: {:?}", self.state, self.stack,)
    }
}

impl PDAConfiguration {
    pub fn new(state: u32, stack: Vec<char>) -> PDAConfiguration {
        PDAConfiguration {
            state: PDAState::State(state),
            stack: stack,
        }
    }
    pub fn stuck(&self) -> PDAConfiguration {
        PDAConfiguration {
            state: PDAState::Stuck,
            stack: self.stack.clone(),
        }
    }

    pub fn is_stuck(&self) -> bool {
        self.state == PDAState::Stuck
    }
}

#[derive(Debug, PartialEq, Deserialize, Clone)]
pub struct PDARule {
    pub state: PDAState,
    pub character: Option<char>,
    pub next_state: PDAState,
    pub pop_character: Option<char>,
    pub push_characters: Vec<char>,
}
impl fmt::Display for PDARule {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "[{}]{};{}/{}",
            self.state,
            if let Some(ch) = self.character {
                format!("{}", ch)
            } else {
                String::new()
            },
            if let Some(ch) = self.pop_character {
                format!("{}", ch)
            } else {
                String::new()
            },
            self.push_characters.iter().collect::<String>()
        )
    }
}

impl PDARule {
    #[allow(unused)]
    pub fn new(
        state: u32,
        character: Option<char>,
        next_state: u32,
        pop_character: Option<char>,
        push_characters: Vec<char>,
    ) -> PDARule {
        PDARule {
            state: PDAState::new(state),
            character: character,
            next_state: PDAState::new(next_state),
            pop_character: pop_character,
            push_characters: push_characters,
        }
    }
    pub fn applies_to(&self, cfg: &PDAConfiguration, character: Option<char>) -> bool {
        self.state == cfg.state
            && self.pop_character == cfg.stack.last().cloned()
            && self.character == character
    }

    pub fn follow(&self, cfg: &PDAConfiguration) -> PDAConfiguration {
        PDAConfiguration {
            state: self.next_state.clone(),
            stack: self.next_stack(cfg),
        }
    }
    pub fn next_stack(&self, cfg: &PDAConfiguration) -> Vec<char> {
        cfg.stack
            .iter()
            .rev()
            .skip(1)
            .rev()
            .cloned()
            .chain(self.push_characters.iter().rev().cloned())
            .collect()
    }
}

#[derive(Debug, PartialEq, Deserialize, Clone)]
pub struct DPDARulebook {
    rules: Vec<PDARule>,
}

impl DPDARulebook {
    #[allow(unused)]
    pub fn new(rules: Vec<PDARule>) -> DPDARulebook {
        DPDARulebook { rules: rules }
    }

    pub fn next_configuration(
        &self,
        cfg: &PDAConfiguration,
        character: Option<char>,
    ) -> Option<PDAConfiguration> {
        if let Some(rule) = self.rule_for(cfg, character) {
            Some(rule.follow(cfg))
        } else {
            None
        }
    }
    pub fn rule_for(&self, cfg: &PDAConfiguration, character: Option<char>) -> Option<&PDARule> {
        self.rules
            .iter()
            .find(|ref rule| rule.applies_to(cfg, character))
    }

    pub fn applies_to(&self, cfg: &PDAConfiguration, character: Option<char>) -> bool {
        self.rule_for(cfg, character).is_some()
    }

    pub fn follow_free_moves(&self, cfg: PDAConfiguration) -> PDAConfiguration {
        if self.applies_to(&cfg, None) {
            self.follow_free_moves(self.next_configuration(&cfg, None).unwrap())
        } else {
            cfg
        }
    }
}

pub struct DPDA {
    pub _current_cfg: PDAConfiguration,
    pub accept_states: Vec<PDAState>,
    pub rulebook: DPDARulebook,
    pub accept_by_empty_stack: bool,
    pub traversed_path: Option<Vec<Option<PDARule>>>,
}

impl DPDA {
    pub fn new(
        cfg: PDAConfiguration,
        accept_states: Vec<u32>,
        rulebook: DPDARulebook,
        accept_by_empty_stack: bool,
        traversed_path: Option<Vec<Option<PDARule>>>,
    ) -> Self {
        DPDA {
            _current_cfg: cfg,
            accept_states: accept_states.iter().map(|x| PDAState::new(*x)).collect(),
            rulebook: rulebook,
            accept_by_empty_stack: accept_by_empty_stack,
            traversed_path: traversed_path,
        }
    }

    pub fn accepting(&self) -> bool {
        let cfg = self.current_cfg();
        let accept = self.accept_states.contains(&cfg.state);
        if self.accept_by_empty_stack {
            accept && cfg.stack.is_empty()
        } else {
            accept
        }
    }

    pub fn is_stuck(&self) -> bool {
        self._current_cfg.is_stuck()
    }

    pub fn next_configuration(&mut self, character: char) -> PDAConfiguration {
        let mut current_cfg = self._current_cfg.clone();
        let may_be_cfg = self
            .rulebook
            .next_configuration(&current_cfg, Some(character));
        if may_be_cfg.is_none() {
            current_cfg = self.rulebook.follow_free_moves(current_cfg)
        }

        if let Some(rule) = self.next_rule(&current_cfg, character) {
            println!("\n{}", rule);
            println!("  Configuration:\n    current: {}", current_cfg);
        }
        let next_cfg;
        if let Some(cfg) = self
            .rulebook
            .next_configuration(&current_cfg, Some(character))
        {
            next_cfg = cfg;
        } else {
            next_cfg = self._current_cfg.stuck();
        }
        println!("    next: {}", next_cfg);
        next_cfg
    }

    pub fn next_rule(&self, cfg: &PDAConfiguration, character: char) -> Option<PDARule> {
        self.rulebook.rule_for(&cfg, Some(character)).cloned()
    }

    pub fn current_cfg(&self) -> PDAConfiguration {
        self.rulebook.follow_free_moves(self._current_cfg.clone())
    }

    pub fn read_character(&mut self, character: char) {
        self._current_cfg = self.next_configuration(character)
    }

    pub fn read_string(&mut self, string: &str) -> String {
        let mut eaten = String::new();
        for character in string.chars() {
            if self.is_stuck() {
                break;
            }
            eaten.push(character);
            self.read_character(character);
        }
        eaten
    }
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct DPDADesign {
    pub start_state: u32,
    pub bottom_character: char,
    pub accept_states: Vec<u32>,
    pub rulebook: DPDARulebook,
    pub accept_by_empty_stack: bool,
}

pub struct DPDADesignResult {
    pub ok: bool,
    pub cfg: PDAConfiguration,
    pub path: Option<Vec<Option<PDARule>>>,
    pub eaten_part: String,
}

impl DPDADesign {
    #[allow(unused)]
    pub fn new(start: u32, bottom: char, accept: Vec<u32>, rulebook: DPDARulebook) -> DPDADesign {
        DPDADesign {
            start_state: start,
            bottom_character: bottom,
            accept_states: accept,
            rulebook: rulebook,
            accept_by_empty_stack: false,
        }
    }

    pub fn load(input_path: &str) -> io::Result<DPDADesign> {
        let file = BufReader::new(File::open(input_path)?);
        DPDADesign::load_from_reader(file)
    }

    pub fn load_from_reader<R: ?Sized + BufRead>(r: R) -> io::Result<DPDADesign>
    where
        R: ::std::marker::Sized,
    {
        match serde_yaml::from_reader(r) {
            Ok(design) => Ok(design),
            Err(err) => Err(io::Error::new(io::ErrorKind::Other, err.description())),
        }
    }

    pub fn accepts(&self, string: &str) -> DPDADesignResult {
        let mut dpda = self.to_dpda();
        let eaten_part = dpda.read_string(string);
        DPDADesignResult {
            ok: dpda.accepting(),
            cfg: dpda.current_cfg(),
            path: dpda.traversed_path,
            eaten_part: eaten_part,
        }
    }

    pub fn to_dpda(&self) -> DPDA {
        let start_stack = vec![self.bottom_character];
        let start_cfg = PDAConfiguration::new(self.start_state, start_stack);
        DPDA::new(
            start_cfg,
            self.accept_states.iter().cloned().collect(),
            self.rulebook.clone(),
            self.accept_by_empty_stack,
            Some(Vec::new()),
        )
    }
}

#[cfg(test)]
mod tests {
    use self::super::*;

    fn get_rulebook() -> DPDARulebook {
        DPDARulebook::new(vec![
            PDARule::new(1, Some('('), 2, Some('$'), vec!['b', '$']),
            PDARule::new(2, Some('('), 2, Some('b'), vec!['b', 'b']),
            PDARule::new(2, Some(')'), 2, Some('b'), vec![]),
            PDARule::new(2, None, 1, Some('$'), vec!['$']),
        ])
    }

    #[test]
    fn applies_to() {
        let rule = PDARule::new(1, Some('('), 2, Some('$'), vec!['b', '$']);
        let cfg = PDAConfiguration::new(1, vec!['$']);
        assert_eq!(rule.applies_to(&cfg, Some('(')), true);
    }

    #[test]
    fn rule_follow() {
        let rule = PDARule::new(1, Some('('), 2, Some('$'), vec!['b', '$']);
        let cfg = PDAConfiguration::new(1, vec!['$']);
        let new_cfg = rule.follow(&cfg);
        assert!(new_cfg.state == PDAState::new(2) && new_cfg.stack == vec!['$', 'b']);
    }

    #[test]
    fn next_stack() {
        let rule = PDARule::new(1, Some('('), 2, Some('T'), vec!['a', 'b', 'T']);
        let cfg = PDAConfiguration::new(1, vec!['$', 'T']);

        let stack = rule.next_stack(&cfg);
        assert_eq!(stack, vec!['$', 'T', 'b', 'a']);
        println!("{:?}", stack);
        assert_eq!(stack.last(), Some(&'a'));
    }

    #[test]
    fn rulebook() {
        let rulebook = get_rulebook();
        let mut cfg = Some(PDAConfiguration::new(1, vec!['$']));
        cfg = rulebook.next_configuration(&cfg.unwrap(), Some('('));
        assert_eq!(cfg, Some(PDAConfiguration::new(2, vec!['$', 'b'])));
        cfg = rulebook.next_configuration(&cfg.unwrap(), Some('('));
        assert_eq!(cfg, Some(PDAConfiguration::new(2, vec!['$', 'b', 'b'])));
        cfg = rulebook.next_configuration(&cfg.unwrap(), Some(')'));
        assert_eq!(cfg, Some(PDAConfiguration::new(2, vec!['$', 'b'])));
    }

    #[test]
    fn dpda() {
        let cfg = PDAConfiguration::new(1, vec!['$']);
        let accept_states: Vec<u32> = vec![1];
        let rulebook = get_rulebook();

        let mut dpda = DPDA::new(cfg, accept_states, rulebook, false, None);

        assert!(dpda.accepting(), "Initial state not accepting!");
        dpda.read_string("(()");
        assert_eq!(dpda.accepting(), false, "Accept invalid string!");

        assert_eq!(
            dpda.current_cfg(),
            PDAConfiguration::new(2, vec!['$', 'b']),
            "Unexpected state"
        );

        dpda.read_string(")");
        assert_eq!(dpda.accepting(), true, "Accept expected!");

        dpda.read_string("(()(");
        assert_eq!(dpda.accepting(), false, "Accept invalid string!");
        assert_eq!(
            dpda.current_cfg(),
            PDAConfiguration::new(2, vec!['$', 'b', 'b'])
        );
        dpda.read_string("))()");
        assert_eq!(dpda.current_cfg(), PDAConfiguration::new(1, vec!['$']));
        assert_eq!(dpda.accepting(), true, "Accept expected!");
    }

    #[test]
    fn follow_free_moves() {
        let cfg = PDAConfiguration::new(2, vec!['$']);
        let rulebook = get_rulebook();

        assert_eq!(rulebook.follow_free_moves(cfg).state, PDAState::new(1))
    }

    #[test]
    fn design() {
        let rulebook = get_rulebook();
        let dpda_design = DPDADesign::new(1, '$', vec![1], rulebook);
        assert!(dpda_design.accepts("(((((((((())))))))))").ok);
        assert!(dpda_design.accepts("()(())((()))(()(()))").ok);
        assert!(!dpda_design.accepts("(()(()(()()(()()))()").ok);
        assert!(!dpda_design.accepts("())").ok);
    }

    #[test]
    fn load_design() {
        let rulebook = get_rulebook();
        let dpda_design = DPDADesign::new(1, '$', vec![1], rulebook);
        let dpda_design_from_sample_file = DPDADesign::load("sample/pda/brackets.yaml").unwrap();
        assert_eq!(dpda_design, dpda_design_from_sample_file);
    }
}
