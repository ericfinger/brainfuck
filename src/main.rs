use std::{fs::File, io::prelude::*, path::PathBuf};
use structopt::StructOpt;

mod brainfuck;

use brainfuck::{VMOptions, VM};

#[derive(Debug, StructOpt)]
#[structopt(
    name = "Brainfuck",
    about = "A fast brainfuck interpreter written in rust."
)]
struct Opt {
    /// Disables optimizer (might improve performance in small programs)
    #[structopt(long)]
    no_optimize: bool,

    /// Disables comment Characters (# and ;)
    #[structopt(long)]
    no_comments: bool,

    /// Input File
    #[structopt(parse(from_os_str))]
    input: PathBuf,
}

fn main() {
    let opt = Opt::from_args();
    let display = opt.input.display();

    let mut file = match File::open(&opt.input) {
        Err(why) => {
            eprintln!("couldn't open {}: {}", display, why);
            return;
        }
        Ok(file) => file,
    };

    let mut program = String::new();
    match file.read_to_string(&mut program) {
        Err(why) => {
            eprintln!("couldn't read {}: {}", display, why);
            return;
        }
        Ok(_) => (),
    };

    let options = VMOptions {
        program,
        disable_optimizer: opt.no_optimize,
        disable_comments: opt.no_comments,
    };

    let mut vm = VM::new(options);
    vm.run();
}
