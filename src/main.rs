#![allow(clippy::comparison_chain)]
#![allow(clippy::match_like_matches_macro)]

mod bottomup_solver;
mod egg_solver;
mod parse;

use std::env;
use std::time::Instant;

use anyhow::Result;

use crate::bottomup_solver::BottomUpSynthesizer;
use crate::egg_solver::{EggSynthesizer, MyAstSize, VariedWeight};
use crate::parse::io_example_from_file;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() - 1 < 5 {
        println!();
        println!("usage: egsolver <sygus_filename> <algo> <maxsize> <e1> <e2>");
        println!("algo:  both, egg, baseline");
        println!("e1:    number of egg answers for cost function: MyAstSize");
        println!("e2:    number of egg answers for cost function: VariedWeight");
        println!();
        std::process::exit(1);
    }

    let filename = &args[1];
    let algo = &args[2];
    let maxsize = args[3].parse::<usize>()?;
    let e1 = args[4].parse::<i32>()?;
    let e2 = args[5].parse::<i32>()?;
    println!();
    println!("reading from sygus file {}", filename);
    println!("algo = {}", &algo);
    println!("maxsize = {}", maxsize);
    println!("e1 = {}", e1);
    println!("e2 = {}", e2);
    println!();
    let exp = io_example_from_file(filename)?;
    if exp.len() > 60 {
        println!("too many examples {} > 60. I cannot handle this", exp.len());
        std::process::exit(1);
    }

    let io_spec = exp;
    println!("io_spec.len = {}", &io_spec.len());

    if algo == "both" || algo == "baseline" {
        println!("\n---Running baseline");
        for enable_oe in [true] {
            for enable_ft in [true] {
                let now = Instant::now();
                let mut synthesizer =
                    BottomUpSynthesizer::new(io_spec.clone(), enable_oe, enable_ft);
                println!("-----");
                if let Some(u) = synthesizer.synthesize(maxsize) {
                    println!("{}", u);
                }
                println!(
                    "enable_oe = {}, enable_ft = {}, time = {}ms",
                    enable_oe,
                    enable_ft,
                    now.elapsed().as_millis()
                );
            }
        }
    }

    if algo == "both" || algo == "egg" {
        println!("\n--- Running egg");
        let search_time = Instant::now();
        let mut egsolver = EggSynthesizer::new(io_spec);
        let res = egsolver.synthesize(maxsize);
        println!("egg. search time = {}ms", search_time.elapsed().as_millis());

        if let Some(id) = res {
            let extract_time = Instant::now();
            println!("------cost func = ast size ------");
            egsolver.print_equivalents::<MyAstSize>(id, e1);
            println!("------cost func = varied weight ------");
            egsolver.print_equivalents::<VariedWeight>(id, e2);
            println!("egg. extract_time time = {}ms", extract_time.elapsed().as_millis());
        }
    }
    Ok(())
}
