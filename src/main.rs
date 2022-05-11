use clap::Parser;
use std::{fs::File, io::prelude::*, path::PathBuf};

mod brainfuck;

use brainfuck::{VMOptions, VM};

#[derive(Debug, Parser)]
#[clap(version, long_about = "A fast brainfuck interpreter written in rust.")]
struct Opt {
    /// Disables optimizer (might improve performance in small programs)
    #[clap(long)]
    no_optimize: bool,

    /// Disables comment Characters (# and ;)
    #[clap(long)]
    no_comments: bool,

    /// Input File
    #[clap(parse(from_os_str))]
    input: PathBuf,
}

fn main() {
    let opt = Opt::parse();
    let display = opt.input.display();

    let mut file = match File::open(&opt.input) {
        Err(why) => {
            eprintln!("couldn't open {}: {}", display, why);
            return;
        }
        Ok(file) => file,
    };

    let mut program = String::new();
    if let Err(why) = file.read_to_string(&mut program) {
        eprintln!("couldn't read {}: {}", display, why);
        return;
    };

    let options = VMOptions {
        program,
        disable_optimizer: opt.no_optimize,
        disable_comments: opt.no_comments,
    };

    let mut vm = VM::new(options);
    vm.run();
}
