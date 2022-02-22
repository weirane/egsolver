use std::collections::HashMap;
use std::path::Path;

use anyhow::Result;
use pest::Parser;
use pest_derive::Parser;

use crate::ValueT;

#[derive(Parser)]
#[grammar = "./sygus.pest"]
pub struct SyGuSParser;

#[derive(Debug, PartialEq)]
pub struct Sygus<I, O> {
    grammar: Grammar,
    examples: Vec<(I, O)>,
}

#[derive(Debug, PartialEq)]
pub struct Grammar {
    pub productions: HashMap<Symbol, Production>,
    pub start: Symbol,
}

#[derive(Debug, PartialEq)]
pub struct Production {
    pub name: Symbol,
    pub sort: Sort,
    pub rhs: Vec<Symbol>,
}

pub type Sort = String; // TODO: change
pub type Symbol = String;

pub fn io_example_from_file(filename: impl AsRef<Path>) -> Result<Vec<(ValueT, ValueT)>> {
    let contents = std::fs::read_to_string(filename)?;
    let pairs = SyGuSParser::parse(Rule::entry, &contents)?;
    pairs
        .into_iter()
        .flat_map(|p| p.into_inner())
        .filter(|i| i.as_rule() == Rule::constraint)
        .filter_map(|i| i.into_inner().next())
        // --- ignore above 4 lines: pest structure related ---
        // i.as_str() = "(= (f #xbeb187cd6ed5b4bd) #x0000beb187cd6ed6)"
        .map(|i| i.as_str())
        .filter(|s| s.starts_with("(= (f "))
        .map(|s| {
            let input = ValueT::from_str_radix(&s[8..24], 16)?;
            let output = ValueT::from_str_radix(&s[28..44], 16)?;
            Ok((input, output))
        })
        .collect()
}
