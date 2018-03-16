#[macro_use]
extern crate clap;
extern crate itertools;

mod args;
mod cfg;
mod generator;

use std::collections::HashSet;
use std::fs::File;
use std::io::Write;
use generator::{GeneratedSet, Generator};
use cfg::CFG;

fn main() {
    let app = args::build_app("plt");

    if let Some(matches) = app.subcommand_matches("gen") {
        let grammar = matches.value_of("CFG").unwrap();
        let cfg = CFG::parse(grammar).unwrap();
        let mut min: u32 = 0;
        if matches.is_present("len-min") {
            min = value_t_or_exit!(matches, "len-min", u32);
        }
        let mut max: u32 = 8;
        if matches.is_present("len-max") {
            max = value_t_or_exit!(matches, "len-max", u32);
        }
        let left = !matches.is_present("right");
        let generated = GeneratedSet(
            Generator::new(cfg, min, max, left).collect::<HashSet<Vec<cfg::Symbol>>>(),
        );
        print!("{}", generated);
    } else if let Some(matches) = app.subcommand_matches("simplify") {
        let grammar = matches.value_of("CFG").unwrap();
        let cfg = CFG::parse(grammar).unwrap();
        let output = matches.value_of("OUT").unwrap_or_else(|| "/dev/stdout");
        let mut output_stream = File::create(output).unwrap();
        if matches.is_present("verbose") {
            println!("Output to {}", output);
            output_stream
                .write_fmt(format_args!("Yep Simplify!\n{}\nTo:\n", cfg,))
                .unwrap();
        }
        output_stream
            .write_fmt(format_args!("{}", cfg.simplify()))
            .unwrap();
    }
}
