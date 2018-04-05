use clap::{App, Arg, ArgMatches, SubCommand};

pub fn build_app<'a>(name: &str) -> ArgMatches<'a> {
    App::new(name)
        .version("0.1.0")
        .author("Sergey Kacheev <uo0@ya.ru>")
        .about("Theory of Programming Languages and Translation Methods")
        .subcommand(
            SubCommand::with_name("gen")
                .about("Sequence generator by CFG")
                .arg(
                    Arg::with_name("right")
                        .long("right")
                        .short("r")
                        .help("Use the right-hand derivation (default left-hand)"),
                )
                .arg(
                    Arg::with_name("len-min")
                        .long("len-min")
                        .takes_value(true)
                        .help("Minimum sequence lenght (default 0)"),
                )
                .arg(
                    Arg::with_name("len-max")
                        .long("len-max")
                        .takes_value(true)
                        .help("Maximum sequence lenght (default 8)"),
                )
                .arg(
                    Arg::with_name("all")
                        .long("all")
                        .short("a")
                        .help("Show all sequences together with duplicates"),
                )
                .arg(
                    Arg::with_name("chomsky")
                        .long("chomsky")
                        .help("Chomsky Normal Form"),
                )
                .arg(
                    Arg::with_name("CFG")
                        .help("Context-Free Grammar rules file to use")
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::with_name("OUT")
                        .help("Output file (default to stdout)")
                        .required(false)
                        .index(2),
                )
        )
        .subcommand(
            SubCommand::with_name("simplify")
                .about("Simplify Context-Free Grammar")
                .arg(
                    Arg::with_name("CFG")
                        .help("Context-Free Grammar rules file to use")
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::with_name("OUT")
                        .required(false)
                        .help("Output file (default to stdout)")
                        .index(2),
                )
                .arg(
                    Arg::with_name("verbose")
                        .long("verbose")
                        .short("v")
                        .help("Verbose output"),
                )
                .arg(
                    Arg::with_name("debug")
                        .long("debug")
                        .short("d")
                        .help("Debug output"),
                )
                .arg(
                    Arg::with_name("reverse")
                        .long("reverse")
                        .short("r")
                        .help("Reverse the order of steps for simplifying"),
                )
                .arg(
                    Arg::with_name("chomsky")
                        .long("chomsky")
                        .help("Chomsky Normal Form"),
                )
        )
        .subcommand(
            SubCommand::with_name("earley")
                .about("Check the string via Earley recognizer")
                .arg(
                    Arg::with_name("CFG")
                        .help("Path to CFG")
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::with_name("INPUT")
                        .required(false)
                        .help("Input stream (default: stdin)")
                        .index(2),
                )
                .arg(
                    Arg::with_name("simplify")
                        .long("simplify")
                        .short("s")
                        .help("Use Simplified Form"),
                )
                .arg(
                    Arg::with_name("chomsky")
                        .long("chomsky")
                        .short("c")
                        .help("Use Chomsky Normal Form"),
                )
        )
        .subcommand(
            SubCommand::with_name("dfa")
                .about("Check the string via DFA")
                .arg(
                    Arg::with_name("DFA")
                        .help("Deterministic Finite Automaton definition (as table)")
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::with_name("INPUT")
                        .required(false)
                        .help("Input stream (default: stdin)")
                        .index(2),
                )
                .arg(
                    Arg::with_name("debug")
                        .long("debug")
                        .short("d")
                        .help("Debug mode"),
                )
                .arg(
                    Arg::with_name("path")
                        .long("path")
                        .short("p")
                        .help("Show derivation path"),
                )
        )
        .subcommand(
            SubCommand::with_name("dpda")
                .about("Check the string via DPDA")
                .arg(
                    Arg::with_name("DPDA")
                        .help("Deterministic Push Down Automaton definition")
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::with_name("INPUT")
                        .required(false)
                        .help("Input stream (default: stdin)")
                        .index(2),
                )
        )
        .subcommand(
            SubCommand::with_name("dpdt")
                .about("Convert the string via DPDT")
                .arg(
                    Arg::with_name("DPDT")
                        .help("Deterministic Push Down Translator definition")
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::with_name("INPUT")
                        .required(false)
                        .help("Input stream (default: stdin)")
                        .index(2),
                )
        )
        .subcommand(
            SubCommand::with_name("coursework")
                .about("Course Work #7")
                .arg(
                    Arg::with_name("len-min")
                        .long("len-min")
                        .takes_value(true)
                        .help("Minimum sequence lenght (default 0)"),
                )
                .arg(
                    Arg::with_name("len-max")
                        .long("len-max")
                        .takes_value(true)
                        .help("Maximum sequence lenght (default 8)"),
                )
                .arg(
                    Arg::with_name("verbose")
                        .long("verbose")
                        .short("v")
                        .help("Show generated sets"),
                )
                .arg(
                    Arg::with_name("CFG")
                        .help("Context-Free Grammar rules file to use")
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::with_name("OUT")
                        .help("Output file (default to stdout)")
                        .required(false)
                        .index(2),
                )
        )
        .get_matches()
}
