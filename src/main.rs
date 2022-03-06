mod bottomup_solver;
mod egg_solver;
mod parse;

use anyhow::Result;

use egg::{AstSize, Extractor};

use crate::bottomup_solver::{BottomUpSynthesizer, IOMapT, NEG1};
use crate::egg_solver::EggSynthesizer;
use crate::parse::io_example_from_file;

fn main() -> Result<()> {
    let filename = "./bv-tests/test1.sl";
    let exp = io_example_from_file(filename)?;
    dbg!(exp);

    let lowest_erased = |x: u64| x.wrapping_add(NEG1) & x;
    let f = lowest_erased;
    let io_spec = vec![1, 2, 3, 18, 256]
        .into_iter()
        .map(|x: u64| (x, f(x)))
        .collect::<IOMapT>();

    println!("{:?}", io_spec);
    let mut synthesizer = BottomUpSynthesizer::new(io_spec.clone());
    if let Some(u) = synthesizer.synthesize(6) {
        println!("{:?}", u);
        println!("{}", u);
    }

    let mut egsolver = EggSynthesizer::new(io_spec);
    if let Some(id) = egsolver.synthesize(6) {
        println!("{:?}", id);
        let ext = Extractor::new(&egsolver.bank, AstSize);
        let (_, ast) = ext.find_best(id);
        println!("{}", ast.pretty(80).replace("18446744073709551615", "-1"));
    }
    Ok(())
}
