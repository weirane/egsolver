mod parse;

use egg::*;
use pest::Parser;
use std::env;
use std::fs;

use crate::parse::{Rule, SyGuSParser};

define_language! {
    enum BitVector {
        "&" = And([Id; 2]),
        "|" = Or([Id; 2]),
        "^" = Xor([Id; 2]),
        "~" = Not(Id),
        Lit(i32),
    }
}

fn main() {
    let mut eg: EGraph<BitVector, ()> = Default::default();
    let one = eg.add(BitVector::Lit(1));
    eg.add(BitVector::And([one, one]));
    println!("{:?}", eg);
    // eg.dot().to_svg("egraph.svg").unwrap();
    let filename  = "./src/test.sl";
    let contents = fs::read_to_string(filename)
        .expect("Something went wrong reading the file");
    //println!("{}", contents);
    let p = SyGuSParser::parse(Rule::entry, &contents).unwrap();
    dbg!(p);
}
