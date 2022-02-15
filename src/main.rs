mod parse;

use egg::*;
use pest::Parser;

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

    let p = SyGuSParser::parse(Rule::literal, "10.23").unwrap();
    dbg!(p);
}
