use cfg;
use std::collections::HashMap;

#[derive(Debug)]
pub struct SDT<'sdt> {
    cfg: cfg::CFG,
    rules: HashMap<&'sdt cfg::Production, Vec<cfg::Symbol>>,
}
