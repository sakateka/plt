use serde_yaml;

use std::fmt;
use std::fs::File;
use std::io::{self, BufRead, BufReader};

#[derive(Debug, PartialEq, Hash, Clone, Copy)]
pub enum PDTState {
    State(u32),
    Stuck,
}

impl PDTState {
    pub fn new(id: u32) -> PDTState {
        PDTState::State(id)
    }
}

impl Eq for PDTState {}

impl fmt::Display for PDTState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                PDTState::State(num) => format!("{}", num),
                PDTState::Stuck => "STUCK".to_owned(),
            }
        )
    }
}

impl<'de> ::serde::Deserialize<'de> for PDTState {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        struct Visitor;

        impl<'de> ::serde::de::Visitor<'de> for Visitor {
            type Value = PDTState;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("positive integer")
            }

            fn visit_u64<E>(self, value: u64) -> Result<PDTState, E>
            where
                E: ::serde::de::Error,
            {
                if value <= ::std::u32::MAX as u64 {
                    Ok(PDTState::State(value as u32))
                } else {
                    Err(E::custom(format!("too bit number {}", value)))
                }
            }
        }

        // Deserialize the PDTState from a u64.
        deserializer.deserialize_u64(Visitor)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct PDTConfiguration {
    pub state: PDTState,
    pub stack: Vec<char>,
    pub translated: Vec<String>,
}
impl fmt::Display for PDTConfiguration {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "state: {}, stack: {:?}, translated: {}",
            self.state,
            self.stack,
            self.translated.iter().fold(String::new(), |mut acc, x| {
                acc.push_str(x);
                acc
            }),
        )
    }
}

impl PDTConfiguration {
    pub fn new(state: u32, stack: Vec<char>, translated: Vec<&str>) -> PDTConfiguration {
        PDTConfiguration {
            state: PDTState::State(state),
            stack,
            translated: translated.iter().map(|x| x.to_string()).collect(),
        }
    }
    pub fn stuck(&self) -> PDTConfiguration {
        PDTConfiguration {
            state: PDTState::Stuck,
            stack: self.stack.clone(),
            translated: self.translated.clone(),
        }
    }

    pub fn is_stuck(&self) -> bool {
        self.state == PDTState::Stuck
    }
}

#[derive(Debug, PartialEq, Deserialize, Clone)]
pub struct PDTRule {
    pub state: PDTState,
    pub character: Option<char>,
    pub translated: Option<String>,
    pub next_state: PDTState,
    pub pop_character: Option<char>,
    pub push_characters: Vec<char>,
}

impl fmt::Display for PDTRule {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Rule (state {}):\n{}\n{}\n{}\n{}",
            self.state,
            format!("  character: {:?}", self.character),
            format!("  translated: {:?}", self.translated),
            format!("  pop: {:?}", self.pop_character),
            format!("  push: {:?}", self.push_characters),
        )
    }
}

impl PDTRule {
    #[allow(unused)]
    pub fn new(
        state: u32,
        character: Option<char>,
        translated: Option<&str>,
        next_state: u32,
        pop_character: Option<char>,
        push_characters: Vec<char>,
    ) -> PDTRule {
        let trans = translated.map(|x| x.to_string());
        PDTRule {
            state: PDTState::new(state),
            character,
            translated: trans,
            next_state: PDTState::new(next_state),
            pop_character,
            push_characters,
        }
    }
    pub fn applies_to(&self, cfg: &PDTConfiguration, character: Option<char>) -> bool {
        self.state == cfg.state && self.pop_character == cfg.stack.last().cloned()
            && self.character == character
    }

