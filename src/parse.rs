use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "./sygus.pest"]
pub struct SyGuSParser;

