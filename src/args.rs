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
                ),
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
                ),
        )
        .subcommand(
            SubCommand::with_name("parse")
                .about("Parse Context-Free Grammar rules")
                .arg(
                    Arg::with_name("CFG")
                        .help("Context-Free Grammar rules file to use")
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::with_name("verbose")
                        .long("verbose")
                        .short("v")
                        .help("Verbose output and simplification steps"),
                ),
        )
        .get_matches()
}