    pub fn follow(&self, cfg: &PDTConfiguration) -> PDTConfiguration {
        let mut translated = cfg.translated.clone();
        if let Some(ch) = self.translated.clone() {
            translated.push(ch);
        }
        PDTConfiguration {
            state: self.next_state,
            stack: self.next_stack(cfg),
            translated,
        }
    }
    pub fn next_stack(&self, cfg: &PDTConfiguration) -> Vec<char> {
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
pub struct DPDTRulebook {
    rules: Vec<PDTRule>,
}

impl DPDTRulebook {
    #[allow(unused)]
    pub fn new(rules: Vec<PDTRule>) -> DPDTRulebook {
        DPDTRulebook { rules }
    }

    pub fn next_configuration(
        &self,
        cfg: &PDTConfiguration,
        character: Option<char>,
    ) -> Option<PDTConfiguration> {
        self.rule_for(cfg, character).map(|rule| rule.follow(cfg))
    }
    pub fn rule_for(&self, cfg: &PDTConfiguration, character: Option<char>) -> Option<&PDTRule> {
        self.rules
            .iter()
            .find(|rule| rule.applies_to(cfg, character))
    }

    pub fn applies_to(&self, cfg: &PDTConfiguration, character: Option<char>) -> bool {
        self.rule_for(cfg, character).is_some()
    }

    pub fn follow_free_moves(&self, cfg: PDTConfiguration) -> PDTConfiguration {
        if self.applies_to(&cfg, None) {
            self.follow_free_moves(self.next_configuration(&cfg, None).unwrap())
        } else {
            cfg
        }
    }
}

pub struct DPDT {
    pub _current_cfg: PDTConfiguration,
    pub accept_states: Vec<PDTState>,
    pub rulebook: DPDTRulebook,
    pub accept_by_empty_stack: bool,
}

impl DPDT {
    pub fn new(
        cfg: PDTConfiguration,
        accept_states: Vec<u32>,
        rulebook: DPDTRulebook,
        accept_by_empty_stack: bool,
    ) -> Self {
        DPDT {
            _current_cfg: cfg,
            accept_states: accept_states.iter().map(|x| PDTState::new(*x)).collect(),
            rulebook,
            accept_by_empty_stack,
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

    pub fn next_configuration(&mut self, character: char) -> PDTConfiguration {
        let mut current_cfg = self._current_cfg.clone();
        let may_be_cfg = self.rulebook
            .next_configuration(&current_cfg, Some(character));
        if may_be_cfg.is_none() {
            current_cfg = self.rulebook.follow_free_moves(current_cfg)
        }

        if let Some(rule) = self.next_rule(&current_cfg, character) {
            println!("\n{}", rule);
            println!("  Configuration:\n    current: {}", current_cfg);
        }
        let next_cfg;
        if let Some(cfg) = self.rulebook
            .next_configuration(&current_cfg, Some(character))
        {
            next_cfg = cfg;
        } else {
            next_cfg = self._current_cfg.stuck();
        }
        println!("    next: {}", next_cfg);
        next_cfg
    }

    pub fn next_rule(&self, cfg: &PDTConfiguration, character: char) -> Option<PDTRule> {
        self.rulebook.rule_for(cfg, Some(character)).cloned()
    }

    pub fn current_cfg(&self) -> PDTConfiguration {
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
pub struct DPDTDesign {
    pub start_state: u32,
    pub bottom_character: char,
    pub accept_states: Vec<u32>,
    pub rulebook: DPDTRulebook,
    pub accept_by_empty_stack: bool,
}

pub struct DPDTDesignResult {
    pub ok: bool,
    pub cfg: PDTConfiguration,
    pub eaten_part: String,
}

impl DPDTDesign {
    #[allow(unused)]
    pub fn new(start: u32, bottom: char, accept: Vec<u32>, rulebook: DPDTRulebook) -> DPDTDesign {
        DPDTDesign {
            start_state: start,
            bottom_character: bottom,
            accept_states: accept,
            rulebook,
            accept_by_empty_stack: false,
        }
    }

    pub fn load(input_path: &str) -> io::Result<DPDTDesign> {
        let file = BufReader::new(File::open(input_path)?);
        DPDTDesign::load_from_reader(file)
    }

    pub fn load_from_reader<R: ?Sized + BufRead>(r: R) -> io::Result<DPDTDesign>
    where
        R: ::std::marker::Sized,
    {
        match serde_yaml::from_reader(r) {
            Ok(design) => Ok(design),
            Err(err) => Err(io::Error::new(io::ErrorKind::Other, err.to_string())),
        }
    }

    pub fn accepts(&self, string: &str) -> DPDTDesignResult {
        let mut dpdt = self.to_dpdt();
        let eaten_part = dpdt.read_string(string);
        DPDTDesignResult {
            ok: dpdt.accepting(),
            cfg: dpdt.current_cfg(),
            eaten_part,
        }
    }

    pub fn to_dpdt(&self) -> DPDT {
        let start_stack = vec![self.bottom_character];
        let start_cfg = PDTConfiguration::new(self.start_state, start_stack, Vec::new());
        DPDT::new(
            start_cfg,
            self.accept_states.to_vec(),
            self.rulebook.clone(),
            self.accept_by_empty_stack,
        )
    }
}

#[cfg(test)]
mod tests {
    use self::super::*;

    fn get_rulebook() -> DPDTRulebook {
        DPDTRulebook::new(vec![
            PDTRule::new(1, Some('('), Some("["), 2, Some('$'), vec!['b', '$']),
            PDTRule::new(2, Some('('), Some("["), 2, Some('b'), vec!['b', 'b']),
            PDTRule::new(2, Some(')'), Some("]"), 2, Some('b'), vec![]),
            PDTRule::new(2, None, None, 1, Some('$'), vec!['$']),
        ])
    }

    #[test]
    fn applies_to() {
        let rule = PDTRule::new(1, Some('('), Some("["), 2, Some('$'), vec!['b', '$']);
        let cfg = PDTConfiguration::new(1, vec!['$'], Vec::new());
        assert!(rule.applies_to(&cfg, Some('(')));
    }

    #[test]
    fn rule_follow() {
        let rule = PDTRule::new(1, Some('('), Some("["), 2, Some('$'), vec!['b', '$']);
        let cfg = PDTConfiguration::new(1, vec!['$'], Vec::new());
        let new_cfg = rule.follow(&cfg);
        assert!(
            new_cfg.state == PDTState::new(2) && new_cfg.stack == vec!['$', 'b']
                && new_cfg.translated == vec!["["]
        );
    }

    #[test]
    fn next_stack() {
        let rule = PDTRule::new(1, Some('('), Some("["), 2, Some('T'), vec!['a', 'b', 'T']);
        let cfg = PDTConfiguration::new(1, vec!['$', 'T'], Vec::new());

        let stack = rule.next_stack(&cfg);
        assert_eq!(stack, vec!['$', 'T', 'b', 'a']);
        println!("{:?}", stack);
        assert_eq!(stack.last(), Some(&'a'));
    }

    #[test]
    fn rulebook() {
        let rulebook = get_rulebook();
        let mut cfg = Some(PDTConfiguration::new(1, vec!['$'], Vec::new()));
        cfg = rulebook.next_configuration(&cfg.unwrap(), Some('('));
        assert_eq!(
            cfg,
            Some(PDTConfiguration::new(2, vec!['$', 'b'], vec!["["]))
        );
        cfg = rulebook.next_configuration(&cfg.unwrap(), Some('('));
        assert_eq!(
            cfg,
            Some(PDTConfiguration::new(
                2,
                vec!['$', 'b', 'b'],
                vec!["[", "["]
            ))
        );
        cfg = rulebook.next_configuration(&cfg.unwrap(), Some(')'));
        assert_eq!(
            cfg,
            Some(PDTConfiguration::new(
                2,
                vec!['$', 'b'],
                vec!["[", "[", "]"]
            ))
        );
    }

    #[test]
    fn dpdt() {
        let cfg = PDTConfiguration::new(1, vec!['$'], Vec::new());
        let accept_states: Vec<u32> = vec![1];
        let rulebook = get_rulebook();

        let mut dpdt = DPDT::new(cfg, accept_states, rulebook, false);

        assert!(dpdt.accepting(), "Initial state not accepting!");
        dpdt.read_string("(()");
        assert!(!dpdt.accepting(), "Accept invalid string!");

        assert_eq!(
            dpdt.current_cfg(),
            PDTConfiguration::new(2, vec!['$', 'b'], vec!["[", "[", "]"]),
            "Unexpected state"
        );

        dpdt.read_string(")");
        assert!(dpdt.accepting(), "Accept expected!");

        dpdt.read_string("(()(");
        assert!(!dpdt.accepting(), "Accept invalid string!");
        assert_eq!(
            dpdt.current_cfg(),
            PDTConfiguration::new(
                2,
                vec!['$', 'b', 'b'],
                vec!["[", "[", "]", "]", "[", "[", "]", "["]
            )
        );
        dpdt.read_string("))()");
        assert_eq!(
            dpdt.current_cfg(),
            PDTConfiguration::new(
                1,
                vec!['$'],
                vec!["[", "[", "]", "]", "[", "[", "]", "[", "]", "]", "[", "]"]
            )
        );
        assert!(dpdt.accepting(), "Accept expected!");
    }

    #[test]
    fn follow_free_moves() {
        let cfg = PDTConfiguration::new(2, vec!['$'], Vec::new());
        let rulebook = get_rulebook();

        assert_eq!(rulebook.follow_free_moves(cfg).state, PDTState::new(1))
    }

    #[test]
    fn design() {
        let rulebook = get_rulebook();
        let dpdt_design = DPDTDesign::new(1, '$', vec![1], rulebook);
        assert!(dpdt_design.accepts("(((((((((())))))))))").ok);
        assert!(dpdt_design.accepts("()(())((()))(()(()))").ok);
        assert!(!dpdt_design.accepts("(()(()(()()(()()))()").ok);
        assert!(!dpdt_design.accepts("())").ok);
    }

    #[test]
    fn load_design() {
        let rulebook = get_rulebook();
        let dpdt_design = DPDTDesign::new(1, '$', vec![1], rulebook);
        let dpdt_design_from_sample_file = DPDTDesign::load("sample/pdt/brackets.yaml").unwrap();
        assert_eq!(dpdt_design, dpdt_design_from_sample_file);
    }
}
