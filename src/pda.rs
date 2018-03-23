type PDAState = u32;

#[derive(PartialEq, Debug)]
pub struct PDAConfiguration {
    pub state: PDAState,
    pub stack: Vec<char>,
}
impl PDAConfiguration {
    pub fn new(state: PDAState, stack: Vec<char>) -> PDAConfiguration {
        PDAConfiguration {
            state: state,
            stack: stack,
        }
    }
}

pub struct PDARule {
    pub state: PDAState,
    pub character: Option<char>,
    pub next_state: PDAState,
    pub pop_character: Option<char>,
    pub push_characters: Vec<char>,
}

impl PDARule {
    pub fn new(
        state: PDAState,
        character: Option<char>,
        next_state: PDAState,
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

    pub fn follow(&self, cfg: PDAConfiguration) -> PDAConfiguration {
        PDAConfiguration {
            state: self.next_state.clone(),
            stack: self.next_stack(cfg),
        }
    }
    pub fn next_stack(&self, cfg: PDAConfiguration) -> Vec<char> {
        cfg.stack
            .iter()
            .rev()
            .cloned()
            .skip(1)
            .rev()
            .chain(self.push_characters.iter().rev().cloned())
            .collect()
    }
}

pub struct DPDARulebook {
    rules: Vec<PDARule>,
}

impl DPDARulebook {
    pub fn new(rules: Vec<PDARule>) -> DPDARulebook {
        DPDARulebook { rules: rules }
    }

    pub fn next_configuration(
        &self,
        cfg: PDAConfiguration,
        character: Option<char>,
    ) -> PDAConfiguration {
        self.rule_for(&cfg, character).follow(cfg)
    }
    pub fn rule_for(&self, cfg: &PDAConfiguration, character: Option<char>) -> &PDARule {
        self.rules
            .iter()
            .find(|ref rule| rule.applies_to(cfg, character))
            .unwrap()
    }
}

#[cfg(test)]
mod tests {
    use self::super::*;

    #[test]
    fn test_applies_to() {
        let rule = PDARule::new(1, Some('('), 2, Some('$'), vec!['b', '$']);
        let cfg = PDAConfiguration::new(1, vec!['$']);
        assert_eq!(rule.applies_to(&cfg, Some('(')), true);
    }

    #[test]
    fn test_rule_follow() {
        let rule = PDARule::new(1, Some('('), 2, Some('$'), vec!['b', '$']);
        let cfg = PDAConfiguration::new(1, vec!['$']);
        let new_cfg = rule.follow(cfg);
        assert!(new_cfg.state == 2 && new_cfg.stack == vec!['$', 'b']);
    }

    #[test]
    fn test_next_stack() {
        let rule = PDARule::new(1, Some('('), 2, Some('T'), vec!['a', 'b', 'T']);
        let cfg = PDAConfiguration::new(1, vec!['$', 'T']);

        let stack = rule.next_stack(cfg);
        assert_eq!(stack, vec!['$', 'T', 'b', 'a']);
        println!("{:?}", stack);
        assert_eq!(stack.last(), Some(&'a'));
    }

    #[test]
    fn test_rulebook() {
        let mut cfg = PDAConfiguration::new(1, vec!['$']);
        let rulebook = DPDARulebook::new(vec![
            PDARule::new(1, Some('('), 2, Some('$'), vec!['b', '$']),
            PDARule::new(2, Some('('), 2, Some('b'), vec!['b', 'b']),
            PDARule::new(2, Some(')'), 2, Some('b'), vec![]),
            PDARule::new(2, None, 1, Some('$'), vec!['$']),
        ]);
        cfg = rulebook.next_configuration(cfg, Some('('));
        assert_eq!(cfg, PDAConfiguration::new(2, vec!['$', 'b']));
        cfg = rulebook.next_configuration(cfg, Some('('));
        assert_eq!(cfg, PDAConfiguration::new(2, vec!['$', 'b', 'b']));
        cfg = rulebook.next_configuration(cfg, Some(')'));
        assert_eq!(cfg, PDAConfiguration::new(2, vec!['$', 'b']));
    }
}
