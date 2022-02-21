mod parse;

use crate::parse::{Rule, SyGuSParser};
use egg::*;
use pest::Parser;
use std::env;
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;

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
    let filename = "./src/test1.sl";
    let contents = fs::read_to_string(filename).expect("Something went wrong reading the file");
    //println!("{}", contents);
    let p = SyGuSParser::parse(Rule::entry, &contents).unwrap();
    //dbg!(p);

    for pair in p {
        for inner1 in pair.into_inner() {
          for inner2 in inner1.into_inner() {
          //println!("{}", inner.as_str());
          // println!("Rule:    {:?}", inner2.as_rule());
            if (inner2.as_rule() == Rule::constraint){
              println!("constraint is {}", inner2.as_str());
            }
            }
        }
      }

    //     //print_type_of(&pair);
    // }
    //print_type_of(p);
    //getConstriant(p);
}

fn print_type_of<T>(_: T) {
    println!("{}", std::any::type_name::<T>())
}

