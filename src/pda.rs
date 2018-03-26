use serde_yaml;

use std::fmt;
use std::fs::File;
use std::error::Error;
use std::io::{self, BufRead, BufReader};

#[derive(Debug, PartialEq, Deserialize, Clone)]
pub struct PDAConfiguration {
    #[serde(skip_deserializing)]
    stuck: bool,
    pub state: u32,
    pub stack: Vec<char>,
}

impl PDAConfiguration {
    pub fn new(state: u32, stack: Vec<char>) -> PDAConfiguration {
        PDAConfiguration {
            stuck: false,
            state: state,
            stack: stack,
        }
    }
    pub fn stuck(&self) -> PDAConfiguration {
        PDAConfiguration {
            stuck: true,
            state: self.state,
            stack: self.stack.clone(),
        }
    }

    pub fn is_stuck(&self) -> bool {
        self.stuck
    }
}

#[derive(Debug, PartialEq, Deserialize, Clone)]
pub struct PDARule {
    pub state: u32,
    pub character: Option<char>,
    pub next_state: u32,
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
                "".to_string()
            },
            if let Some(ch) = self.pop_character {
                format!("{}", ch)
            } else {
                "".to_string()
            },
            self.push_characters.iter().collect::<String>()
        )
    }
}

impl PDARule {
    pub fn new(
        state: u32,
        character: Option<char>,
        next_state: u32,
        pop_character: Option<char>,
        push_characters: Vec<char>,
    ) -> PDARule {
        PDARule {
            state: state,
            character: character,
            next_state: next_state,
            pop_character: pop_character,
            push_characters: push_characters,
        }
    }
    pub fn applies_to(&self, cfg: &PDAConfiguration, character: Option<char>) -> bool {
        self.state == cfg.state && self.pop_character == cfg.stack.last().cloned()
            && self.character == character
    }

    pub fn follow(&self, cfg: &PDAConfiguration) -> PDAConfiguration {
        PDAConfiguration {
            stuck: false,
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
    pub accept_states: Vec<u32>,
    pub rulebook: DPDARulebook,
    pub traversed_path: Option<Vec<Option<PDARule>>>,
}

impl DPDA {
    pub fn new(
        cfg: PDAConfiguration,
        accept_states: Vec<u32>,
        rulebook: DPDARulebook,
        traversed_path: Option<Vec<Option<PDARule>>>,
    ) -> DPDA {
        DPDA {
            _current_cfg: cfg,
            accept_states: accept_states,
            rulebook: rulebook,
            traversed_path: traversed_path,
        }
    }

    pub fn accepting(&self) -> bool {
        self.accept_states.contains(&self.current_cfg().state)
    }

    pub fn is_stuck(&self) -> bool {
        self.current_cfg().is_stuck()
    }

    pub fn read_character(&mut self, character: char) {
        let save_path = self.traversed_path.is_some();
        if save_path {
            let rule = self.rulebook
                .rule_for(&self.current_cfg(), Some(character))
                .cloned();
            self.traversed_path.as_mut().unwrap().push(rule);
        }
        if let Some(cfg) = self.rulebook
            .next_configuration(&self.current_cfg(), Some(character))
        {
            self._current_cfg = cfg
        } else {
            self._current_cfg = self.current_cfg().stuck()
        }
    }

    pub fn read_string(&mut self, string: String) {
        for character in string.chars() {
            if self.is_stuck() {
                break;
            }
            self.read_character(character);
        }
    }

    pub fn current_cfg(&self) -> PDAConfiguration {
        self.rulebook.follow_free_moves(self._current_cfg.clone())
    }
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct DPDADesign {
    pub start_state: u32,
    pub bottom_character: char,
    pub accept_states: Vec<u32>,
    pub rulebook: DPDARulebook,
    #[serde(skip_deserializing)]
    pub remember_traversed_path: bool,
}

pub struct DPDADesignResult {
    pub cfg: PDAConfiguration,
    pub path: Option<Vec<Option<PDARule>>>,
    pub ok: bool,
}

impl DPDADesign {
    pub fn new(start: u32, bottom: char, accept: Vec<u32>, rulebook: DPDARulebook) -> DPDADesign {
        DPDADesign {
            start_state: start,
            bottom_character: bottom,
            accept_states: accept,
            rulebook: rulebook,
            remember_traversed_path: false,
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

    pub fn accepts(&self, string: String) -> DPDADesignResult {
        let mut dpda = self.to_dpda();
        dpda.read_string(string);
        DPDADesignResult {
            ok: dpda.accepting(),
            cfg: dpda.current_cfg(),
            path: dpda.traversed_path,
        }
    }

    pub fn to_dpda(&self) -> DPDA {
        let start_stack = vec![self.bottom_character];
        let start_cfg = PDAConfiguration::new(self.start_state, start_stack);
        let mut path = None;
        if self.remember_traversed_path {
            path = Some(Vec::new());
        }
        DPDA::new(
            start_cfg,
            self.accept_states.iter().cloned().collect(),
            self.rulebook.clone(),
            path,
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
        assert!(new_cfg.state == 2 && new_cfg.stack == vec!['$', 'b']);
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

        let mut dpda = DPDA::new(cfg, accept_states, rulebook, None);

        assert!(dpda.accepting(), "Initial state not accepting!");
        dpda.read_string("(()".to_string());
        assert_eq!(dpda.accepting(), false, "Accept invalid string!");

        assert_eq!(
            dpda.current_cfg(),
            PDAConfiguration::new(2, vec!['$', 'b']),
            "Unexpected state"
        );

        dpda.read_string(")".to_string());
        assert_eq!(dpda.accepting(), true, "Accept expected!");

        dpda.read_string("(()(".to_string());
        assert_eq!(dpda.accepting(), false, "Accept invalid string!");
        assert_eq!(
            dpda.current_cfg(),
            PDAConfiguration::new(2, vec!['$', 'b', 'b'])
        );
        dpda.read_string("))()".to_string());
        assert_eq!(dpda.current_cfg(), PDAConfiguration::new(1, vec!['$']));
        assert_eq!(dpda.accepting(), true, "Accept expected!");
    }

    #[test]
    fn follow_free_moves() {
        let cfg = PDAConfiguration::new(2, vec!['$']);
        let rulebook = get_rulebook();

        assert_eq!(rulebook.follow_free_moves(cfg).state, 1)
    }

    #[test]
    fn design() {
        let rulebook = get_rulebook();
        let dpda_design = DPDADesign::new(1, '$', vec![1], rulebook);
        assert!(dpda_design.accepts("(((((((((())))))))))".to_string()).ok);
        assert!(dpda_design.accepts("()(())((()))(()(()))".to_string()).ok);
        assert!(!dpda_design.accepts("(()(()(()()(()()))()".to_string()).ok);
        assert!(!dpda_design.accepts("())".to_string()).ok);
    }

    #[test]
    fn load_design() {
        let rulebook = get_rulebook();
        let dpda_design = DPDADesign::new(1, '$', vec![1], rulebook);
        let dpda_design_from_sample_file = DPDADesign::load("sample/pda-config.yaml").unwrap();
        assert_eq!(dpda_design, dpda_design_from_sample_file);
    }
}
