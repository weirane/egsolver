use std::collections::HashMap;
use std::path::Path;
//use std::slice::range;

use anyhow::Result;
use pest::{iterators::Pair, Parser};
use pest_derive::Parser;

use crate::bottomup_solver::{IOMapT, ValueT};
use crate::PBE_string_solver_bottomup;
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

pub fn string_io_example_from_file(
    filename: impl AsRef<Path>,
) -> Result<(PBE_string_solver_bottomup::IOMapT, Vec<String>, Vec<i32>)> {
    let contents = std::fs::read_to_string(filename)?;
    let pairs = SyGuSParser::parse(Rule::entry, &contents)?;
    //println!("{}", pairs);
    let iomap = pairs
        .clone()
        .into_iter()
        .flat_map(|p| p.into_inner())
        .filter(|i| i.as_rule() == Rule::constraint)
        .filter_map(|i| i.into_inner().next())
        // --- ignore above 4 lines: pest structure related ---
        // i.as_str() = "(= (f #xbeb187cd6ed5b4bd) #x0000beb187cd6ed6)"
        .map(|i| i.as_str())
        .filter(|s| s.starts_with("(= (f "))
        .map(|s| {
            //ln!("{}", s);
            let io: Vec<&str> = s.split("\"").collect();
            let input = PBE_string_solver_bottomup::ValueT::StringV(io[1].to_string());
            let output = PBE_string_solver_bottomup::ValueT::StringV(io[3].to_string());
            Ok((input, output))
        })
        .collect::<Result<PBE_string_solver_bottomup::IOMapT>>()?;
    let mut a = vec![];
    let mut b = vec![];
    for pair in pairs.clone() {
        for inner1 in pair.into_inner() {
            if (inner1.as_rule() == Rule::synth_func) {
                for inner3 in inner1.into_inner() {
                    if (inner3.as_rule() == Rule::grammar_def) {
                        for grammardef in inner3.into_inner() {
                            //dbg!(&grammar_def.as_str());
                            let groupedlist = grammardef.into_inner();
                            for spec in groupedlist.clone() {
                                if spec.as_rule() == Rule::symbol && spec.as_str() == "ntString" {
                                    let strlist: Vec<&str> =
                                        groupedlist.as_str().split("\"").collect();
                                    let size = strlist.len() / 2;
                                    let mut i = 0;
                                    while i < size {
                                        let strV = strlist[1 + 2 * i].to_string();
                                        i += 1;
                                        a.push(strV);
                                    }
                                    //print!("size {}", size);
                                }
                                else if spec.as_rule() == Rule::symbol && spec.as_str() == "ntInt" {
                                  let intlist:Vec<&str>  =
                                      groupedlist.as_str().split("(").collect::<Vec<&str>>();
                                      let separtedint = intlist[1].split_whitespace().collect::<Vec<&str>>();
                                      //dbg!(&intlist);
                                      let mut i = 0;
                                      while i < separtedint.len() {
                                        let int_v:i32 = separtedint[i].parse().unwrap();
                                        i += 1;
                                        b.push(int_v);
                                    }
                                  
                              }
                            }
                        }
                    }
                }
            }
        }
    }
    dbg!(&a);
    dbg!(&b);
    return Ok((iomap.clone(), a, b));
}

pub fn io_example_from_file(filename: impl AsRef<Path>) -> Result<IOMapT> {
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
