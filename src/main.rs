mod bottomup_solver;
mod egg_solver;
mod parse;

use std::env;
use std::time::Instant;

use anyhow::Result;

use egg::{AstSize, Extractor};

use crate::bottomup_solver::{BottomUpSynthesizer, IOMapT, NEG1};
use crate::egg_solver::EggSynthesizer;
use crate::parse::io_example_from_file;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() - 1 < 2 {
        panic!("usage: egsolver sygus_filename maxsize");
    }
    let filename = &args[1];
    let maxsize = args[2].parse::<usize>()?;
    println!("reading from sygus file {}", filename);
    println!("maxsize = {}", maxsize);
    let exp = io_example_from_file(filename)?;
    dbg!(&exp);

    // let lowest_erased = |x: u64| x.wrapping_add(NEG1) & x;
    // let f = lowest_erased;
    // let io_spec = vec![1, 2, 3, 18, 256]
    //     .into_iter()
    //     .map(|x: u64| (x, f(x)))
    //     .collect::<IOMapT>();

    let io_spec = exp;

    println!("Given io spec\n{:?}", io_spec);
    println!("\n---Running baseline");
    for enable_oe in [true] {
        for enable_mc in [false, true] {
            println!();
            for _ in [1, 2, 3] {
                let now = Instant::now();
                let mut synthesizer =
                    BottomUpSynthesizer::new(io_spec.clone(), enable_oe, enable_mc);
                println!("-----");
                if let Some(u) = synthesizer.synthesize(maxsize) {
                    println!("{}", u);
                }
                println!(
                    "enable_oe = {}, enable_mc = {}, time = {}ms",
                    enable_oe,
                    enable_mc,
                    now.elapsed().as_millis()
                );
            }
        }
    }

    println!("\n--- Running egg");
    let mut egsolver = EggSynthesizer::new(io_spec);
    if let Some(id) = egsolver.synthesize(maxsize) {
        println!("{:?}", id);
        let ext = Extractor::new(&egsolver.bank, AstSize);
        let (_, ast) = ext.find_best(id);
        println!("{}", ast.pretty(80).replace("18446744073709551615", "-1"));
    }
    Ok(())
}
