use clap::{Command, Arg};


pub fn build_app() -> Command {
    Command::new("plt")
        .version("5.4.0")
        .author("Sergey Kacheev <uo0@ya.ru>")
        .about("Theory of Programming Languages and Translation Methods")
        .subcommand(
            Command::new("gen")
                .about("Sequence generator by CFG")
                .arg(
                    Arg::new("right")
                        .long("right")
                        .short('r')
                        .help("Use the right-hand derivation (default left-hand)"),
                )
                .arg(
                    Arg::new("len-min")
                        .long("len-min")
                        .num_args(1)
                        .help("Minimum sequence lenght (default 0)"),
                )
                .arg(
                    Arg::new("len-max")
                        .long("len-max")
                        .num_args(1)
                        .help("Maximum sequence lenght (default 8)"),
                )
                .arg(
                    Arg::new("all")
                        .long("all")
                        .short('a')
                        .help("Show all sequences together with duplicates"),
                )
                .arg(
                    Arg::new("chomsky")
                        .long("chomsky")
                        .help("Chomsky Normal Form"),
                )
                .arg(
                    Arg::new("CFG")
                        .help("Context-Free Grammar rules file to use")
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::new("OUT")
                        .help("Output file (default to stdout)")
                        .required(false)
                        .index(2),
                ),
        )
        .subcommand(
            Command::new("simplify")
                .about("Simplify Context-Free Grammar")
                .arg(
                    Arg::new("CFG")
                        .help("Context-Free Grammar rules file to use")
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::new("OUT")
                        .required(false)
                        .help("Output file (default to stdout)")
                        .index(2),
                )
                .arg(
                    Arg::new("verbose")
                        .long("verbose")
                        .short('v')
                        .help("Verbose output"),
                )
                .arg(
                    Arg::new("debug")
                        .long("debug")
                        .short('d')
                        .help("Debug output"),
                )
                .arg(
                    Arg::new("reverse")
                        .long("reverse")
                        .short('r')
                        .help("Reverse the order of steps for simplifying"),
                )
                .arg(
                    Arg::new("chomsky")
                        .long("chomsky")
                        .help("Chomsky Normal Form"),
                ),
        )
        .subcommand(
            Command::new("earley")
                .about("Check the string via Earley recognizer")
                .arg(Arg::new("CFG").help("Path to CFG").required(true).index(1))
                .arg(
                    Arg::new("INPUT")
                        .required(false)
                        .help("Input stream (default: stdin)")
                        .index(2),
                )
                .arg(
                    Arg::new("simplify")
                        .long("simplify")
                        .short('s')
                        .help("Use Simplified Form"),
                )
                .arg(
                    Arg::new("chomsky")
                        .long("chomsky")
                        .short('c')
                        .help("Use Chomsky Normal Form"),
                ),
        )
        .subcommand(
            Command::new("cyk")
                .about("Check the string via CYK recognizer")
                .arg(Arg::new("CFG").help("Path to CFG").required(true).index(1))
                .arg(
                    Arg::new("INPUT")
                        .required(false)
                        .help("Input stream (default: stdin)")
                        .index(2),
                )
                .arg(
                    Arg::new("parse")
                        .long("parse")
                        .short('p')
                        .help("Build parse tree"),
                ),
        )
        .subcommand(
            Command::new("dfa")
                .about("Check the string via DFA")
                .arg(
                    Arg::new("DFA")
                        .help("Deterministic Finite Automaton definition (as table)")
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::new("INPUT")
                        .required(false)
                        .help("Input stream (default: stdin)")
                        .index(2),
                )
                .arg(
                    Arg::new("debug")
                        .long("debug")
                        .short('d')
                        .help("Debug mode"),
                )
                .arg(
                    Arg::new("path")
                        .long("path")
                        .short('p')
                        .help("Show derivation path"),
                ),
        )
        .subcommand(
            Command::new("dpda")
                .about("Check the string via DPDA")
                .arg(
                    Arg::new("DPDA")
                        .help("Deterministic Push Down Automaton definition")
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::new("INPUT")
                        .required(false)
                        .help("Input stream (default: stdin)")
                        .index(2),
                ),
        )
        .subcommand(
            Command::new("dpdt")
                .about("Convert the string via DPDT")
                .arg(
                    Arg::new("DPDT")
                        .help("Deterministic Push Down Translator definition")
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::new("INPUT")
                        .required(false)
                        .help("Input stream (default: stdin)")
                        .index(2),
                ),
        )
        .subcommand(
            Command::new("coursework")
                .about("Course Work #7")
                .arg(
                    Arg::new("len-min")
                        .long("len-min")
                        .num_args(1)
                        .help("Minimum sequence lenght (default 0)"),
                )
                .arg(
                    Arg::new("len-max")
                        .long("len-max")
                        .num_args(1)
                        .help("Maximum sequence lenght (default 8)"),
                )
                .arg(
                    Arg::new("verbose")
                        .long("verbose")
                        .short('v')
                        .help("Show generated sets"),
                )
                .arg(
                    Arg::new("CFG")
                        .help("Context-Free Grammar rules file to use")
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::new("OUT")
                        .help("Output file (default to stdout)")
                        .required(false)
                        .index(2),
                ),
        )
}
