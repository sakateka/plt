use serde_derive::Deserialize;

use crate::cfg;
use std::collections::HashMap;
use std::io::{self, BufRead, BufReader};
use std::fs::File;

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct SDTRule {
    left: String,
    right: String,
    translated: String,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct SDTRules{
    rules: Vec<SDTRule>,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct SDT {
    rules: HashMap<cfg::Production, Vec<cfg::Symbol>>,
}

#[allow(dead_code)]
impl SDT {
    pub fn load(input_path: &str) -> io::Result<SDT> {
        let file = BufReader::new(File::open(input_path)?);
        SDT::load_from_reader(file)
    }

    pub fn load_from_reader<R: ?Sized + BufRead>(r: R) -> io::Result<SDT>
    where
        R: ::std::marker::Sized,
    {
        let _rules: SDTRules = match serde_yaml::from_reader(r) {
            Ok(sdt) => Ok(sdt),
            Err(err) => Err(io::Error::new(io::ErrorKind::Other, err)),
        }?;
        todo!();
        //Ok(SDT {
        //    rules: HashMap::new(),
        //})
    }
}
