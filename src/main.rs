#![cfg_attr(feature = "flame_it", feature(proc_macro_hygiene))]

use std::fs::File;
use std::io::{self, BufRead, BufReader, Read, Write};
use structopt::StructOpt;

#[cfg(feature = "flame_it")]
extern crate flame;
#[cfg(feature = "flame_it")]
#[macro_use]
extern crate flamer;

mod choice;
mod config;
use config::Config;

#[cfg_attr(feature = "flame_it", flame)]
fn main() {
    let opt = config::Opt::from_args();
    let config = Config::new(opt);

    let read = match &config.opt.input {
        Some(f) => Box::new(File::open(f).expect("Could not open file")) as Box<dyn Read>,
        None => Box::new(io::stdin()) as Box<dyn Read>,
    };

    let buf = BufReader::new(read);

    let stdout = io::stdout();
    let lock = stdout.lock();
    let mut handle = io::BufWriter::new(lock);

    let lines = buf.lines();
    for line in lines {
        match line {
            Ok(l) => {
                for choice in &config.opt.choice {
                    choice.print_choice(&l, &config, &mut handle);
                }
                writeln!(handle, "");
            }
            Err(e) => println!("ERROR: {}", e),
        }
    }

    #[cfg(feature = "flame_it")]
    flame::dump_html(File::create("flamegraph.html").unwrap()).unwrap();
}
