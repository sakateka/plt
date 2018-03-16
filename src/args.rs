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
        .get_matches()
}
