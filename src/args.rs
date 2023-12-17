use clap::{arg, value_parser, ArgAction, Command};

pub fn build_app() -> Command {
    Command::new("plt")
        .version("5.4.1")
        .author("Sergey Kacheev <uo0@ya.ru>")
        .about("Theory of Programming Languages and Translation Methods")
        .subcommand(
            Command::new("gen")
                .about("Sequence generator by CFG")
                .arg(arg!(<CFG> "Context-Free Grammar rules file to use"))
                .arg(arg!([OUT] "Output file (default to stdout)"))
                .arg(
                    arg!(--len_min <min> "Minimum sequence lenght (default 0)")
                        .value_parser(value_parser!(u32)),
                )
                .arg(
                    arg!(--len_max <max> "Maximum sequence lenght (default 8)")
                        .value_parser(value_parser!(u32)),
                )
                .arg(
                    arg!(-r --right "Use the right-hand derivation (default left-hand)")
                        .action(ArgAction::SetTrue),
                )
                .arg(
                    arg!(-a --all "Show all sequences together with duplicates")
                        .action(ArgAction::SetTrue),
                )
                .arg(arg!(-c --chomsky "Chomsky Normal Form").action(ArgAction::SetTrue)),
        )
        .subcommand(
            Command::new("simplify")
                .about("Simplify Context-Free Grammar")
                .arg(arg!(<CFG> "Context-Free Grammar rules file to use"))
                .arg(arg!([OUT] "Output file (default to stdout)"))
                .arg(arg!(-v --verbose "Verbose output").action(ArgAction::SetTrue))
                .arg(arg!(-d --debug "Debug output").action(ArgAction::SetTrue))
                .arg(
                    arg!(-r --reverse "Reverse the order of steps for simplifying")
                        .action(ArgAction::SetTrue),
                )
                .arg(arg!(--chomsky "Chomsky Normal Form").action(ArgAction::SetTrue)),
        )
        .subcommand(
            Command::new("earley")
                .about("Check the string via Earley recognizer")
                .arg(arg!(<CFG> "Context-Free Grammar rules file to use"))
                .arg(arg!([INPUT] "Input stream (default: stdin)"))
                .arg(arg!(-s --simplify "Use Simplified Form").action(ArgAction::SetTrue))
                .arg(arg!(-c --chomsky "Chomsky Normal Form").action(ArgAction::SetTrue)),
        )
        .subcommand(
            Command::new("cyk")
                .about("Check the string via CYK recognizer")
                .arg(arg!(<CFG> "Context-Free Grammar rules file to use"))
                .arg(arg!([INPUT] "Input stream (default: stdin)"))
                .arg(arg!(-p --parse "Build parse tree").action(ArgAction::SetTrue)),
        )
        .subcommand(
            Command::new("dfa")
                .about("Check the string via DFA")
                .arg(arg!(<DFA> "Deterministic Finite Automaton definition (as table)"))
                .arg(arg!([INPUT] "Input stream (default: stdin)"))
                .arg(arg!(-d --debug "Debug output").action(ArgAction::SetTrue))
                .arg(arg!(-p --path "Show derivation path").action(ArgAction::SetTrue)),
        )
        .subcommand(
            Command::new("dpda")
                .about("Check the string via DPDA")
                .arg(arg!(<DPDA> "Deterministic Push Down Automaton definition"))
                .arg(arg!([INPUT] "Input stream (default: stdin)"))
        )
        .subcommand(
            Command::new("dpdt")
                .about("Convert the string via DPDT")
                .arg(arg!(<DPDT> "Deterministic Push Down Translator definition"))
                .arg(arg!([INPUT] "Input stream (default: stdin)"))
        )
        .subcommand(
            Command::new("coursework")
                .about("Course Work #7")
                .arg(arg!(<CFG> "Context-Free Grammar rules file to use"))
                .arg(arg!([OUT] "Output file (default to stdout)"))
                .arg(
                    arg!(--len_min <min> "Minimum sequence lenght (default 0)")
                        .value_parser(value_parser!(u32)),
                )
                .arg(
                    arg!(--len_max <max> "Maximum sequence lenght (default 8)")
                        .value_parser(value_parser!(u32)),
                )
                .arg(arg!(-v --verbose "Verbose output").action(ArgAction::SetTrue))
        )
}
