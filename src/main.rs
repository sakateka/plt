#[macro_use]
extern crate clap;
extern crate itertools;

mod args;
mod cfg;
mod generator;
mod dfa;

use std::collections::HashSet;
use std::fs::File;
use std::io::Write;
use generator::{GeneratedItem, GeneratedSet, Generator};
use cfg::{CFG, Symbol};
use dfa::DFA;

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
        let gen = Generator::new(cfg, min, max, left);
        if matches.is_present("all") {
            for seq in gen {
                print!("{}\n", GeneratedItem(&seq));
            }
        } else {
            print!(
                "{}",
                GeneratedSet(gen.collect::<HashSet<Vec<Symbol>>>())
            );
        }
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
    } else if let Some(matches) = app.subcommand_matches("parse") {
        let grammar = matches.value_of("CFG").unwrap();
        let cfg = CFG::parse(grammar).unwrap();
        print!("Parsed CFG\n{}", cfg);
        print!("Remove epsilon\n{}", cfg.remove_epsilon_rules());
        if matches.is_present("verbose") {
            println!("{:?}\n", cfg.remove_epsilon_rules());
        }
        print!("Remove units\n{}", cfg.remove_unit_rules());
        if matches.is_present("verbose") {
            println!("{:?}\n", cfg.remove_unit_rules());
        }
        print!("Remove useless\n{}", cfg.remove_useless_rules());
        if matches.is_present("verbose") {
            println!("{:?}\n", cfg.remove_useless_rules());
        }
        print!("Remove unreachable\n{}", cfg.remove_unreachable_rules());
        if matches.is_present("verbose") {
            println!("{:?}\n", cfg.remove_unreachable_rules());
        }
    } else if let Some(matches) = app.subcommand_matches("dfa") {
        let dfa_table = matches.value_of("DFA").unwrap();
        let debug = matches.is_present("debug");
        let show_path = matches.is_present("path");
        let mut dfa = DFA::parse(dfa_table, debug).unwrap();
        let input = matches.value_of("INPUT").unwrap_or_else(|| "/dev/stdin");
        dfa.check(input, show_path).unwrap();
    }
}
