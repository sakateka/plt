#[macro_use]
extern crate clap;
extern crate itertools;

mod args;
mod cfg;
mod generator;

use std::collections::HashSet;
use generator::Generator;

fn main() {
    let app = args::build_app("plt");

    if let Some(matches) = app.subcommand_matches("gen") {
        let grammar = matches.value_of("CFG").unwrap();
        let cfg = cfg::CFG::parse(grammar).unwrap();
        let mut len_min: u32 = 0; // pixels
        if matches.is_present("len-min") {
            len_min = value_t_or_exit!(matches, "len-min", u32);
        }
        let mut len_max: u32 = 8; // pixels
        if matches.is_present("len-max") {
            len_max = value_t_or_exit!(matches, "len-max", u32);
        }
        let generated = Generator::new(cfg, len_min, len_max)
            .collect::<HashSet<Vec<cfg::Symbol>>>();
        println!("Yep!\n {:?}\n", generated);
    }
}
