mod bottomup_solver;
mod egg_solver;
mod parse;

use std::env;
use std::time::Instant;

use anyhow::Result;

use crate::bottomup_solver::BottomUpSynthesizer;
use crate::egg_solver::EggSynthesizer;
use crate::parse::io_example_from_file;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() - 1 < 3 {
        println!();
        println!("usage: egsolver sygus_filename maxsize number_off_egg_answers");
        println!();
        std::process::exit(1);
    }
    let filename = &args[1];
    let maxsize = args[2].parse::<usize>()?;
    let equivalents = args[3].parse::<i32>()?;
    println!("reading from sygus file {}", filename);
    println!("maxsize = {}", maxsize);
    let exp = io_example_from_file(filename)?;

    // let lowest_erased = |x: u64| x.wrapping_add(NEG1) & x;
    // let f = lowest_erased;
    // let io_spec = vec![1, 2, 3, 18, 256]
    //     .into_iter()
    //     .map(|x: u64| (x, f(x)))
    //     .collect::<IOMapT>();

    let io_spec = exp;
    dbg!(&io_spec);

    println!("\n---Running baseline");
    for enable_oe in [true] {
        for enable_ft in [true] {
            let now = Instant::now();
            let mut synthesizer = BottomUpSynthesizer::new(io_spec.clone(), enable_oe, enable_ft);
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

    println!("\n--- Running egg");
    let now = Instant::now();
    let mut egsolver = EggSynthesizer::new(io_spec);
    let res = egsolver.synthesize(maxsize);
    println!("egg. time = {}ms", now.elapsed().as_millis());
    if let Some(id) = res {
        egsolver.print_equivalents(id, equivalents);
    }
    Ok(())
}
