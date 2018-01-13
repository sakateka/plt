use clap::{App, Arg, ArgMatches, SubCommand};

pub fn build_app<'a>(name: &str) -> ArgMatches<'a> {
    App::new(name)
        .version("0.1.0")
        .author("Sergey K. <uo0@ya.ru>")
        .about("PLT works")
        .subcommand(SubCommand::with_name("generate")
                .about("Generate strings by CFG")
                .arg(Arg::with_name("GRAMMAR")
                        .help("Context free grammar file to use")
                        .required(true)
                        .index(1),
                )
                .arg(Arg::with_name("OUT")
                        .help("Output file (default to stdout)")
                        .required(false)
                        .index(2),
                ),
        )
        .get_matches()
}
